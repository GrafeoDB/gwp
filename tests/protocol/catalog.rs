//! The full catalog flow (schemas, graphs, graph types) with IF NOT EXISTS
//! and IF EXISTS variants, and the gwp#15 regression for DDL statements.

use gwp::client::{CatalogClient, GqlConnection};
use gwp::error::GqlError;
use gwp::server::{CreateGraphConfig, GraphTypeSpec};
use gwp::status;

use crate::common::{self, DEFAULT_SCHEMA};

fn grpc_code<T: std::fmt::Debug>(result: Result<T, GqlError>) -> tonic::Code {
    match result {
        Err(GqlError::Grpc(status)) => status.code(),
        other => panic!("expected a gRPC error, got {other:?}"),
    }
}

fn graph_config(schema: &str, name: &str) -> CreateGraphConfig {
    CreateGraphConfig {
        schema: schema.to_owned(),
        name: name.to_owned(),
        if_not_exists: false,
        or_replace: false,
        type_spec: None,
        copy_of: None,
        storage_mode: "InMemory".to_owned(),
        memory_limit_bytes: None,
        backward_edges: None,
        threads: None,
        wal_enabled: None,
        wal_durability: None,
    }
}

async fn catalog() -> (common::TestServer, CatalogClient) {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let client = conn.create_catalog_client();
    (server, client)
}

#[tokio::test]
async fn schema_lifecycle() {
    let (_server, mut catalog) = catalog().await;

    let names: Vec<String> = catalog
        .list_schemas()
        .await
        .unwrap()
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert_eq!(names, vec![DEFAULT_SCHEMA]);

    catalog.create_schema("sales", false).await.unwrap();
    assert_eq!(
        grpc_code(catalog.create_schema("sales", false).await),
        tonic::Code::AlreadyExists
    );
    // IF NOT EXISTS on an existing schema is a no-op.
    catalog.create_schema("sales", true).await.unwrap();

    let schemas = catalog.list_schemas().await.unwrap();
    assert_eq!(schemas.len(), 2);
    assert!(schemas.iter().any(|s| s.name == "sales"));

    assert!(catalog.drop_schema("sales", false).await.unwrap());
    assert_eq!(
        grpc_code(catalog.drop_schema("sales", false).await),
        tonic::Code::NotFound
    );
    // IF EXISTS on a missing schema reports that nothing was dropped.
    assert!(!catalog.drop_schema("sales", true).await.unwrap());

    assert_eq!(
        grpc_code(catalog.create_schema("", false).await),
        tonic::Code::InvalidArgument
    );
    assert_eq!(
        grpc_code(catalog.drop_schema("", true).await),
        tonic::Code::InvalidArgument
    );
}

#[tokio::test]
async fn graph_lifecycle() {
    let (server, mut catalog) = catalog().await;
    catalog.create_schema("sales", false).await.unwrap();

    let mut config = graph_config("sales", "orders");
    config.memory_limit_bytes = Some(1 << 30);
    config.backward_edges = Some(true);
    config.threads = Some(4);
    config.storage_mode = "Persistent".to_owned();
    let created = catalog.create_graph(config.clone()).await.unwrap();
    assert_eq!(created.schema, "sales");
    assert_eq!(created.name, "orders");

    // Options reach the backend and come back through GetGraphInfo.
    let info = catalog.get_graph_info("sales", "orders").await.unwrap();
    assert_eq!(info.storage_mode, "Persistent");
    assert_eq!(info.memory_limit_bytes, Some(1 << 30));
    assert_eq!(info.backward_edges, Some(true));
    assert_eq!(info.threads, Some(4));

    // Plain CREATE on an existing graph fails...
    assert_eq!(
        grpc_code(catalog.create_graph(config.clone()).await),
        tonic::Code::AlreadyExists
    );
    // ...IF NOT EXISTS succeeds and returns the existing graph...
    let mut idempotent = graph_config("sales", "orders");
    idempotent.if_not_exists = true;
    let existing = catalog.create_graph(idempotent).await.unwrap();
    assert_eq!(existing.name, "orders");
    // ...and OR REPLACE replaces it.
    let mut replace = graph_config("sales", "orders");
    replace.or_replace = true;
    catalog.create_graph(replace).await.unwrap();
    let replaced = catalog.get_graph_info("sales", "orders").await.unwrap();
    assert_eq!(replaced.storage_mode, "InMemory");

    let graphs = catalog.list_graphs("sales").await.unwrap();
    assert_eq!(graphs.len(), 1);
    assert!(
        catalog
            .list_graphs(DEFAULT_SCHEMA)
            .await
            .unwrap()
            .is_empty()
    );

    // A schema with graphs cannot be dropped.
    assert_eq!(
        grpc_code(catalog.drop_schema("sales", false).await),
        tonic::Code::InvalidArgument
    );

    assert!(catalog.drop_graph("sales", "orders", false).await.unwrap());
    assert_eq!(
        grpc_code(catalog.drop_graph("sales", "orders", false).await),
        tonic::Code::NotFound
    );
    assert!(!catalog.drop_graph("sales", "orders", true).await.unwrap());
    assert_eq!(
        grpc_code(catalog.get_graph_info("sales", "orders").await),
        tonic::Code::NotFound
    );
    assert!(!server.backend.has_graph("sales", "orders"));
    assert!(catalog.drop_schema("sales", false).await.unwrap());
}

#[tokio::test]
async fn graph_requests_are_validated() {
    let (server, mut catalog) = catalog().await;

    assert_eq!(
        grpc_code(catalog.create_graph(graph_config(DEFAULT_SCHEMA, "")).await),
        tonic::Code::InvalidArgument
    );
    assert_eq!(
        grpc_code(catalog.get_graph_info(DEFAULT_SCHEMA, "").await),
        tonic::Code::InvalidArgument
    );
    assert_eq!(
        grpc_code(catalog.drop_graph(DEFAULT_SCHEMA, "", true).await),
        tonic::Code::InvalidArgument
    );
    assert_eq!(
        grpc_code(catalog.create_graph(graph_config("nowhere", "g")).await),
        tonic::Code::NotFound
    );

    // `copy_of` reaches the backend unchanged.
    let mut copy = graph_config(DEFAULT_SCHEMA, "copy");
    copy.copy_of = Some("default.original".to_owned());
    catalog.create_graph(copy).await.unwrap();
    assert_eq!(
        server.backend.events_with("create_graph default."),
        vec!["create_graph default.copy copy_of=Some(\"default.original\")"]
    );
}

#[tokio::test]
async fn graph_type_lifecycle() {
    let (_server, mut catalog) = catalog().await;

    catalog
        .create_graph_type(DEFAULT_SCHEMA, "Social", false, false)
        .await
        .unwrap();
    assert_eq!(
        grpc_code(
            catalog
                .create_graph_type(DEFAULT_SCHEMA, "Social", false, false)
                .await
        ),
        tonic::Code::AlreadyExists
    );
    catalog
        .create_graph_type(DEFAULT_SCHEMA, "Social", true, false)
        .await
        .unwrap();
    catalog
        .create_graph_type(DEFAULT_SCHEMA, "Social", false, true)
        .await
        .unwrap();

    let types = catalog.list_graph_types(DEFAULT_SCHEMA).await.unwrap();
    assert_eq!(types.len(), 1);
    assert_eq!(types[0].name, "Social");

    // Graphs can be typed by a named graph type, which must exist.
    let mut network = graph_config(DEFAULT_SCHEMA, "network");
    network.type_spec = Some(GraphTypeSpec::Named("Social".to_owned()));
    let info = catalog.create_graph(network).await.unwrap();
    assert_eq!(info.graph_type, "Social");

    let mut unknown = graph_config(DEFAULT_SCHEMA, "other");
    unknown.type_spec = Some(GraphTypeSpec::Named("Missing".to_owned()));
    assert_eq!(
        grpc_code(catalog.create_graph(unknown).await),
        tonic::Code::NotFound
    );

    let mut open = graph_config(DEFAULT_SCHEMA, "open");
    open.type_spec = Some(GraphTypeSpec::Open);
    assert_eq!(catalog.create_graph(open).await.unwrap().graph_type, "");

    assert!(
        catalog
            .drop_graph_type(DEFAULT_SCHEMA, "Social", false)
            .await
            .unwrap()
    );
    assert_eq!(
        grpc_code(
            catalog
                .drop_graph_type(DEFAULT_SCHEMA, "Social", false)
                .await
        ),
        tonic::Code::NotFound
    );
    assert!(
        !catalog
            .drop_graph_type(DEFAULT_SCHEMA, "Social", true)
            .await
            .unwrap()
    );
    assert_eq!(
        grpc_code(
            catalog
                .create_graph_type(DEFAULT_SCHEMA, "", false, false)
                .await
        ),
        tonic::Code::InvalidArgument
    );
}

/// gwp#15: `CREATE GRAPH x IF NOT EXISTS` over a remote session reported an
/// error. The protocol passes DDL statements to the backend verbatim and
/// reports the backend's outcome unchanged; IF NOT EXISTS is idempotent end
/// to end when the backend implements it.
#[tokio::test]
async fn create_graph_if_not_exists_statement_is_idempotent() {
    let server = common::start().await;
    let conn = GqlConnection::connect(&server.endpoint()).await.unwrap();
    let mut session = conn.create_session().await.unwrap();

    // On a missing graph: success, and the graph exists.
    let mut cursor = session
        .execute_simple("CREATE GRAPH IF NOT EXISTS reports")
        .await
        .unwrap();
    assert!(cursor.collect_rows().await.unwrap().is_empty());
    assert!(cursor.is_success().await.unwrap());
    let summary = cursor.summary().await.unwrap().unwrap().clone();
    assert_eq!(summary.status.unwrap().code, status::OMITTED_RESULT);
    assert!(server.backend.has_graph(DEFAULT_SCHEMA, "reports"));

    // On an existing graph: success again, no-op.
    let mut cursor = session
        .execute_simple("CREATE GRAPH IF NOT EXISTS reports")
        .await
        .unwrap();
    assert!(cursor.is_success().await.unwrap());

    // Without IF NOT EXISTS on an existing graph: the backend's error.
    let mut cursor = session
        .execute_simple("CREATE GRAPH reports")
        .await
        .unwrap();
    assert!(!cursor.is_success().await.unwrap());
    let summary = cursor.summary().await.unwrap().unwrap().clone();
    assert_eq!(summary.status.unwrap().code, status::DUPLICATE_DEFINITION);

    // The form from the issue (IF NOT EXISTS after the name) is not GQL
    // syntax: a backend rejects it as a whole, and the protocol relays that.
    let mut cursor = session
        .execute_simple("CREATE GRAPH other IF NOT EXISTS")
        .await
        .unwrap();
    assert!(!cursor.is_success().await.unwrap());
    let summary = cursor.summary().await.unwrap().unwrap().clone();
    assert_eq!(summary.status.unwrap().code, status::INVALID_SYNTAX);
    assert!(!server.backend.has_graph(DEFAULT_SCHEMA, "other"));

    // DROP GRAPH IF EXISTS mirrors it.
    let mut cursor = session
        .execute_simple("DROP GRAPH IF EXISTS reports")
        .await
        .unwrap();
    assert!(cursor.is_success().await.unwrap());
    let mut cursor = session
        .execute_simple("DROP GRAPH IF EXISTS reports")
        .await
        .unwrap();
    assert!(cursor.is_success().await.unwrap());
    let mut cursor = session.execute_simple("DROP GRAPH reports").await.unwrap();
    assert!(!cursor.is_success().await.unwrap());

    // Every statement reached the backend exactly once and unchanged.
    let session_id = session.session_id().to_owned();
    let statements: Vec<String> = server
        .backend
        .events_with("execute")
        .into_iter()
        .map(|e| {
            e.strip_prefix(&format!("execute {session_id} tx=- "))
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(
        statements,
        vec![
            "CREATE GRAPH IF NOT EXISTS reports",
            "CREATE GRAPH IF NOT EXISTS reports",
            "CREATE GRAPH reports",
            "CREATE GRAPH other IF NOT EXISTS",
            "DROP GRAPH IF EXISTS reports",
            "DROP GRAPH IF EXISTS reports",
            "DROP GRAPH reports",
        ]
    );
}

#[tokio::test]
async fn catalog_rpcs_need_no_session() {
    // The catalog service is session-less: it works before any handshake.
    let (server, mut catalog) = catalog().await;
    catalog.create_schema("s", true).await.unwrap();
    assert!(
        server
            .backend
            .events()
            .iter()
            .all(|e| !e.starts_with("create session"))
    );
}

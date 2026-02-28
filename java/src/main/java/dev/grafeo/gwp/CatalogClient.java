package dev.grafeo.gwp;

import gql.GqlServiceOuterClass;
import gql.GqlServiceOuterClass.GraphOptions;
import gql.GqlServiceOuterClass.GraphSummary;
import gql.GqlServiceOuterClass.GetGraphInfoResponse;
import gql.GqlServiceOuterClass.SchemaInfo;
import gql.GqlServiceOuterClass.GraphTypeInfo;
import io.grpc.ManagedChannel;

import java.util.List;

/**
 * A client for managing schemas, graphs, and graph types on a GWP server.
 *
 * <p>Replaces {@code DatabaseClient} from 0.1.5. Uses the CatalogService
 * which follows the GQL spec catalog &gt; schema &gt; graph hierarchy.</p>
 */
public class CatalogClient implements AutoCloseable {

    private final gql.CatalogServiceGrpc.CatalogServiceBlockingStub stub;

    CatalogClient(ManagedChannel channel) {
        this.stub = gql.CatalogServiceGrpc.newBlockingStub(channel);
    }

    // ========================================================================
    // Domain types
    // ========================================================================

    /**
     * Summary information about a schema.
     */
    public record SchemaDetails(
            String name,
            int graphCount,
            int graphTypeCount
    ) {
        static SchemaDetails fromProto(SchemaInfo s) {
            return new SchemaDetails(
                    s.getName(),
                    s.getGraphCount(),
                    s.getGraphTypeCount()
            );
        }
    }

    /**
     * Summary information about a graph.
     */
    public record GraphInfo(
            String schema,
            String name,
            long nodeCount,
            long edgeCount,
            String graphType,
            String storageMode,
            Long memoryLimitBytes,
            Boolean backwardEdges,
            Integer threads
    ) {
        static GraphInfo fromSummary(GraphSummary s) {
            return new GraphInfo(
                    s.getSchema(),
                    s.getName(),
                    s.getNodeCount(),
                    s.getEdgeCount(),
                    s.getGraphType(),
                    "",
                    null,
                    null,
                    null
            );
        }

        static GraphInfo fromGetResponse(GetGraphInfoResponse r) {
            return new GraphInfo(
                    r.getSchema(),
                    r.getName(),
                    r.getNodeCount(),
                    r.getEdgeCount(),
                    r.getGraphType(),
                    r.getStorageMode(),
                    r.getMemoryLimitBytes() > 0 ? r.getMemoryLimitBytes() : null,
                    r.getBackwardEdges() ? true : null,
                    r.getThreads() > 0 ? (int) r.getThreads() : null
            );
        }
    }

    /**
     * Summary information about a graph type.
     */
    public record GraphTypeDetails(
            String schema,
            String name
    ) {
        static GraphTypeDetails fromProto(GraphTypeInfo t) {
            return new GraphTypeDetails(t.getSchema(), t.getName());
        }
    }

    /**
     * Configuration for creating a new graph.
     */
    public record CreateGraphConfig(
            String schema,
            String name,
            boolean ifNotExists,
            boolean orReplace,
            String storageMode,
            Long memoryLimitBytes,
            Boolean backwardEdges,
            Integer threads,
            Boolean walEnabled,
            String walDurability
    ) {
        public CreateGraphConfig(String schema, String name) {
            this(schema, name, false, false, "InMemory", null, null, null, null, null);
        }

        public CreateGraphConfig(String schema, String name, String storageMode) {
            this(schema, name, false, false, storageMode, null, null, null, null, null);
        }
    }

    // ========================================================================
    // Schema operations
    // ========================================================================

    /**
     * List all schemas on the server.
     */
    public List<SchemaDetails> listSchemas() {
        var resp = stub.listSchemas(GqlServiceOuterClass.ListSchemasRequest.getDefaultInstance());
        return resp.getSchemasList().stream()
                .map(SchemaDetails::fromProto)
                .toList();
    }

    /**
     * Create a new schema.
     */
    public void createSchema(String name, boolean ifNotExists) {
        stub.createSchema(GqlServiceOuterClass.CreateSchemaRequest.newBuilder()
                .setName(name)
                .setIfNotExists(ifNotExists)
                .build());
    }

    /**
     * Drop a schema.
     *
     * @return true if the schema existed and was dropped
     */
    public boolean dropSchema(String name, boolean ifExists) {
        var resp = stub.dropSchema(GqlServiceOuterClass.DropSchemaRequest.newBuilder()
                .setName(name)
                .setIfExists(ifExists)
                .build());
        return resp.getExisted();
    }

    // ========================================================================
    // Graph operations
    // ========================================================================

    /**
     * List all graphs in a schema.
     */
    public List<GraphInfo> listGraphs(String schema) {
        var resp = stub.listGraphs(GqlServiceOuterClass.ListGraphsRequest.newBuilder()
                .setSchema(schema)
                .build());
        return resp.getGraphsList().stream()
                .map(GraphInfo::fromSummary)
                .toList();
    }

    /**
     * Create a new graph.
     */
    public GraphInfo createGraph(CreateGraphConfig config) {
        var optionsBuilder = GraphOptions.newBuilder();
        if (config.memoryLimitBytes() != null) {
            optionsBuilder.setMemoryLimitBytes(config.memoryLimitBytes());
        }
        if (config.backwardEdges() != null) {
            optionsBuilder.setBackwardEdges(config.backwardEdges());
        }
        if (config.threads() != null) {
            optionsBuilder.setThreads(config.threads());
        }
        if (config.walEnabled() != null) {
            optionsBuilder.setWalEnabled(config.walEnabled());
        }
        if (config.walDurability() != null) {
            optionsBuilder.setWalDurability(config.walDurability());
        }

        var resp = stub.createGraph(GqlServiceOuterClass.CreateGraphRequest.newBuilder()
                .setSchema(config.schema())
                .setName(config.name())
                .setIfNotExists(config.ifNotExists())
                .setOrReplace(config.orReplace())
                .setStorageMode(config.storageMode())
                .setOptions(optionsBuilder.build())
                .build());

        return GraphInfo.fromSummary(resp.getGraph());
    }

    /**
     * Drop a graph.
     *
     * @return true if the graph existed and was dropped
     */
    public boolean dropGraph(String schema, String name, boolean ifExists) {
        var resp = stub.dropGraph(GqlServiceOuterClass.DropGraphRequest.newBuilder()
                .setSchema(schema)
                .setName(name)
                .setIfExists(ifExists)
                .build());
        return resp.getExisted();
    }

    /**
     * Get detailed information about a specific graph.
     */
    public GraphInfo getGraphInfo(String schema, String name) {
        var resp = stub.getGraphInfo(GqlServiceOuterClass.GetGraphInfoRequest.newBuilder()
                .setSchema(schema)
                .setName(name)
                .build());
        return GraphInfo.fromGetResponse(resp);
    }

    // ========================================================================
    // Graph type operations
    // ========================================================================

    /**
     * List all graph types in a schema.
     */
    public List<GraphTypeDetails> listGraphTypes(String schema) {
        var resp = stub.listGraphTypes(GqlServiceOuterClass.ListGraphTypesRequest.newBuilder()
                .setSchema(schema)
                .build());
        return resp.getGraphTypesList().stream()
                .map(GraphTypeDetails::fromProto)
                .toList();
    }

    /**
     * Create a new graph type.
     */
    public void createGraphType(String schema, String name, boolean ifNotExists, boolean orReplace) {
        stub.createGraphType(GqlServiceOuterClass.CreateGraphTypeRequest.newBuilder()
                .setSchema(schema)
                .setName(name)
                .setIfNotExists(ifNotExists)
                .setOrReplace(orReplace)
                .build());
    }

    /**
     * Drop a graph type.
     *
     * @return true if the graph type existed and was dropped
     */
    public boolean dropGraphType(String schema, String name, boolean ifExists) {
        var resp = stub.dropGraphType(GqlServiceOuterClass.DropGraphTypeRequest.newBuilder()
                .setSchema(schema)
                .setName(name)
                .setIfExists(ifExists)
                .build());
        return resp.getExisted();
    }

    @Override
    public void close() {
        // Stub doesn't own the channel, so nothing to close.
    }
}

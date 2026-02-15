package dev.grafeo.gwp;

import gql.GqlServiceOuterClass;
import gql.GqlServiceOuterClass.DatabaseOptions;
import gql.GqlServiceOuterClass.DatabaseSummary;
import gql.GqlServiceOuterClass.GetDatabaseInfoResponse;
import io.grpc.ManagedChannel;

import java.util.List;

/**
 * A client for managing databases on a GWP server.
 */
public class DatabaseClient implements AutoCloseable {

    private final gql.DatabaseServiceGrpc.DatabaseServiceBlockingStub stub;

    DatabaseClient(ManagedChannel channel) {
        this.stub = gql.DatabaseServiceGrpc.newBlockingStub(channel);
    }

    /**
     * Summary information about a database.
     */
    public record DatabaseInfo(
            String name,
            long nodeCount,
            long edgeCount,
            boolean persistent,
            String databaseType,
            String storageMode,
            Long memoryLimitBytes,
            Boolean backwardEdges,
            Integer threads
    ) {
        static DatabaseInfo fromSummary(DatabaseSummary s) {
            return new DatabaseInfo(
                    s.getName(),
                    s.getNodeCount(),
                    s.getEdgeCount(),
                    s.getPersistent(),
                    s.getDatabaseType(),
                    "",
                    null,
                    null,
                    null
            );
        }

        static DatabaseInfo fromGetResponse(GetDatabaseInfoResponse r) {
            return new DatabaseInfo(
                    r.getName(),
                    r.getNodeCount(),
                    r.getEdgeCount(),
                    r.getPersistent(),
                    r.getDatabaseType(),
                    r.getStorageMode(),
                    r.getMemoryLimitBytes() > 0 ? r.getMemoryLimitBytes() : null,
                    r.getBackwardEdges() ? true : null,
                    r.getThreads() > 0 ? (int) r.getThreads() : null
            );
        }
    }

    /**
     * Configuration for creating a new database.
     */
    public record CreateDatabaseConfig(
            String name,
            String databaseType,
            String storageMode,
            Long memoryLimitBytes,
            Boolean backwardEdges,
            Integer threads,
            Boolean walEnabled,
            String walDurability
    ) {
        public CreateDatabaseConfig(String name) {
            this(name, "Lpg", "InMemory", null, null, null, null, null);
        }

        public CreateDatabaseConfig(String name, String databaseType, String storageMode) {
            this(name, databaseType, storageMode, null, null, null, null, null);
        }
    }

    /**
     * List all databases on the server.
     */
    public List<DatabaseInfo> list() {
        var resp = stub.listDatabases(GqlServiceOuterClass.ListDatabasesRequest.getDefaultInstance());
        return resp.getDatabasesList().stream()
                .map(DatabaseInfo::fromSummary)
                .toList();
    }

    /**
     * Create a new database.
     */
    public DatabaseInfo create(CreateDatabaseConfig config) {
        var optionsBuilder = DatabaseOptions.newBuilder();
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

        var resp = stub.createDatabase(GqlServiceOuterClass.CreateDatabaseRequest.newBuilder()
                .setName(config.name())
                .setDatabaseType(config.databaseType())
                .setStorageMode(config.storageMode())
                .setOptions(optionsBuilder.build())
                .build());

        return DatabaseInfo.fromSummary(resp.getDatabase());
    }

    /**
     * Delete a database by name. Returns the name of the deleted database.
     */
    public String delete(String name) {
        var resp = stub.deleteDatabase(GqlServiceOuterClass.DeleteDatabaseRequest.newBuilder()
                .setName(name)
                .build());
        return resp.getDeleted();
    }

    /**
     * Get detailed information about a specific database.
     */
    public DatabaseInfo getInfo(String name) {
        var resp = stub.getDatabaseInfo(GqlServiceOuterClass.GetDatabaseInfoRequest.newBuilder()
                .setName(name)
                .build());
        return DatabaseInfo.fromGetResponse(resp);
    }

    @Override
    public void close() {
        // Stub doesn't own the channel, so nothing to close.
    }
}

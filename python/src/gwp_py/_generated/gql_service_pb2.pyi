import gql_types_pb2 as _gql_types_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ResetTarget(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    RESET_ALL: _ClassVar[ResetTarget]
    RESET_SCHEMA: _ClassVar[ResetTarget]
    RESET_GRAPH: _ClassVar[ResetTarget]
    RESET_TIME_ZONE: _ClassVar[ResetTarget]
    RESET_PARAMETERS: _ClassVar[ResetTarget]

class ResultType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    BINDING_TABLE: _ClassVar[ResultType]
    GRAPH: _ClassVar[ResultType]
    OMITTED: _ClassVar[ResultType]

class TransactionMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    READ_WRITE: _ClassVar[TransactionMode]
    READ_ONLY: _ClassVar[TransactionMode]
RESET_ALL: ResetTarget
RESET_SCHEMA: ResetTarget
RESET_GRAPH: ResetTarget
RESET_TIME_ZONE: ResetTarget
RESET_PARAMETERS: ResetTarget
BINDING_TABLE: ResultType
GRAPH: ResultType
OMITTED: ResultType
READ_WRITE: TransactionMode
READ_ONLY: TransactionMode

class HandshakeRequest(_message.Message):
    __slots__ = ("protocol_version", "credentials", "client_info")
    class ClientInfoEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    PROTOCOL_VERSION_FIELD_NUMBER: _ClassVar[int]
    CREDENTIALS_FIELD_NUMBER: _ClassVar[int]
    CLIENT_INFO_FIELD_NUMBER: _ClassVar[int]
    protocol_version: int
    credentials: _gql_types_pb2.AuthCredentials
    client_info: _containers.ScalarMap[str, str]
    def __init__(self, protocol_version: _Optional[int] = ..., credentials: _Optional[_Union[_gql_types_pb2.AuthCredentials, _Mapping]] = ..., client_info: _Optional[_Mapping[str, str]] = ...) -> None: ...

class HandshakeResponse(_message.Message):
    __slots__ = ("protocol_version", "session_id", "server_info", "limits")
    class LimitsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: int
        def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...
    PROTOCOL_VERSION_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    SERVER_INFO_FIELD_NUMBER: _ClassVar[int]
    LIMITS_FIELD_NUMBER: _ClassVar[int]
    protocol_version: int
    session_id: str
    server_info: ServerInfo
    limits: _containers.ScalarMap[str, int]
    def __init__(self, protocol_version: _Optional[int] = ..., session_id: _Optional[str] = ..., server_info: _Optional[_Union[ServerInfo, _Mapping]] = ..., limits: _Optional[_Mapping[str, int]] = ...) -> None: ...

class ServerInfo(_message.Message):
    __slots__ = ("name", "version", "features")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VERSION_FIELD_NUMBER: _ClassVar[int]
    FEATURES_FIELD_NUMBER: _ClassVar[int]
    name: str
    version: str
    features: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, name: _Optional[str] = ..., version: _Optional[str] = ..., features: _Optional[_Iterable[str]] = ...) -> None: ...

class ConfigureRequest(_message.Message):
    __slots__ = ("session_id", "schema", "graph", "time_zone_offset_minutes", "parameter")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    GRAPH_FIELD_NUMBER: _ClassVar[int]
    TIME_ZONE_OFFSET_MINUTES_FIELD_NUMBER: _ClassVar[int]
    PARAMETER_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    schema: str
    graph: str
    time_zone_offset_minutes: int
    parameter: SessionParameter
    def __init__(self, session_id: _Optional[str] = ..., schema: _Optional[str] = ..., graph: _Optional[str] = ..., time_zone_offset_minutes: _Optional[int] = ..., parameter: _Optional[_Union[SessionParameter, _Mapping]] = ...) -> None: ...

class SessionParameter(_message.Message):
    __slots__ = ("name", "value")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    name: str
    value: _gql_types_pb2.Value
    def __init__(self, name: _Optional[str] = ..., value: _Optional[_Union[_gql_types_pb2.Value, _Mapping]] = ...) -> None: ...

class ConfigureResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ResetRequest(_message.Message):
    __slots__ = ("session_id", "target")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    target: ResetTarget
    def __init__(self, session_id: _Optional[str] = ..., target: _Optional[_Union[ResetTarget, str]] = ...) -> None: ...

class ResetResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class CloseRequest(_message.Message):
    __slots__ = ("session_id",)
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    def __init__(self, session_id: _Optional[str] = ...) -> None: ...

class CloseResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class PingRequest(_message.Message):
    __slots__ = ("session_id",)
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    def __init__(self, session_id: _Optional[str] = ...) -> None: ...

class PongResponse(_message.Message):
    __slots__ = ("timestamp",)
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    timestamp: int
    def __init__(self, timestamp: _Optional[int] = ...) -> None: ...

class ExecuteRequest(_message.Message):
    __slots__ = ("session_id", "statement", "parameters", "transaction_id")
    class ParametersEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: _gql_types_pb2.Value
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[_gql_types_pb2.Value, _Mapping]] = ...) -> None: ...
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    STATEMENT_FIELD_NUMBER: _ClassVar[int]
    PARAMETERS_FIELD_NUMBER: _ClassVar[int]
    TRANSACTION_ID_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    statement: str
    parameters: _containers.MessageMap[str, _gql_types_pb2.Value]
    transaction_id: str
    def __init__(self, session_id: _Optional[str] = ..., statement: _Optional[str] = ..., parameters: _Optional[_Mapping[str, _gql_types_pb2.Value]] = ..., transaction_id: _Optional[str] = ...) -> None: ...

class ExecuteResponse(_message.Message):
    __slots__ = ("header", "row_batch", "summary")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    ROW_BATCH_FIELD_NUMBER: _ClassVar[int]
    SUMMARY_FIELD_NUMBER: _ClassVar[int]
    header: ResultHeader
    row_batch: RowBatch
    summary: ResultSummary
    def __init__(self, header: _Optional[_Union[ResultHeader, _Mapping]] = ..., row_batch: _Optional[_Union[RowBatch, _Mapping]] = ..., summary: _Optional[_Union[ResultSummary, _Mapping]] = ...) -> None: ...

class ResultHeader(_message.Message):
    __slots__ = ("result_type", "columns")
    RESULT_TYPE_FIELD_NUMBER: _ClassVar[int]
    COLUMNS_FIELD_NUMBER: _ClassVar[int]
    result_type: ResultType
    columns: _containers.RepeatedCompositeFieldContainer[ColumnDescriptor]
    def __init__(self, result_type: _Optional[_Union[ResultType, str]] = ..., columns: _Optional[_Iterable[_Union[ColumnDescriptor, _Mapping]]] = ...) -> None: ...

class ColumnDescriptor(_message.Message):
    __slots__ = ("name", "type")
    NAME_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    name: str
    type: _gql_types_pb2.TypeDescriptor
    def __init__(self, name: _Optional[str] = ..., type: _Optional[_Union[_gql_types_pb2.TypeDescriptor, _Mapping]] = ...) -> None: ...

class RowBatch(_message.Message):
    __slots__ = ("rows",)
    ROWS_FIELD_NUMBER: _ClassVar[int]
    rows: _containers.RepeatedCompositeFieldContainer[Row]
    def __init__(self, rows: _Optional[_Iterable[_Union[Row, _Mapping]]] = ...) -> None: ...

class Row(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedCompositeFieldContainer[_gql_types_pb2.Value]
    def __init__(self, values: _Optional[_Iterable[_Union[_gql_types_pb2.Value, _Mapping]]] = ...) -> None: ...

class ResultSummary(_message.Message):
    __slots__ = ("status", "warnings", "rows_affected", "counters")
    class CountersEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: int
        def __init__(self, key: _Optional[str] = ..., value: _Optional[int] = ...) -> None: ...
    STATUS_FIELD_NUMBER: _ClassVar[int]
    WARNINGS_FIELD_NUMBER: _ClassVar[int]
    ROWS_AFFECTED_FIELD_NUMBER: _ClassVar[int]
    COUNTERS_FIELD_NUMBER: _ClassVar[int]
    status: _gql_types_pb2.GqlStatus
    warnings: _containers.RepeatedCompositeFieldContainer[_gql_types_pb2.GqlStatus]
    rows_affected: int
    counters: _containers.ScalarMap[str, int]
    def __init__(self, status: _Optional[_Union[_gql_types_pb2.GqlStatus, _Mapping]] = ..., warnings: _Optional[_Iterable[_Union[_gql_types_pb2.GqlStatus, _Mapping]]] = ..., rows_affected: _Optional[int] = ..., counters: _Optional[_Mapping[str, int]] = ...) -> None: ...

class BeginRequest(_message.Message):
    __slots__ = ("session_id", "mode")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    MODE_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    mode: TransactionMode
    def __init__(self, session_id: _Optional[str] = ..., mode: _Optional[_Union[TransactionMode, str]] = ...) -> None: ...

class BeginResponse(_message.Message):
    __slots__ = ("transaction_id", "status")
    TRANSACTION_ID_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    transaction_id: str
    status: _gql_types_pb2.GqlStatus
    def __init__(self, transaction_id: _Optional[str] = ..., status: _Optional[_Union[_gql_types_pb2.GqlStatus, _Mapping]] = ...) -> None: ...

class CommitRequest(_message.Message):
    __slots__ = ("session_id", "transaction_id")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    TRANSACTION_ID_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    transaction_id: str
    def __init__(self, session_id: _Optional[str] = ..., transaction_id: _Optional[str] = ...) -> None: ...

class CommitResponse(_message.Message):
    __slots__ = ("status",)
    STATUS_FIELD_NUMBER: _ClassVar[int]
    status: _gql_types_pb2.GqlStatus
    def __init__(self, status: _Optional[_Union[_gql_types_pb2.GqlStatus, _Mapping]] = ...) -> None: ...

class RollbackRequest(_message.Message):
    __slots__ = ("session_id", "transaction_id")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    TRANSACTION_ID_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    transaction_id: str
    def __init__(self, session_id: _Optional[str] = ..., transaction_id: _Optional[str] = ...) -> None: ...

class RollbackResponse(_message.Message):
    __slots__ = ("status",)
    STATUS_FIELD_NUMBER: _ClassVar[int]
    status: _gql_types_pb2.GqlStatus
    def __init__(self, status: _Optional[_Union[_gql_types_pb2.GqlStatus, _Mapping]] = ...) -> None: ...

class ListDatabasesRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class DatabaseSummary(_message.Message):
    __slots__ = ("name", "node_count", "edge_count", "persistent", "database_type")
    NAME_FIELD_NUMBER: _ClassVar[int]
    NODE_COUNT_FIELD_NUMBER: _ClassVar[int]
    EDGE_COUNT_FIELD_NUMBER: _ClassVar[int]
    PERSISTENT_FIELD_NUMBER: _ClassVar[int]
    DATABASE_TYPE_FIELD_NUMBER: _ClassVar[int]
    name: str
    node_count: int
    edge_count: int
    persistent: bool
    database_type: str
    def __init__(self, name: _Optional[str] = ..., node_count: _Optional[int] = ..., edge_count: _Optional[int] = ..., persistent: bool = ..., database_type: _Optional[str] = ...) -> None: ...

class ListDatabasesResponse(_message.Message):
    __slots__ = ("databases",)
    DATABASES_FIELD_NUMBER: _ClassVar[int]
    databases: _containers.RepeatedCompositeFieldContainer[DatabaseSummary]
    def __init__(self, databases: _Optional[_Iterable[_Union[DatabaseSummary, _Mapping]]] = ...) -> None: ...

class CreateDatabaseRequest(_message.Message):
    __slots__ = ("name", "database_type", "storage_mode", "options")
    NAME_FIELD_NUMBER: _ClassVar[int]
    DATABASE_TYPE_FIELD_NUMBER: _ClassVar[int]
    STORAGE_MODE_FIELD_NUMBER: _ClassVar[int]
    OPTIONS_FIELD_NUMBER: _ClassVar[int]
    name: str
    database_type: str
    storage_mode: str
    options: DatabaseOptions
    def __init__(self, name: _Optional[str] = ..., database_type: _Optional[str] = ..., storage_mode: _Optional[str] = ..., options: _Optional[_Union[DatabaseOptions, _Mapping]] = ...) -> None: ...

class DatabaseOptions(_message.Message):
    __slots__ = ("memory_limit_bytes", "backward_edges", "threads", "wal_enabled", "wal_durability")
    MEMORY_LIMIT_BYTES_FIELD_NUMBER: _ClassVar[int]
    BACKWARD_EDGES_FIELD_NUMBER: _ClassVar[int]
    THREADS_FIELD_NUMBER: _ClassVar[int]
    WAL_ENABLED_FIELD_NUMBER: _ClassVar[int]
    WAL_DURABILITY_FIELD_NUMBER: _ClassVar[int]
    memory_limit_bytes: int
    backward_edges: bool
    threads: int
    wal_enabled: bool
    wal_durability: str
    def __init__(self, memory_limit_bytes: _Optional[int] = ..., backward_edges: bool = ..., threads: _Optional[int] = ..., wal_enabled: bool = ..., wal_durability: _Optional[str] = ...) -> None: ...

class CreateDatabaseResponse(_message.Message):
    __slots__ = ("database",)
    DATABASE_FIELD_NUMBER: _ClassVar[int]
    database: DatabaseSummary
    def __init__(self, database: _Optional[_Union[DatabaseSummary, _Mapping]] = ...) -> None: ...

class DeleteDatabaseRequest(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class DeleteDatabaseResponse(_message.Message):
    __slots__ = ("deleted",)
    DELETED_FIELD_NUMBER: _ClassVar[int]
    deleted: str
    def __init__(self, deleted: _Optional[str] = ...) -> None: ...

class GetDatabaseInfoRequest(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class GetDatabaseInfoResponse(_message.Message):
    __slots__ = ("name", "node_count", "edge_count", "persistent", "database_type", "storage_mode", "memory_limit_bytes", "backward_edges", "threads")
    NAME_FIELD_NUMBER: _ClassVar[int]
    NODE_COUNT_FIELD_NUMBER: _ClassVar[int]
    EDGE_COUNT_FIELD_NUMBER: _ClassVar[int]
    PERSISTENT_FIELD_NUMBER: _ClassVar[int]
    DATABASE_TYPE_FIELD_NUMBER: _ClassVar[int]
    STORAGE_MODE_FIELD_NUMBER: _ClassVar[int]
    MEMORY_LIMIT_BYTES_FIELD_NUMBER: _ClassVar[int]
    BACKWARD_EDGES_FIELD_NUMBER: _ClassVar[int]
    THREADS_FIELD_NUMBER: _ClassVar[int]
    name: str
    node_count: int
    edge_count: int
    persistent: bool
    database_type: str
    storage_mode: str
    memory_limit_bytes: int
    backward_edges: bool
    threads: int
    def __init__(self, name: _Optional[str] = ..., node_count: _Optional[int] = ..., edge_count: _Optional[int] = ..., persistent: bool = ..., database_type: _Optional[str] = ..., storage_mode: _Optional[str] = ..., memory_limit_bytes: _Optional[int] = ..., backward_edges: bool = ..., threads: _Optional[int] = ...) -> None: ...

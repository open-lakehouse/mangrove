# olai-sharing-common

**Open Sharing** / **Delta Sharing** wire models, shared by client and server.

- The generated request/response types of the discovery services
  (`models::open_sharing::v1`), with snake_case JSON on the wire and camelCase
  accepted on read.
- The hand-written newline-delimited JSON lines of the table query path
  (`QueryResponseLine`, `MetadataResponse`, `ProtocolResponseData`,
  `ParquetFile`, …) and their conversions from `delta_kernel` actions.
- Parsing of the `delta-sharing-capabilities` header
  (`DeltaSharingCapabilities`).
- Behind the `axum` feature, the axum request extractors a server needs for
  these types.

[`olai-sharing-client`](../sharing-client) sends these types and
[`olai-sharing-api`](../sharing-api) serves them. Keeping them in their own crate
lets the server side build without the client's dependencies.

The generated code under `src/models/_gen/` comes from `proto/sharing` via
`just codegen generate-proto` + `just codegen generate-code-sharing`. Never edit
it by hand.

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.2](https://github.com/open-lakehouse/mangrove/compare/uc-app-v0.0.1...uc-app-v0.0.2) - 2026-10-31

### Added

- [**breaking**] per-resource features and clients as handlers (#261) (#268)
- restore the shares, recipients and providers models (#264)

### Documentation

- consolidate and correct agent instructions (#274)

## [0.0.1](https://github.com/open-lakehouse/mangrove/releases/tag/uc-app-v0.0.1) - 2026-10-30

### Added

- [**breaking**] align Unity Catalog protos with the Databricks API reference (#251)
- *(node)* publish-ready builds and DESIGN.md contracts for the UI packages (#209) (#216)
- *(node)* [**breaking**] publish the UC Node and TS clients to GitHub Packages (#208) (#214)
- *(release)* commit-driven Changesets pipeline for JS packages (dormant) (#183)
- *(query-wasm)* volume-files write path — in-wasm ConnectRPC transport + connect-es client (#162)
- folder drill-down, breadcrumb & download in the volume Files tab (#161)
- Delta-log visualizations — min/max boxes + rich action rows (#160)
- *(editor)* reusable Monaco editor package for the volumes integration (#149)
- volume files/browsing seam + read-only wasm backend (#148)
- in-browser wasm runner for the Delta log explorer (reconciled + actions) (#137)
- Delta-log explorer tab in Table detail (log-query seam scaffold) (#134)
- migrate from npm to bun (#82)
- add registered models + model versions API (#148) (#76)
- polish the catalog explorer — app shell, resizable sidebar, detail tabs, and console styling (#72)
- [**breaking**] migrate proto runtime from prost to buffa (supersedes #1) (#53)
- extract the UC Delta v1 API into the olai-uc-delta-api crate (#37)
- in-browser wasm query engine for table previews (#23) (#27)
- ABAC /policies endpoint (Databricks Policies API) (#22)
- query preview seam (#21)
- split server into its own CLI + bundle the web UI (headwaters parity) (#19)
- carve in shared UI packages + @open-lakehouse/unity-catalog (#17)
- Open Sharing Agent + AgentSkill resources (v0alpha1) (#227)
- track metric-view view_dependencies derived from the definition (#225)
- align managed storage paths with UC reference, separate from external (#217)
- require resolvable managed storage location for catalogs (#211)
- Open Sharing volume & agent-skill surfaces + generated sharing codegen (#119) (#204)
- first-class metric-view table type + view_definition (#196) (#198)
- add managed-table staging flow (staging-tables API + CreateTable) (#175)
- tag policy & entity-tag-assignment clients via generated aggregate (#169)
- Delta catalog-managed commits coordinator (server + Postgres) (#158) (#161)
- align enum wire format with UC API and validate API coverage (#141) (#150)
- extend Delta Sharing server with providers and side-by-side support (#31) (#146)
- migrate to buoyant_kernel and inline delta_kernel integration (#126)
- use shared codegen crate/cli (#117)
- napi-rs v3, code generation improvements, and proto annotation cleanup (#116)
- implement temporary credential vending with least-privilege downscoping (#115)
- simplify codegen and prepare for external use. (#103)
- Functions API surface (UDFs in Unity Catalog) (#85) (#97)
- dedicated error types for UC clients (#16) (#67)
- Enhance generated code with protobuf documentation and architecture improvements (#62)
- extend node client generation (#59)
- list builders with into_stream (#53)
- generate top level python client (#47)
- next gen codegen (#42)
- integration test framework (#36)
- use RequestBuilders + IntoFuture to simplify APIs (#33)
- volumes API (#27)
- cleaner server / client / common separation (#19)
- object store backed by unity vended credential (#18)
- improve client generation and implementation (#15)
- extend node client methods (#10)
- node client (#9)

### Changed

- [**breaking**] move Open Sharing and the shares/recipients/providers APIs to distributary (#257)
- *(wasm)* isolate query-wasm + olai-delta-df into a standalone wasm/ workspace (#182) (#187)
- *(just)* split the root justfile into task modules (#147)
- *(log-query)* drop vestigial fixed-logical-name SQL from the log seam (#139)
- *(acceptance)* [**breaking**] replace journey framework with an API-coverage conformance battery (#64)
- migrate trestle codegen to the nested co-located config (#3)
- generate Resource/ObjectLabel enums from google.api.resource annotations (#104)
- remove AxumPath/AxumQuery variants from common::Error (#71)
- remove Info suffix from resource types (#56)
- separate delta-sharing (#50)
- split out client crate (#25)

### Documentation

- *(node)* add UI architecture for state, seams and composition (#222)

### Fixed

- *(query-wasm)* stop doubled _delta_log paths in the wasm resolver (#157)
- *(query)* survive StrictMode remount so preview/log grids fill on first view (#146)
- *(data-grid)* flush virtualizer when the grid mounts after the stream ends (#145)
- defer wasm preview/log-query run start to mount effect (#144)
- *(models)* [**breaking**] flatten registered-model & model-version create bodies (#78)
- *(functions)* [**breaking**] wrap CreateFunction body in a function_info envelope (#75)
- pin internal path-dep versions on the publish=false glue crates (#61)
- unblock node CI unit tests and spell-check (#136)
- address generated code quality issues from proto-gen review (#107)
- emit correct Python class names and add docs with code examples (#63)
- node build (#39)

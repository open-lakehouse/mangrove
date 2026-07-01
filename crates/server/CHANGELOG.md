# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1](https://github.com/open-lakehouse/mangrove/releases/tag/olai-uc-server-v0.0.1) - 2026-07-01

### Added

- split server into its own CLI + bundle the web UI (headwaters parity) (#19)
- Open Sharing Agent + AgentSkill resources (v0alpha1) (#227)
- track metric-view view_dependencies derived from the definition (#225)
- *(managed)* support managed Delta tables on local file:// storage (#222)
- embedded SQLite backend, managed storage_root validation, and Azurite support (#218)
- align managed storage paths with UC reference, separate from external (#217)
- use GUID-based storage paths for managed volumes (#216)
- require resolvable managed storage location for catalogs (#211)
- support local file:// paths for object stores (#206)
- Open Sharing volume & agent-skill surfaces + generated sharing codegen (#119) (#204)
- first-class metric-view table type + view_definition (#196) (#198)
- delta v1 client surface (#186)
- catalog managed tables via DRC (#181)
- add the /delta/v1 Delta REST API (managed tables + commits) (#180)
- add managed-table staging flow (staging-tables API + CreateTable) (#175)
- tag policy & entity-tag-assignment clients via generated aggregate (#169)
- add Entity Tag Assignments API (tags as associations) (#167)
- add Tag Policies API for governed tag definitions (#166)
- Delta catalog-managed commits coordinator (server + Postgres) (#158) (#161)
- envelope-encrypt secrets at rest with KEK rotation and a credential-vending cache (#157)
- extend Delta Sharing server with providers and side-by-side support (#31) (#146)
- migrate to buoyant_kernel and inline delta_kernel integration (#126)
- use shared codegen crate/cli (#117)
- napi-rs v3, code generation improvements, and proto annotation cleanup (#116)
- implement temporary credential vending with least-privilege downscoping (#115)
- simplify codegen and prepare for external use. (#103)
- Functions API surface (UDFs in Unity Catalog) (#85) (#97)
- Enhance generated code with protobuf documentation and architecture improvements (#62)
- extend node client generation (#59)
- list builders with into_stream (#53)
- next gen codegen (#42)
- use RequestBuilders + IntoFuture to simplify APIs (#33)
- volumes API (#27)
- cleaner server / client / common separation (#19)
- object store backed by unity vended credential (#18)
- improve client generation and implementation (#15)
- kernel engine via uc object store registry

### Build

- *(deps)* bump to DataFusion 54 + delta-rs olai/main (#11)

### Changed

- move FromRequestParts impl out of api module (#137) (#143)
- move ResourceStore/SecretManager traits to common (#127) (#140)
- dedup error->IntoResponse mapping and finish AxumPath removal (#128) (#134)
- generate Resource/ObjectLabel enums from google.api.resource annotations (#104)
- remove AxumPath/AxumQuery variants from common::Error (#71)
- remove Info suffix from resource types (#56)
- separate delta-sharing (#50)
- split out client crate (#25)
- improve crate structure (#21)
- more server movement (#20)
- split server into separate crate (#17)

### Documentation

- drive UC datafusion example from query references (#171)
- fix codegen/crate drift and improve agent-friendliness (#132)

### Fixed

- silence dead-code clippy errors failing CI (#4)
- *(managed)* adopt staging table-id on create across all backends (#221)
- align object_label PG enum with Rust ObjectLabel (#168)
- correct authorization filtering and harden credential vending (#131)
- address generated code quality issues from proto-gen review (#107)
- emit correct Python class names and add docs with code examples (#63)

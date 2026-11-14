# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.9](https://github.com/open-lakehouse/mangrove/compare/olai-uc-delta-api-v0.0.8...olai-uc-delta-api-v0.0.9) - 2026-11-14

### Added

- Cedar policy over olai-auth's Authorizer and EXTERNAL USE (#336)
- [**breaking**] atomic apply_update, commit retries, authz order (#321)

### Fixed

- spec-conformant updateTable and loadTable schema round-trip (#320)

## [0.0.8](https://github.com/open-lakehouse/mangrove/compare/olai-uc-delta-api-v0.0.7...olai-uc-delta-api-v0.0.8) - 2026-11-13

### Fixed

- keep the reserved location on managed create (#311)

## [0.0.6](https://github.com/open-lakehouse/mangrove/compare/olai-uc-delta-api-v0.0.5...olai-uc-delta-api-v0.0.6) - 2026-10-30

### Changed

- [**breaking**] adopt trestle credential-vending crates (#247) (#255)

## [0.0.5](https://github.com/open-lakehouse/mangrove/compare/olai-uc-delta-api-v0.0.4...olai-uc-delta-api-v0.0.5) - 2026-07-26

### Changed

- *(wasm)* isolate query-wasm + olai-delta-df into a standalone wasm/ workspace (#182) (#187)

## [0.0.4](https://github.com/open-lakehouse/mangrove/compare/olai-uc-delta-api-v0.0.3...olai-uc-delta-api-v0.0.4) - 2026-07-26

### Added

- make olai-uc-client wasm-capable via dual transport (Azure-first) (#150)
- make ContextExtractor async for path-segment-derived contexts (#142) (#143)
- make the Delta router state-agnostic and host-composable (#135) (#140)

## [0.0.3](https://github.com/open-lakehouse/mangrove/compare/olai-uc-delta-api-v0.0.2...olai-uc-delta-api-v0.0.3) - 2026-07-16

### Changed

- *(acceptance)* [**breaking**] replace journey framework with an API-coverage conformance battery (#64)

## [0.0.2](https://github.com/open-lakehouse/mangrove/compare/olai-uc-delta-api-v0.0.1...olai-uc-delta-api-v0.0.2) - 2026-07-12

### Changed

- consolidate storage abstractions on olai-store 0.0.6 (#48)

### Documentation

- crate READMEs and rustdoc for delta-api, common, client, object-store (#45)

//! Hand-maintained module root for the generated models, compiled in place of
//! the trestle-emitted `_gen/mod.rs`.
//!
//! Each proto package's buffa output (`_gen/unitycatalog.<pkg>.<ver>.rs`) is
//! self-contained, so it is mounted behind a per-resource feature of the same
//! name. The trestle aggregates that reference every package (`labels`,
//! `pyo3_impls`) require `all-resources`, which `store` and `python` imply. The
//! axum extractors are per package and follow the same features (see
//! `generated_extractors.rs`).
//!
//! `_gen/mod.rs` keeps being regenerated and is the reference this module is
//! checked against: the drift test at the bottom fails when a regen adds a
//! package that is not wired up here.
#![allow(clippy::empty_docs)]
#![allow(clippy::useless_borrows_in_formatting)]
#![allow(non_camel_case_types)]
#![allow(clippy::derivable_impls)]
use std::collections::HashMap;

#[cfg(feature = "all-resources")]
#[path = "_gen/labels.rs"]
pub mod labels;
#[cfg(feature = "all-resources")]
pub use labels::{ObjectLabel, Resource};

#[cfg(feature = "python")]
#[allow(clippy::clone_on_copy)]
#[path = "_gen/pyo3_impls.rs"]
pub mod pyo3_impls;
#[cfg(feature = "python")]
pub use pyo3_impls::*;

#[cfg(feature = "axum")]
#[path = "generated_extractors.rs"]
pub mod extractors;

pub type PropertyMap = HashMap<String, serde_json::Value>;

#[cfg(feature = "agent_skills")]
pub mod agent_skills {
    pub mod v0alpha1 {
        include!("_gen/unitycatalog.agent_skills.v0alpha1.rs");
    }
}
#[cfg(feature = "agent_skills")]
pub use agent_skills::v0alpha1::AgentSkill;

#[cfg(feature = "agents")]
pub mod agents {
    pub mod v0alpha1 {
        include!("_gen/unitycatalog.agents.v0alpha1.rs");
    }
}
#[cfg(feature = "agents")]
pub use agents::v0alpha1::Agent;

#[cfg(feature = "catalogs")]
pub mod catalogs {
    pub mod v1 {
        include!("_gen/unitycatalog.catalogs.v1.rs");
    }
}
#[cfg(feature = "catalogs")]
pub use catalogs::v1::Catalog;

#[cfg(feature = "credentials")]
pub mod credentials {
    pub mod v1 {
        include!("_gen/unitycatalog.credentials.v1.rs");
    }
}
#[cfg(feature = "credentials")]
pub use credentials::v1::Credential;

#[cfg(feature = "external_locations")]
pub mod external_locations {
    pub mod v1 {
        include!("_gen/unitycatalog.external_locations.v1.rs");
    }
}
#[cfg(feature = "external_locations")]
pub use external_locations::v1::ExternalLocation;

#[cfg(feature = "functions")]
pub mod functions {
    pub mod v1 {
        include!("_gen/unitycatalog.functions.v1.rs");
    }
}
#[cfg(feature = "functions")]
pub use functions::v1::Function;

#[cfg(feature = "model_versions")]
pub mod model_versions {
    pub mod v1 {
        include!("_gen/unitycatalog.model_versions.v1.rs");
    }
}
#[cfg(feature = "model_versions")]
pub use model_versions::v1::ModelVersion;

#[cfg(feature = "policies")]
pub mod policies {
    pub mod v1 {
        include!("_gen/unitycatalog.policies.v1.rs");
    }
}
#[cfg(feature = "policies")]
pub use policies::v1::PolicyInfo;

#[cfg(feature = "providers")]
pub mod providers {
    pub mod v1 {
        include!("_gen/unitycatalog.providers.v1.rs");
    }
}
#[cfg(feature = "providers")]
pub use providers::v1::Provider;

#[cfg(feature = "recipients")]
pub mod recipients {
    pub mod v1 {
        include!("_gen/unitycatalog.recipients.v1.rs");
    }
}
#[cfg(feature = "recipients")]
pub use recipients::v1::Recipient;

#[cfg(feature = "registered_models")]
pub mod registered_models {
    pub mod v1 {
        include!("_gen/unitycatalog.registered_models.v1.rs");
    }
}
#[cfg(feature = "registered_models")]
pub use registered_models::v1::RegisteredModel;

#[cfg(feature = "schemas")]
pub mod schemas {
    pub mod v1 {
        include!("_gen/unitycatalog.schemas.v1.rs");
    }
}
#[cfg(feature = "schemas")]
pub use schemas::v1::Schema;

#[cfg(feature = "shares")]
pub mod shares {
    pub mod v1 {
        include!("_gen/unitycatalog.shares.v1.rs");
    }
}
#[cfg(feature = "shares")]
pub use shares::v1::Share;

#[cfg(feature = "staging_tables")]
pub mod staging_tables {
    pub mod v1 {
        include!("_gen/unitycatalog.staging_tables.v1.rs");
    }
}
#[cfg(feature = "staging_tables")]
pub use staging_tables::v1::StagingTable;

#[cfg(feature = "tables")]
pub mod tables {
    pub mod v1 {
        include!("_gen/unitycatalog.tables.v1.rs");
    }
}
#[cfg(feature = "tables")]
pub use tables::v1::{Column, Table};

#[cfg(feature = "tags")]
pub mod tags {
    pub mod v1 {
        include!("_gen/unitycatalog.tags.v1.rs");
    }
}
#[cfg(feature = "tags")]
pub use tags::v1::TagPolicy;

#[cfg(feature = "temporary_credentials")]
pub mod temporary_credentials {
    pub mod v1 {
        include!("_gen/unitycatalog.temporary_credentials.v1.rs");
    }
}

#[cfg(feature = "volumes")]
pub mod volumes {
    pub mod v1 {
        include!("_gen/unitycatalog.volumes.v1.rs");
    }
}
#[cfg(feature = "volumes")]
pub use volumes::v1::Volume;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    /// Collects the quoted targets of every `include!(...)` and the paths of
    /// every root `pub use <pkg>::<ver>::...` re-export, normalised so the
    /// trestle root (`./x.rs`) and this one (`_gen/x.rs`) compare equal.
    fn wiring(src: &str) -> (BTreeSet<String>, BTreeSet<String>) {
        let mut includes = BTreeSet::new();
        let mut reexports = BTreeSet::new();
        for line in src.lines().map(str::trim) {
            if let Some(rest) = line.strip_prefix("include!(\"") {
                let path = rest.trim_end_matches("\");");
                let file = path.rsplit('/').next().unwrap_or(path);
                includes.insert(file.to_string());
            } else if let Some(rest) = line.strip_prefix("pub use ") {
                let path = rest.trim_end_matches(';');
                // Only package re-exports (`pkg::ver::Type`), not aggregates.
                if path.matches("::").count() >= 2 {
                    reexports.extend(expand(path));
                }
            }
        }
        (includes, reexports)
    }

    /// Expands `a::b::{C, D}` into `a::b::C` and `a::b::D`.
    fn expand(path: &str) -> Vec<String> {
        match path.split_once('{') {
            Some((prefix, items)) => items
                .trim_end_matches('}')
                .split(',')
                .map(|item| format!("{prefix}{}", item.trim()))
                .collect(),
            None => vec![path.to_string()],
        }
    }

    #[test]
    fn test_wiring_matches_trestle_root() {
        let (gen_includes, gen_reexports) = wiring(include_str!("_gen/mod.rs"));
        let (our_includes, our_reexports) = wiring(include_str!("generated.rs"));
        assert!(!gen_includes.is_empty(), "no packages found in _gen/mod.rs");
        assert_eq!(
            gen_includes, our_includes,
            "generated.rs must include exactly the packages in the trestle-emitted _gen/mod.rs"
        );
        assert_eq!(
            gen_reexports, our_reexports,
            "generated.rs must re-export exactly the root types of _gen/mod.rs"
        );
    }

    #[test]
    fn test_extractor_wiring_matches_trestle_root() {
        let modules = |src: &str| -> BTreeSet<String> {
            src.lines()
                .filter_map(|line| line.trim().strip_prefix("pub mod "))
                .map(|name| name.trim_end_matches(';').to_string())
                .collect()
        };
        let gen_modules = modules(include_str!("_gen/extractors/mod.rs"));
        assert!(
            !gen_modules.is_empty(),
            "no modules found in _gen/extractors/mod.rs"
        );
        assert_eq!(
            gen_modules,
            modules(include_str!("generated_extractors.rs")),
            "generated_extractors.rs must mount exactly the modules of _gen/extractors/mod.rs"
        );
    }
}

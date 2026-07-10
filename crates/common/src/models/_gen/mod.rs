// @generated — do not edit by hand.
#![allow(unexpected_cfgs)]
#![allow(clippy::empty_docs)]
use std::collections::HashMap;
pub mod labels;
pub use labels::{ObjectLabel, Resource};
#[cfg(feature = "python")]
pub mod pyo3_impls;
pub use agent_skills::v0alpha1::AgentSkill;
pub use agents::v0alpha1::Agent;
pub use catalogs::v1::Catalog;
pub use credentials::v1::Credential;
pub use external_locations::v1::ExternalLocation;
pub use functions::v1::Function;
pub use policies::v1::PolicyInfo;
pub use providers::v1::Provider;
#[cfg(feature = "python")]
pub use pyo3_impls::*;
pub use recipients::v1::Recipient;
pub use schemas::v1::Schema;
pub use shares::v1::Share;
pub use staging_tables::v1::StagingTable;
pub use tables::v1::Column;
pub use tables::v1::Table;
pub use tags::v1::TagPolicy;
pub use volumes::v1::Volume;
pub type PropertyMap = HashMap<String, serde_json::Value>;
pub mod agent_skills {
    pub mod v0alpha1 {
        include!("./unitycatalog.agent_skills.v0alpha1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.agent_skills.v0alpha1.tonic.rs");
    }
}
pub mod agents {
    pub mod v0alpha1 {
        include!("./unitycatalog.agents.v0alpha1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.agents.v0alpha1.tonic.rs");
    }
}
pub mod catalogs {
    pub mod v1 {
        include!("./unitycatalog.catalogs.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.catalogs.v1.tonic.rs");
    }
}
pub mod credentials {
    pub mod v1 {
        include!("./unitycatalog.credentials.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.credentials.v1.tonic.rs");
    }
}
pub mod external_locations {
    pub mod v1 {
        include!("./unitycatalog.external_locations.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.external_locations.v1.tonic.rs");
    }
}
pub mod functions {
    pub mod v1 {
        include!("./unitycatalog.functions.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.functions.v1.tonic.rs");
    }
}
pub mod policies {
    pub mod v1 {
        include!("./unitycatalog.policies.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.policies.v1.tonic.rs");
    }
}
pub mod providers {
    pub mod v1 {
        include!("./unitycatalog.providers.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.providers.v1.tonic.rs");
    }
}
pub mod recipients {
    pub mod v1 {
        include!("./unitycatalog.recipients.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.recipients.v1.tonic.rs");
    }
}
pub mod schemas {
    pub mod v1 {
        include!("./unitycatalog.schemas.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.schemas.v1.tonic.rs");
    }
}
pub mod shares {
    pub mod v1 {
        include!("./unitycatalog.shares.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.shares.v1.tonic.rs");
    }
}
pub mod staging_tables {
    pub mod v1 {
        include!("./unitycatalog.staging_tables.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.staging_tables.v1.tonic.rs");
    }
}
pub mod tables {
    pub mod v1 {
        include!("./unitycatalog.tables.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.tables.v1.tonic.rs");
    }
}
pub mod tags {
    pub mod v1 {
        include!("./unitycatalog.tags.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.tags.v1.tonic.rs");
    }
}
pub mod temporary_credentials {
    pub mod v1 {
        include!("./unitycatalog.temporary_credentials.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.temporary_credentials.v1.tonic.rs");
    }
}
pub mod volumes {
    pub mod v1 {
        include!("./unitycatalog.volumes.v1.rs");
        #[cfg(feature = "grpc")]
        include!("./unitycatalog.volumes.v1.tonic.rs");
    }
}
#[cfg(feature = "axum")]
pub mod extractors;

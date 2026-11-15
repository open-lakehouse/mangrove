// @generated — do not edit by hand.
//! [`Authorizable`] for the requests of this service.
#![allow(unused_imports)]

use crate::authz::{Authorizable, Operation, Securable};
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::models::volumes::v1::*;

impl Authorizable for ListVolumesRequest {
    fn operation(&self) -> Operation {
        Operation::ListVolumes
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Schema),
            name,
        }
    }
}

impl Authorizable for CreateVolumeRequest {
    fn operation(&self) -> Operation {
        Operation::CreateVolume
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.catalog_name.split('.').map(str::to_string));
        name.extend(self.schema_name.split('.').map(str::to_string));
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Volume),
            name,
        }
    }
}

impl Authorizable for GetVolumeRequest {
    fn operation(&self) -> Operation {
        Operation::GetVolume
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Volume),
            name,
        }
    }
}

impl Authorizable for UpdateVolumeRequest {
    fn operation(&self) -> Operation {
        Operation::UpdateVolume
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Volume),
            name,
        }
    }
}

impl Authorizable for DeleteVolumeRequest {
    fn operation(&self) -> Operation {
        Operation::DeleteVolume
    }

    fn securable(&self) -> Securable {
        let mut name = Vec::new();
        name.extend(self.name.split('.').map(str::to_string));
        Securable {
            label: Some(ObjectLabel::Volume),
            name,
        }
    }
}

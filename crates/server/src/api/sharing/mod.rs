use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};
use unitycatalog_sharing_client::models::open_sharing::v1::*;
pub use unitycatalog_sharing_client::models::*;

use crate::api::SecuredAction;
use crate::policy::Permission;

// The sharing handler traits + the NDJSON query handler now live in the portable
// `olai-uc-sharing-api` crate; re-exported here so existing `crate::api::sharing`
// references (routers, `build_rest_router` bounds) keep resolving. The server
// satisfies them all by implementing `SharingBackend` (see
// `crate::services::sharing_backend`).
pub use unitycatalog_sharing_api::{
    SharingHandler, SharingQueryHandler, SharingSkillHandler, SharingVolumeHandler,
};

impl SecuredAction for GetShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}
impl SecuredAction for ListSharesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}
impl SecuredAction for ListSchemasRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for ListAllTablesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for ListTablesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for QueryTableRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetTableVersionRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetTableMetadataRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

// Open Sharing asset requests (volumes, agent skills). All are scoped to the
// share and require read access; the concrete asset is authorized when its
// storage location is resolved.
macro_rules! sharing_read_on_share {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl SecuredAction for $ty {
                fn resource(&self) -> ResourceIdent {
                    ResourceIdent::share(ResourceName::new([self.share.as_str()]))
                }

                fn permission(&self) -> &'static Permission {
                    &Permission::Read
                }
            }
        )+
    };
}

sharing_read_on_share! {
    ListVolumesRequest,
    ListAllVolumesRequest,
    GetVolumeRequest,
    GenerateTemporaryVolumeCredentialsRequest,
    ListSkillsRequest,
    ListAllSkillsRequest,
    GetSkillRequest,
    GenerateTemporarySkillCredentialsRequest,
}

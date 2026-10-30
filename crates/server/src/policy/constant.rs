use unitycatalog_common::models::ResourceIdent;

use super::{Decision, Permission, Policy};
use crate::Result;

/// Policy that always returns a constant decision.
///
/// This policy is mainly useful for testing and development, or servers that do not require
/// authorization checks - e.g. when deployed in a trusted environment.
#[derive(Debug, Clone, Copy)]
pub struct ConstantPolicy {
    decision: Decision,
}

impl Default for ConstantPolicy {
    fn default() -> Self {
        Self {
            decision: Decision::Allow,
        }
    }
}

impl ConstantPolicy {
    /// Create a new instance of [`ConstantPolicy`].
    pub fn new(decision: Decision) -> Self {
        Self { decision }
    }
}

#[async_trait::async_trait]
impl<Cx: Send + Sync + 'static> Policy<Cx> for ConstantPolicy {
    async fn authorize(&self, _: &ResourceIdent, _: &Permission, _: &Cx) -> Result<Decision> {
        Ok(self.decision)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use unitycatalog_common::models::resource_name;

    #[test]
    fn assert_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ConstantPolicy>();
    }

    #[tokio::test]
    async fn allow_by_default() {
        let policy = ConstantPolicy::default();

        let resource = ResourceIdent::catalog(resource_name!("test_catalog"));
        let permission = Permission::Read;

        let decision = policy.authorize(&resource, &permission, &()).await.unwrap();
        assert_eq!(decision, Decision::Allow);
    }

    #[tokio::test]
    async fn allow() {
        let policy = ConstantPolicy::new(Decision::Allow);

        let resource = ResourceIdent::catalog(resource_name!("test_catalog"));
        let permission = Permission::Read;

        let decision = policy.authorize(&resource, &permission, &()).await.unwrap();
        assert_eq!(decision, Decision::Allow);
    }

    #[tokio::test]
    async fn deny() {
        let policy = ConstantPolicy::new(Decision::Deny);

        let resource = ResourceIdent::catalog(resource_name!("test_catalog"));
        let permission = Permission::Read;

        let decision = policy.authorize(&resource, &permission, &()).await.unwrap();
        assert_eq!(decision, Decision::Deny);
    }
}

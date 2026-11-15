use super::{Decision, Facts, Policy, Securable, ServerAction};
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
    async fn authorize(
        &self,
        _: ServerAction,
        _: &Securable,
        _: &Facts,
        _: &Cx,
    ) -> Result<Decision> {
        Ok(self.decision)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::policy::Operation;
    use unitycatalog_common::models::ObjectLabel;

    async fn decide(policy: ConstantPolicy) -> Decision {
        let catalog = Securable::new(ObjectLabel::Catalog, vec!["test_catalog".to_string()]);
        policy
            .authorize(
                Operation::GetCatalog.into(),
                &catalog,
                &Facts::default(),
                &(),
            )
            .await
            .unwrap()
    }

    #[test]
    fn assert_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ConstantPolicy>();
    }

    #[tokio::test]
    async fn allow_by_default() {
        assert_eq!(decide(ConstantPolicy::default()).await, Decision::Allow);
    }

    #[tokio::test]
    async fn allow() {
        assert_eq!(
            decide(ConstantPolicy::new(Decision::Allow)).await,
            Decision::Allow
        );
    }

    #[tokio::test]
    async fn deny() {
        assert_eq!(
            decide(ConstantPolicy::new(Decision::Deny)).await,
            Decision::Deny
        );
    }
}

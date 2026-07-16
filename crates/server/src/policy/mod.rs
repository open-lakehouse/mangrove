//! Authorization for the Unity Catalog server.
//!
//! Every request that touches a resource is gated by a [`Policy`]: given a
//! resource, the [`Permission`] the operation requires, and a caller-supplied
//! context, it returns a [`Decision`] of [`Allow`](Decision::Allow) or
//! [`Deny`](Decision::Deny). The trait is generic over the context type `Cx` so
//! the identity model is not baked in — a server can authorize on a
//! [`Principal`], a request-scoped struct, or `()` for an unauthenticated
//! allow-all deployment.
//!
//! Request types don't pass a bare resource/permission pair; they implement
//! [`SecuredAction`], which pins each operation to the exact permission it needs
//! at compile time. Handlers call [`Policy::check_required`] with the request
//! itself, so the required permission can never be mismatched by hand.
//!
//! # Provided implementations
//!
//! [`ConstantPolicy`] is the only implementation shipped today. It returns a
//! fixed decision regardless of input; [`ConstantPolicy::default`] allows
//! everything, which suits development and deployments where a trusted proxy
//! enforces access control upstream. Real per-resource RBAC is a future
//! implementation of this same trait.
//!
//! # Composing a policy into a handler
//!
//! A type that holds a policy behind an [`Arc`] implements [`ProvidesPolicy`],
//! and a blanket impl then makes that type act as a [`Policy`] by delegation, so
//! handlers can call [`check_required`](Policy::check_required) on `self`
//! directly. List endpoints use [`process_resources`] (or [`filter_authorized`]
//! for a `dyn`-typed policy) to drop entries the caller may not see.
//!
//! # Examples
//!
//! ```
//! use unitycatalog_server::policy::{ConstantPolicy, Decision, Permission, Policy};
//! use unitycatalog_common::models::{ResourceIdent, resource_name};
//!
//! # async fn run() -> unitycatalog_server::Result<()> {
//! // An allow-all policy authorized against the unit context.
//! let policy = ConstantPolicy::default();
//! let resource = ResourceIdent::catalog(resource_name!("main"));
//!
//! let decision = policy.authorize(&resource, &Permission::Read, &()).await?;
//! assert_eq!(decision, Decision::Allow);
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use strum::AsRefStr;
use unitycatalog_common::models::{ResourceExt, ResourceIdent};

pub use self::constant::*;
use crate::api::SecuredAction;
use crate::{Error, Result};

mod constant;

/// The identity a request is made on behalf of.
///
/// This is one possible authorization context (`Cx`) for a [`Policy`]; a server
/// that extracts a username from a reverse-proxy header would authorize against
/// a `Principal`.
#[derive(Clone, Debug)]
pub enum Principal {
    /// No authenticated identity — used when no credentials are present, e.g.
    /// behind a proxy that does not forward an identity header.
    Anonymous,
    /// A named user identity, typically the value of a forwarded-user header.
    User(String),
}

impl Principal {
    /// Returns an [`Anonymous`](Principal::Anonymous) principal.
    pub fn anonymous() -> Self {
        Self::Anonymous
    }

    /// Returns a [`User`](Principal::User) principal with the given name.
    pub fn user(name: impl Into<String>) -> Self {
        Self::User(name.into())
    }
}

/// The access level an operation requires on a resource.
///
/// Mirrors the Unity Catalog privilege model. A request type reports the
/// permission it needs through [`SecuredAction::permission`], so the mapping
/// from operation to permission is fixed rather than chosen at each call site.
///
/// Serialized as `snake_case`; parsing is ASCII-case-insensitive.
#[derive(Debug, Clone, AsRefStr, PartialEq, Eq, strum::EnumString)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum Permission {
    /// List and read a resource (e.g. `get`/`list` operations).
    Read,
    /// Modify data within an existing resource.
    Write,
    /// Update or delete a resource's metadata.
    Manage,
    /// Create new resources.
    Create,
    /// Use a resource as part of another operation, e.g. attaching a credential
    /// (mirrors `USE_CATALOG` / `USE_SCHEMA`).
    Use,
    /// Discover that a resource exists without reading its contents (`BROWSE`).
    Browse,
    /// Query data from a table or volume (`SELECT`).
    Select,
}

impl From<Permission> for String {
    fn from(val: Permission) -> Self {
        val.as_ref().to_string()
    }
}

/// Decision made by a policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Allow the action.
    Allow,
    /// Deny the action.
    Deny,
}

/// Access-control decision point for the server.
///
/// Implementations decide whether a caller (described by the context `Cx`) may
/// perform an operation on a resource. [`authorize`](Self::authorize) is the only
/// required method; every other method has a default that delegates to it. The
/// trait is `async` because real backends (external RBAC, database lookups) are
/// I/O bound.
///
/// `Cx` is the authorization context — the identity or request state the policy
/// evaluates against, e.g. [`Principal`] or `()`.
#[async_trait::async_trait]
pub trait Policy<Cx: Send + Sync + 'static>: Send + Sync + 'static {
    /// Authorizes a [`SecuredAction`], reading the resource and required
    /// permission from the action itself.
    ///
    /// Convenience over [`authorize`](Self::authorize) that removes the chance of
    /// passing a permission that doesn't match the operation.
    async fn check(&self, obj: &dyn SecuredAction, context: &Cx) -> Result<Decision> {
        self.authorize(&obj.resource(), obj.permission(), context)
            .await
    }

    /// Like [`check`](Self::check), but maps a [`Deny`](Decision::Deny) to
    /// [`Error::NotAllowed`] so a handler can propagate it with `?`.
    async fn check_required(&self, obj: &dyn SecuredAction, context: &Cx) -> Result<()> {
        match self.check(obj, context).await? {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(Error::NotAllowed),
        }
    }

    /// Decides whether `context` may exercise `permission` on `resource`.
    ///
    /// Returns [`Decision::Allow`] if the permission is granted and
    /// [`Decision::Deny`] otherwise. This is the one method an implementation
    /// must provide.
    async fn authorize(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<Decision>;

    /// Authorizes the same `permission` against many resources, returning one
    /// [`Decision`] per resource in the same order.
    ///
    /// The default calls [`authorize`](Self::authorize) sequentially;
    /// implementations backed by a batchable service should override it to issue
    /// a single bulk lookup. Used by [`process_resources`] to filter list
    /// results.
    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        let mut decisions = Vec::with_capacity(resources.len());
        for resource in resources {
            decisions.push(self.authorize(resource, permission, context).await?);
        }
        Ok(decisions)
    }

    /// Like [`authorize`](Self::authorize), but maps a [`Deny`](Decision::Deny)
    /// to [`Error::NotAllowed`].
    async fn authorize_checked(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<()> {
        match self.authorize(resource, permission, context).await? {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(Error::NotAllowed),
        }
    }
}

/// Types that own a [`Policy`] and want to act as one.
///
/// A blanket impl makes any `ProvidesPolicy<Cx>` implement `Policy<Cx>` by
/// delegating every method to [`policy`](Self::policy), so a composed handler
/// can call [`check_required`](Policy::check_required) on `self` without reaching
/// into the wrapped policy explicitly.
pub trait ProvidesPolicy<Cx: Send + Sync + 'static>: Send + Sync + 'static {
    /// Returns the policy this type delegates authorization to.
    fn policy(&self) -> &Arc<dyn Policy<Cx>>;
}

#[async_trait::async_trait]
impl<T: Policy<Cx>, Cx: Send + Sync + 'static> Policy<Cx> for Arc<T> {
    async fn authorize(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<Decision> {
        T::authorize(self, resource, permission, context).await
    }

    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        T::authorize_many(self, resources, permission, context).await
    }
}

/// Filters a `Vec` of resources in place, keeping only those the context is
/// allowed to access.
///
/// Batches the check through [`Policy::authorize_many`] and then retains each
/// resource whose decision is [`Allow`](Decision::Allow), preserving order. List
/// endpoints call this before returning results so callers never see resources
/// they lack `permission` on. For a policy held behind a trait object, use
/// [`filter_authorized`] instead.
pub async fn process_resources<
    T: Policy<Cx> + Sized,
    Cx: Send + Sync + 'static,
    R: ResourceExt + Send,
>(
    handler: &T,
    context: &Cx,
    permission: &Permission,
    resources: &mut Vec<R>,
) -> Result<()> {
    filter_authorized(handler, context, permission, resources).await
}

/// [`process_resources`] for a `dyn`-typed policy.
///
/// Identical filtering behavior, but takes `&dyn Policy<Cx>` so it can be used
/// with an `Arc<dyn Policy<Cx>>` (which does not satisfy the `Sized` bound on
/// [`process_resources`]). Handler patterns that hold the policy behind a trait
/// object — e.g. proxy/decorator handlers — use this.
pub async fn filter_authorized<Cx: Send + Sync + 'static, R: ResourceExt + Send>(
    policy: &dyn Policy<Cx>,
    context: &Cx,
    permission: &Permission,
    resources: &mut Vec<R>,
) -> Result<()> {
    let res = resources.iter().map(|r| r.into()).collect::<Vec<_>>();
    let decisions = policy.authorize_many(&res, permission, context).await?;
    // `decisions[i]` corresponds to `resources[i]`; pair them in forward order.
    let mut allow = decisions.into_iter().map(|d| d == Decision::Allow);
    resources.retain(|_| allow.next().unwrap_or(false));
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use unitycatalog_common::models::{ResourceName, ResourceRef, resource_name};

    /// Minimal resource carrying a share name, for exercising [`filter_authorized`].
    #[derive(Debug, Clone, PartialEq)]
    struct TestShare(&'static str);

    impl ResourceExt for TestShare {
        fn resource_name(&self) -> ResourceName {
            ResourceName::new([self.0])
        }
        fn resource_ref(&self) -> ResourceRef {
            ResourceRef::Name(self.resource_name())
        }
        fn resource_ident(&self) -> ResourceIdent {
            ResourceIdent::share(self.resource_name())
        }
    }

    /// Policy that allows only resources whose ident matches one of `allow`,
    /// and denies everything else — i.e. a non-uniform per-resource decision.
    struct AllowListPolicy {
        allow: Vec<ResourceIdent>,
    }

    #[async_trait::async_trait]
    impl Policy<()> for AllowListPolicy {
        async fn authorize(
            &self,
            resource: &ResourceIdent,
            _permission: &Permission,
            _context: &(),
        ) -> Result<Decision> {
            Ok(if self.allow.contains(resource) {
                Decision::Allow
            } else {
                Decision::Deny
            })
        }
    }

    /// Regression test for the reversed decision mapping bug: a non-uniform
    /// policy must filter the *correct* resources, in their original order.
    /// Before the fix, decisions were consumed back-to-front via `pop()`, so
    /// resource `i` was matched against decision `n-1-i`.
    #[tokio::test]
    async fn filter_authorized_pairs_decision_with_correct_resource() {
        let policy = AllowListPolicy {
            allow: vec![
                ResourceIdent::share(resource_name!("a")),
                ResourceIdent::share(resource_name!("c")),
            ],
        };

        // Asymmetric layout so a reversed mapping yields a different result:
        // allowed at indices 0 and 2, denied at 1 and 3.
        let mut resources = vec![
            TestShare("a"),
            TestShare("b"),
            TestShare("c"),
            TestShare("d"),
        ];

        filter_authorized(&policy, &(), &Permission::Read, &mut resources)
            .await
            .unwrap();

        assert_eq!(resources, vec![TestShare("a"), TestShare("c")]);
    }
}

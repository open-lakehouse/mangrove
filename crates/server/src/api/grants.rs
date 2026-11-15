//! The grants API: `GET` / `PATCH /permissions/{securable_type}/{full_name}`.
//!
//! Reads and writes the grants of a securable in the handler's grant store (see
//! [`crate::grants`]). Both are authorized on the securable itself: `GetGrants`
//! and `UpdateGrants`, which a policy typically allows its owner and holders of
//! `MANAGE`. Ownership is not a grant here: it shows as the securable's `owner`
//! and changes by updating the securable.

use std::collections::BTreeMap;

use olai_auth::grants::{Grant, GrantChange, Subject};
use unitycatalog_common::models::grants::v1::*;

use super::RequestContext;
use crate::grants::{OWNER, grant_error, grants_securable, oss_privilege, parse_privilege};
use crate::policy::{Facts, Operation, Policy, Securable, ident_of};
use crate::services::ServerHandler;
use crate::store::ResourceStoreReader;
use crate::{Error, Result};
use unitycatalog_common::models::ObjectLabel;
pub use unitycatalog_handlers::grants::GrantHandler;

impl ServerHandler<RequestContext> {
    /// Authorize `operation` on the securable a grants request names, and check
    /// that it exists.
    async fn grants_securable(
        &self,
        operation: Operation,
        securable_type: &str,
        full_name: &str,
        context: &RequestContext,
    ) -> Result<Securable> {
        let securable = grants_securable(securable_type, full_name)?;
        self.authorize_checked(operation.into(), &securable, &Facts::default(), context)
            .await?;
        // Credentials and external locations live in the storage-access store,
        // and the metastore always exists; the rest are in the resource store.
        let in_resource_store = !matches!(
            securable.label,
            None | Some(ObjectLabel::ExternalLocation | ObjectLabel::Credential)
        );
        if in_resource_store && let Some(ident) = ident_of(&securable) {
            self.get(&ident).await?;
        }
        Ok(securable)
    }

    /// The grants of `grants`, one assignment per principal, privileges in the
    /// OSS spelling and without ownership. Only `principal`'s, when given.
    async fn assignments(
        &self,
        grants: Vec<Grant>,
        principal: Option<&Subject>,
    ) -> Result<Vec<GrantAssignment>> {
        let mut by_subject: BTreeMap<Subject, Vec<String>> = BTreeMap::new();
        for grant in grants {
            if grant.privilege == OWNER || principal.is_some_and(|p| *p != grant.subject) {
                continue;
            }
            by_subject
                .entry(grant.subject)
                .or_default()
                .push(oss_privilege(&grant.privilege));
        }
        let mut assignments = Vec::with_capacity(by_subject.len());
        for (subject, privileges) in by_subject {
            assignments.push(GrantAssignment {
                principal: self.grants().principals().name(&subject).await?,
                privileges,
                ..Default::default()
            });
        }
        Ok(assignments)
    }
}

#[async_trait::async_trait]
impl GrantHandler<RequestContext> for ServerHandler<RequestContext> {
    #[tracing::instrument(skip(self, context))]
    async fn get_grants(
        &self,
        request: GetGrantsRequest,
        context: RequestContext,
    ) -> Result<GetGrantsResponse> {
        if request.max_results.is_some_and(|m| m < 0) {
            return Err(Error::invalid_argument("max_results must not be negative"));
        }
        let securable = self
            .grants_securable(
                Operation::GetGrants,
                &request.securable_type,
                &request.full_name,
                &context,
            )
            .await?;
        let object = crate::grants::object_of(&securable)?;
        let principal = match request.principal.as_deref().filter(|p| !p.is_empty()) {
            Some(principal) => Some(self.grants().principals().resolve(principal).await?),
            None => None,
        };
        let grants = self
            .grants()
            .store()
            .list(&object)
            .await
            .map_err(grant_error)?;
        // Every assignment fits one page: the grants of one securable are few.
        Ok(GetGrantsResponse {
            privilege_assignments: self.assignments(grants, principal.as_ref()).await?,
            next_page_token: None,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context))]
    async fn update_grants(
        &self,
        request: UpdateGrantsRequest,
        context: RequestContext,
    ) -> Result<UpdateGrantsResponse> {
        let securable = self
            .grants_securable(
                Operation::UpdateGrants,
                &request.securable_type,
                &request.full_name,
                &context,
            )
            .await?;
        let object = crate::grants::object_of(&securable)?;
        let mut changes = Vec::with_capacity(request.changes.len());
        for change in &request.changes {
            let subject = self
                .grants()
                .principals()
                .resolve(&change.principal)
                .await?;
            let parse = |privileges: &[String]| -> Result<Vec<String>> {
                privileges.iter().map(|p| parse_privilege(p)).collect()
            };
            changes.push(GrantChange {
                subject,
                add: parse(&change.add)?,
                remove: parse(&change.remove)?,
            });
        }
        let grants = self
            .grants()
            .store()
            .apply(&object, &changes)
            .await
            .map_err(grant_error)?;
        Ok(UpdateGrantsResponse {
            privilege_assignments: self.assignments(grants, None).await?,
            ..Default::default()
        })
    }
}

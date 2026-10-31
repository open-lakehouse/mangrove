// @generated — do not edit by hand.
#[cfg(feature = "agent_skills")]
pub mod agent_skills;
#[cfg(feature = "agents")]
pub mod agents;
#[cfg(feature = "catalogs")]
pub mod catalogs;
#[allow(clippy::too_many_arguments, clippy::doc_lazy_continuation)]
pub mod client;
#[cfg(feature = "credentials")]
pub mod credentials;
#[cfg(feature = "tags")]
pub mod entity_tag_assignments;
#[cfg(feature = "external_locations")]
pub mod external_locations;
#[cfg(feature = "functions")]
pub mod functions;
#[cfg(feature = "model_versions")]
pub mod model_versions;
#[cfg(feature = "policies")]
pub mod policies;
#[cfg(feature = "providers")]
pub mod providers;
#[cfg(feature = "recipients")]
pub mod recipients;
#[cfg(feature = "registered_models")]
pub mod registered_models;
#[cfg(feature = "schemas")]
pub mod schemas;
#[cfg(feature = "shares")]
pub mod shares;
#[cfg(feature = "staging_tables")]
pub mod staging_tables;
#[cfg(feature = "tables")]
pub mod tables;
#[cfg(feature = "tags")]
pub mod tag_policies;
#[cfg(feature = "temporary_credentials")]
pub mod temporary_credentials;
#[cfg(feature = "volumes")]
pub mod volumes;
#[allow(unused_imports)]
pub use client::*;
use futures::Future;
#[allow(dead_code)]
pub(super) fn stream_paginated<F, Fut, S, T>(
    state: S,
    op: F,
) -> impl futures::Stream<Item = crate::Result<T>>
where
    F: Fn(S, Option<String>) -> Fut + Copy,
    Fut: Future<Output = crate::Result<(T, S, Option<String>)>>,
{
    enum PaginationState<T> {
        Start(T),
        HasMore(T, String),
        Done,
    }
    futures::stream::unfold(PaginationState::Start(state), move |state| async move {
        let (s, page_token) = match state {
            PaginationState::Start(s) => (s, None),
            PaginationState::HasMore(s, page_token) if !page_token.is_empty() => {
                (s, Some(page_token))
            }
            _ => {
                return None;
            }
        };
        let (resp, s, continuation) = match op(s, page_token).await {
            Ok(resp) => resp,
            Err(e) => return Some((Err(e), PaginationState::Done)),
        };
        let next_state = match continuation {
            Some(token) => PaginationState::HasMore(s, token),
            None => PaginationState::Done,
        };
        Some((Ok(resp), next_state))
    })
}

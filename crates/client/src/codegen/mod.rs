// @generated — do not edit by hand.
pub mod agent_skills;
pub mod agents;
pub mod catalogs;
#[allow(clippy::too_many_arguments, clippy::doc_lazy_continuation)]
pub mod client;
pub mod credentials;
pub mod entity_tag_assignments;
pub mod external_locations;
pub mod functions;
pub mod model_versions;
pub mod policies;
pub mod providers;
pub mod recipients;
pub mod registered_models;
pub mod schemas;
pub mod shares;
pub mod staging_tables;
pub mod tables;
pub mod tag_policies;
pub mod temporary_credentials;
pub mod volumes;
#[allow(unused_imports)]
pub use client::*;
use futures::Future;
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

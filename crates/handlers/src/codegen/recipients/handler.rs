// @generated — do not edit by hand.
//! Handler trait for [`RecipientHandler`].
//!
//! Implement this trait to provide a custom backend for this service, then mount the
//! generated handler functions (in the sibling `server` module) onto an `axum::Router`
//! with your implementation as state.
//!
//! # Composability
//!
//! A single struct can implement multiple handler traits to serve multiple
//! services. Use [`axum::Router::merge`] to compose per-service routers together.
//!
//! Recipients
//!
//! A recipient is an object you create using recipients/create to represent an organization which
//! you want to allow access shares. when you create a recipient object, Unity Catalog generates an
//! activation link you can send to the recipient. The recipient follows the activation link to download
//! the credential file, and then uses the credential file to establish a secure connection to receive
//! the shared data. This sharing mode is called open sharing.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::recipients::v1::*;

#[async_trait]
pub trait RecipientHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// List recipients.
    async fn list_recipients(
        &self,
        request: ListRecipientsRequest,
        context: Cx,
    ) -> Result<ListRecipientsResponse>;
    /// Create a new recipient.
    async fn create_recipient(
        &self,
        request: CreateRecipientRequest,
        context: Cx,
    ) -> Result<Recipient>;
    /// Get a recipient by name.
    async fn get_recipient(&self, request: GetRecipientRequest, context: Cx) -> Result<Recipient>;
    /// Update a recipient.
    async fn update_recipient(
        &self,
        request: UpdateRecipientRequest,
        context: Cx,
    ) -> Result<Recipient>;
    /// Delete a recipient.
    async fn delete_recipient(&self, request: DeleteRecipientRequest, context: Cx) -> Result<()>;
    /// Rotate a recipient's token: issue a new token (and activation link) and
    /// retire the existing one.
    async fn rotate_recipient_token(
        &self,
        request: RotateRecipientTokenRequest,
        context: Cx,
    ) -> Result<Recipient>;
}

//! The in-memory store behaves like the SQL stores.

use unitycatalog_sharing::InMemorySharingStore;
use unitycatalog_sharing::testing::round_trips;

#[tokio::test]
async fn memory_store_round_trips() {
    round_trips(&InMemorySharingStore::new()).await;
}

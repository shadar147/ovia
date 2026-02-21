use async_trait::async_trait;
use uuid::Uuid;

use crate::identity::models::{
    BulkConfirmResult, ConflictQueueFilter, ConflictQueueStats, Identity, IdentityEvent,
    IdentityMappingFilter, Person, PersonIdentityLink,
};
use ovia_common::error::OviaResult;

#[async_trait]
pub trait PersonRepository: Send + Sync {
    async fn get_by_id(&self, org_id: Uuid, id: Uuid) -> OviaResult<Option<Person>>;
    async fn create(&self, person: Person) -> OviaResult<Person>;
    async fn update(&self, person: Person) -> OviaResult<Person>;
}

#[async_trait]
pub trait IdentityRepository: Send + Sync {
    async fn get_by_id(&self, org_id: Uuid, id: Uuid) -> OviaResult<Option<Identity>>;
    async fn create(&self, identity: Identity) -> OviaResult<Identity>;
    async fn update(&self, identity: Identity) -> OviaResult<Identity>;

    /// Insert or update an identity keyed by (org_id, source, external_id).
    /// On conflict, updates mutable fields (email, display_name, etc.) but preserves first_seen_at.
    async fn upsert_by_external_id(&self, identity: Identity) -> OviaResult<Identity>;
}

#[async_trait]
pub trait PersonIdentityLinkRepository: Send + Sync {
    async fn list_mappings(
        &self,
        org_id: Uuid,
        filter: IdentityMappingFilter,
    ) -> OviaResult<Vec<PersonIdentityLink>>;

    async fn confirm_mapping(
        &self,
        org_id: Uuid,
        link_id: Uuid,
        verified_by: &str,
    ) -> OviaResult<()>;

    async fn remap_mapping(
        &self,
        org_id: Uuid,
        link_id: Uuid,
        new_person_id: Uuid,
        verified_by: &str,
    ) -> OviaResult<()>;

    async fn split_mapping(&self, org_id: Uuid, link_id: Uuid, verified_by: &str)
        -> OviaResult<()>;

    async fn list_conflicts(
        &self,
        org_id: Uuid,
        filter: ConflictQueueFilter,
    ) -> OviaResult<Vec<PersonIdentityLink>>;

    async fn bulk_confirm_conflicts(
        &self,
        org_id: Uuid,
        link_ids: Vec<Uuid>,
        verified_by: &str,
    ) -> OviaResult<BulkConfirmResult>;

    async fn conflict_queue_stats(&self, org_id: Uuid) -> OviaResult<ConflictQueueStats>;
}

#[async_trait]
pub trait IdentityEventRepository: Send + Sync {
    async fn create(&self, event: IdentityEvent) -> OviaResult<IdentityEvent>;
    async fn list_by_link(&self, org_id: Uuid, link_id: Uuid) -> OviaResult<Vec<IdentityEvent>>;
}

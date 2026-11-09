use super::*;
choices!(FolderRole {
    Reference,
    Project,
    Meeting,
    Log,
    Other
});
value!(FolderRoleOverride {
    path: String,
    role: FolderRole
});
value!(InventoryDocument {
    source: SourceSummary,
    bytes: u64,
    title: String,
    linked: bool,
    role: FolderRole,
    role_reason: String
});
value!(InventoryFolder { path: String, role: FolderRole, reason: String, documents: u64, bytes: u64, dated: u64, undated: u64, linked: u64, date_from: Option<Date>, date_to: Option<Date> });
choices!(SetupEntry {
    Empty,
    New,
    Returning
});
value!(SetupInventoryRequest { root_id: Id, roles: Vec<FolderRoleOverride> });
value!(SetupInventory { schema: u32, root_id: Id, entry: SetupEntry, folders: Vec<InventoryFolder>, documents: Vec<InventoryDocument>, excluded: Vec<Exclusion>, eligible_bytes: u64, saved_topic_count: u64, resumable_job_ids: Vec<Id> });
value!(SetupLimits {
    max_files_per_batch: u32,
    max_bytes_per_batch: u64,
    max_batches: u32,
    max_candidates_per_turn: u32,
    max_turns_per_batch: u32
});
value!(SetupManifestPreviewRequest { root_id: Id, spec: ScopeSpec, roles: Vec<FolderRoleOverride>, limits: SetupLimits, runtime: String });
value!(SetupManifestSource {
    source: SourceSummary,
    bytes: u64,
    role: FolderRole
});
value!(SetupManifestBatch { batch_id: Id, sources: Vec<SetupManifestSource> });
value!(SetupManifest { schema: u32, manifest_id: Id, root_id: Id, fingerprint: Hash, spec: ScopeSpec, roles: Vec<FolderRoleOverride>, batches: Vec<SetupManifestBatch>, excluded: Vec<Exclusion>, limits: SetupLimits, runtime: String, created_at: Timestamp });

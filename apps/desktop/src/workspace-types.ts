// Mirrors topics_core::workspace and the committed semantic contract. No UI policy.
export type Id = string;
export type Sensitivity = 'unknown' | 'cloud_allowed' | 'local_only';
export type RootRef = { root_id: Id; path: string };
export type Locator = {
  start_byte: number;
  end_byte: number;
  heading: string | null;
};
export type SourceRef = {
  source_id: Id;
  path: string;
  document_hash: string;
  locator: Locator;
  passage_hash: string;
  excerpt: string;
};
export type SourceSummary = {
  source_id: Id;
  path: string;
  hash: string;
  date: string | null;
  sensitivity: Sensitivity;
};
export type ScopeSpec = {
  files: string[];
  folders: string[];
  date_from: string | null;
  date_to: string | null;
};
export type ScopePreview = {
  scope_id: Id;
  root_id: Id;
  sources: SourceSummary[];
  excluded: { path: string; reason: string }[];
};
export type Topic = {
  id: Id;
  topic_kind: 'person' | 'project' | 'thread';
  name: string;
  aliases: string[];
  sensitivity: Sensitivity;
};
export type Endpoint =
  | { kind: 'topic'; id: Id }
  | { kind: 'claim'; id: Id }
  | { kind: 'source'; source_id: Id; locator: Locator };
export type Evidence =
  | { kind: 'passage'; source: SourceRef }
  | { kind: 'user_statement'; statement_id: Id };
export type Claim = {
  id: Id;
  revision_id: Id;
  subject_id: Id;
  predicate: string;
  value: string;
  uncertainty: string | null;
  sources: Evidence[];
  supersedes: Id[];
  recorded_at: string;
  valid_from: string | null;
  valid_to: string | null;
};
export type Connection = {
  id: Id;
  from: Endpoint;
  relation: string;
  to: Endpoint;
  uncertainty: string | null;
  sources: SourceRef[];
  valid_from: string | null;
  valid_to: string | null;
};
export type Rule = {
  id: Id;
  topic_id: Id;
  predicate: string;
  match_value: string;
  replacement: string;
  scope: { source_paths: string[]; context: string | null };
  source_revision_id: Id;
};
export type Candidate =
  | { kind: 'topic'; topic: Topic }
  | { kind: 'claim'; claim: Claim }
  | { kind: 'connection'; connection: Connection }
  | { kind: 'correction'; claim: Claim; rule: Rule }
  | {
      kind: 'unresolved_identity';
      name: string;
      candidates: Id[];
      sources: SourceRef[];
    };
export type Proposal = {
  id: Id;
  root_id: Id;
  scope_id: Id | null;
  origin:
    | { kind: 'analysis'; run_id: Id }
    | { kind: 'chat'; topic_id: Id; message_id: Id }
    | { kind: 'manual' };
  status: 'pending' | 'accepted' | 'rejected' | 'conflict';
  candidates: Candidate[];
  fingerprint: string;
  created_at: string;
};
export type TopicView = {
  topic: Topic;
  current_claims: Claim[];
  connections: Connection[];
  unresolved: Candidate[];
  history: Claim[];
  warnings: string[];
};
export type Operation = {
  id: Id;
  status: string;
  description: string;
  created_at: string;
};
export type ApiError = {
  code: string;
  message: string;
  details: unknown | null;
};
export type RunEvent =
  | { kind: 'text_delta'; text: string }
  | { kind: 'proposal'; proposal_id: Id }
  | { kind: 'completed' | 'cancelled' }
  | { kind: 'error'; error: ApiError };
export type WorkspaceRunEvent = {
  root_id: Id;
  run_id: Id;
  seq: number;
  event: RunEvent;
};
export type RuntimeStatus = {
  available: boolean;
  version: string | null;
  message: string;
};
export type ScopedSource = {
  summary: SourceSummary;
  content: string;
  citation: SourceRef;
};

// Backend contract types — hand-maintained until OpenAPI generator is added.
// Keep in sync with backend response schemas.

export const BACKEND_CONTRACT_VERSION = "v0";

export interface Person {
  id: string;
  org_id: string;
  display_name: string;
  primary_email: string | null;
  team: string | null;
  role: string | null;
  status: string;
}

export interface Identity {
  id: string;
  org_id: string;
  source: string;
  external_id: string | null;
  username: string | null;
  email: string | null;
  display_name: string | null;
  is_service_account: boolean;
}

export type LinkStatus = "auto" | "verified" | "conflict" | "rejected" | "split";

export interface ScorerResult {
  rule: string;
  score: number;
  weight: number;
  weighted_score: number;
}

export interface RuleTrace {
  scorers: ScorerResult[];
  raw_total: number;
  weight_sum: number;
  confidence: number;
  classification: string;
}

export interface PersonIdentityLink {
  id: string;
  person_id: string;
  identity_id: string;
  status: LinkStatus;
  confidence: number;
  rule_trace: RuleTrace | null;
  verified_by: string | null;
  verified_at: string | null;
  created_at: string;
  updated_at: string;
  person: Person | null;
  identity: Identity | null;
}

export interface IdentityMappingFilter {
  status?: LinkStatus;
  min_confidence?: number;
  max_confidence?: number;
  limit?: number;
  offset?: number;
}

export interface ConflictQueueFilter {
  sort?: "confidence_asc" | "age_desc";
  min_confidence?: number;
  max_confidence?: number;
  limit?: number;
  offset?: number;
}

export interface ConflictQueueStats {
  total: number;
  by_status: Record<string, number>;
}

export interface BulkConfirmResult {
  confirmed: number;
  failed: number;
}

export interface KpiSnapshot {
  id: string;
  org_id: string;
  delivery_health: number;
  throughput: number;
  review_latency_p50: number;
  review_latency_p90: number;
  blocker_count: number;
  pipeline_success_rate: number;
  release_risk_label: string;
  release_risk_score: number;
  computed_at: string;
}

export interface RiskItem {
  id: string;
  snapshot_id: string;
  entity_type: string;
  title: string;
  owner: string | null;
  age_days: number;
  impact_scope: string | null;
  source_link: string | null;
}

export interface AskSession {
  id: string;
  org_id: string;
  query: string;
  answer: string;
  confidence: string;
  assumptions: string | null;
  citations: Citation[];
  filters: Record<string, string> | null;
  created_at: string;
}

export interface Citation {
  source: string;
  label: string;
  url: string | null;
  snippet: string | null;
}

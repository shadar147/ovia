use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiSnapshot {
    pub id: Uuid,
    pub org_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub delivery_health_score: Option<f64>,
    pub release_risk_score: Option<f64>,
    pub throughput_total: i32,
    pub throughput_bugs: i32,
    pub throughput_features: i32,
    pub throughput_chores: i32,
    pub review_latency_median_hours: Option<f64>,
    pub review_latency_p90_hours: Option<f64>,
    pub blocker_count: i32,
    pub spillover_rate: Option<f64>,
    pub cycle_time_p50_hours: Option<f64>,
    pub cycle_time_p90_hours: Option<f64>,
    pub computed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskItem {
    pub id: Uuid,
    pub org_id: Uuid,
    pub snapshot_id: Uuid,
    pub entity_type: String,
    pub title: String,
    pub owner: Option<String>,
    pub age_days: i32,
    pub impact_scope: Option<String>,
    pub status: String,
    pub source_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KpiFilter {
    pub org_id: Option<Uuid>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

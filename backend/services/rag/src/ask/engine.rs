use chrono::Utc;
use uuid::Uuid;

use ovia_common::error::OviaResult;
use ovia_db::ask::models::{AskSession, Citation};
use ovia_db::ask::repositories::AskRepository;
use ovia_db::kpi::repositories::KpiRepository;

use super::filters::AskFilters;

/// Response from the Ask engine.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AskResponse {
    pub session_id: Uuid,
    pub answer: String,
    pub confidence: String,
    pub assumptions: Option<String>,
    pub citations: Vec<Citation>,
}

/// Stub Ask engine that returns structured responses from DB data.
/// LLM integration is pending — this provides a basic response from KPI data.
pub struct AskEngine<A: AskRepository, K: KpiRepository> {
    ask_repo: A,
    kpi_repo: K,
}

impl<A: AskRepository, K: KpiRepository> AskEngine<A, K> {
    pub fn new(ask_repo: A, kpi_repo: K) -> Self {
        Self { ask_repo, kpi_repo }
    }

    /// Answer a question for a given org. For MVP this is a stub that:
    /// 1. Looks up the latest KPI snapshot
    /// 2. Formats a basic text answer with citations pointing to real data
    /// 3. Saves the session to DB
    pub async fn answer(
        &self,
        org_id: Uuid,
        query: &str,
        filters: Option<AskFilters>,
    ) -> OviaResult<AskResponse> {
        let start = std::time::Instant::now();

        // Look up latest KPI data
        let snapshot = self.kpi_repo.get_latest(org_id).await?;

        let (answer, confidence, citations) = if let Some(snap) = &snapshot {
            let answer = format!(
                "Based on the latest KPI snapshot (period {start} to {end}):\n\
                 - Delivery health score: {health}\n\
                 - Release risk score: {risk}\n\
                 - Total throughput: {throughput} items ({features} features, {bugs} bugs, {chores} chores)\n\
                 - Median review latency: {latency} hours\n\n\
                 Note: This is an automated summary. LLM-powered analysis is pending integration.",
                start = snap.period_start,
                end = snap.period_end,
                health = snap.delivery_health_score.map_or("N/A".to_string(), |v| format!("{v:.1}")),
                risk = snap.release_risk_score.map_or("N/A".to_string(), |v| format!("{v:.1}")),
                throughput = snap.throughput_total,
                features = snap.throughput_features,
                bugs = snap.throughput_bugs,
                chores = snap.throughput_chores,
                latency = snap.review_latency_median_hours.map_or("N/A".to_string(), |v| format!("{v:.1}")),
            );

            let citations = vec![Citation {
                source: "kpi_snapshot".to_string(),
                url: None,
                excerpt: format!(
                    "delivery_health_score: {}, release_risk_score: {}, throughput_total: {}",
                    snap.delivery_health_score
                        .map_or("null".to_string(), |v| format!("{v:.1}")),
                    snap.release_risk_score
                        .map_or("null".to_string(), |v| format!("{v:.1}")),
                    snap.throughput_total
                ),
            }];

            (answer, "medium".to_string(), citations)
        } else {
            let answer = format!(
                "No KPI data is available for this organization yet. \
                 Please run a KPI computation first.\n\n\
                 Your query was: \"{query}\"\n\n\
                 Note: This is an automated stub response. LLM-powered analysis is pending integration."
            );

            (answer, "low".to_string(), vec![])
        };

        let latency_ms = start.elapsed().as_millis() as i32;

        let filters_json = filters
            .as_ref()
            .map(|f| serde_json::to_value(f).unwrap_or_default());

        let session = AskSession {
            id: Uuid::new_v4(),
            org_id,
            query: query.to_string(),
            answer: Some(answer.clone()),
            confidence: Some(confidence.clone()),
            assumptions: Some("Stub response based on latest KPI snapshot data.".to_string()),
            citations: Some(citations.clone()),
            filters: filters_json,
            model: Some("stub-v1".to_string()),
            prompt_tokens: Some(0),
            completion_tokens: Some(0),
            latency_ms: Some(latency_ms),
            created_at: Utc::now(),
        };

        let saved = self.ask_repo.save_session(session).await?;

        Ok(AskResponse {
            session_id: saved.id,
            answer,
            confidence,
            assumptions: Some("Stub response based on latest KPI snapshot data.".to_string()),
            citations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};
    use ovia_db::ask::models::AskFilter;
    use ovia_db::kpi::models::{KpiFilter, KpiSnapshot, RiskItem};
    use std::sync::Mutex;

    struct MockAskRepo {
        sessions: Mutex<Vec<AskSession>>,
    }

    impl MockAskRepo {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl AskRepository for MockAskRepo {
        async fn save_session(&self, session: AskSession) -> OviaResult<AskSession> {
            self.sessions.lock().unwrap().push(session.clone());
            Ok(session)
        }

        async fn get_session(&self, _org_id: Uuid, id: Uuid) -> OviaResult<Option<AskSession>> {
            Ok(self
                .sessions
                .lock()
                .unwrap()
                .iter()
                .find(|s| s.id == id)
                .cloned())
        }

        async fn list_sessions(&self, _filter: AskFilter) -> OviaResult<Vec<AskSession>> {
            Ok(self.sessions.lock().unwrap().clone())
        }
    }

    struct MockKpiRepo {
        snapshot: Mutex<Option<KpiSnapshot>>,
    }

    impl MockKpiRepo {
        fn new(snapshot: Option<KpiSnapshot>) -> Self {
            Self {
                snapshot: Mutex::new(snapshot),
            }
        }
    }

    #[async_trait]
    impl KpiRepository for MockKpiRepo {
        async fn save_snapshot(&self, snapshot: KpiSnapshot) -> OviaResult<KpiSnapshot> {
            Ok(snapshot)
        }

        async fn get_latest(&self, _org_id: Uuid) -> OviaResult<Option<KpiSnapshot>> {
            Ok(self.snapshot.lock().unwrap().clone())
        }

        async fn list_snapshots(&self, _filter: KpiFilter) -> OviaResult<Vec<KpiSnapshot>> {
            Ok(vec![])
        }

        async fn save_risk_items(&self, items: Vec<RiskItem>) -> OviaResult<Vec<RiskItem>> {
            Ok(items)
        }

        async fn list_risk_items(&self, _snapshot_id: Uuid) -> OviaResult<Vec<RiskItem>> {
            Ok(vec![])
        }
    }

    fn make_snapshot() -> KpiSnapshot {
        KpiSnapshot {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 2, 14).unwrap(),
            delivery_health_score: Some(75.5),
            release_risk_score: Some(30.0),
            throughput_total: 42,
            throughput_bugs: 10,
            throughput_features: 25,
            throughput_chores: 7,
            review_latency_median_hours: Some(4.5),
            review_latency_p90_hours: None,
            blocker_count: 0,
            spillover_rate: None,
            cycle_time_p50_hours: None,
            cycle_time_p90_hours: None,
            computed_at: Utc::now(),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn answer_with_kpi_data_returns_structured_response() {
        let ask_repo = MockAskRepo::new();
        let kpi_repo = MockKpiRepo::new(Some(make_snapshot()));
        let engine = AskEngine::new(ask_repo, kpi_repo);

        let response = engine
            .answer(Uuid::new_v4(), "What is our delivery health?", None)
            .await
            .expect("should succeed");

        assert!(response.answer.contains("75.5"));
        assert_eq!(response.confidence, "medium");
        assert!(!response.citations.is_empty());
        assert_eq!(response.citations[0].source, "kpi_snapshot");
    }

    #[tokio::test]
    async fn answer_without_kpi_data_returns_stub() {
        let ask_repo = MockAskRepo::new();
        let kpi_repo = MockKpiRepo::new(None);
        let engine = AskEngine::new(ask_repo, kpi_repo);

        let response = engine
            .answer(Uuid::new_v4(), "What is our velocity?", None)
            .await
            .expect("should succeed");

        assert!(response.answer.contains("No KPI data"));
        assert_eq!(response.confidence, "low");
        assert!(response.citations.is_empty());
    }

    #[tokio::test]
    async fn answer_saves_session_to_repo() {
        let ask_repo = MockAskRepo::new();
        let kpi_repo = MockKpiRepo::new(Some(make_snapshot()));
        let engine = AskEngine::new(ask_repo, kpi_repo);

        let response = engine
            .answer(Uuid::new_v4(), "Test query", None)
            .await
            .expect("should succeed");

        assert!(!response.session_id.is_nil());
        assert!(response.assumptions.is_some());
    }

    #[tokio::test]
    async fn answer_with_filters_includes_them() {
        let ask_repo = MockAskRepo::new();
        let kpi_repo = MockKpiRepo::new(Some(make_snapshot()));
        let engine = AskEngine::new(ask_repo, kpi_repo);

        let filters = AskFilters {
            team: Some("backend".to_string()),
            product: None,
            date_range: None,
            sources: None,
        };

        let response = engine
            .answer(
                Uuid::new_v4(),
                "How is the backend team doing?",
                Some(filters),
            )
            .await
            .expect("should succeed");

        assert!(response.answer.contains("75.5"));
    }
}

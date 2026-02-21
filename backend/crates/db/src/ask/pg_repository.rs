use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

use crate::ask::models::{AskFilter, AskSession, Citation};
use crate::ask::repositories::AskRepository;
use ovia_common::error::{OviaError, OviaResult};

#[derive(Clone)]
pub struct PgAskRepository {
    pool: PgPool,
}

impl PgAskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_session_row(row: &sqlx::postgres::PgRow) -> AskSession {
    let citations_json: Option<serde_json::Value> = row.get("citations");
    let citations: Option<Vec<Citation>> =
        citations_json.and_then(|v| serde_json::from_value(v).ok());

    AskSession {
        id: row.get("id"),
        org_id: row.get("org_id"),
        query: row.get("query"),
        answer: row.get("answer"),
        confidence: row.get("confidence"),
        assumptions: row.get("assumptions"),
        citations,
        filters: row.get("filters"),
        model: row.get("model"),
        prompt_tokens: row.get("prompt_tokens"),
        completion_tokens: row.get("completion_tokens"),
        latency_ms: row.get("latency_ms"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl AskRepository for PgAskRepository {
    async fn save_session(&self, session: AskSession) -> OviaResult<AskSession> {
        let citations_json = session
            .citations
            .as_ref()
            .map(|c| serde_json::to_value(c).unwrap_or_default());

        let row = sqlx::query(
            "insert into ask_sessions
             (id, org_id, query, answer, confidence, assumptions, citations, filters,
              model, prompt_tokens, completion_tokens, latency_ms, created_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             returning id, org_id, query, answer, confidence, assumptions, citations,
                       filters, model, prompt_tokens, completion_tokens, latency_ms, created_at",
        )
        .bind(session.id)
        .bind(session.org_id)
        .bind(&session.query)
        .bind(&session.answer)
        .bind(&session.confidence)
        .bind(&session.assumptions)
        .bind(&citations_json)
        .bind(&session.filters)
        .bind(&session.model)
        .bind(session.prompt_tokens)
        .bind(session.completion_tokens)
        .bind(session.latency_ms)
        .bind(session.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(map_session_row(&row))
    }

    async fn get_session(&self, org_id: Uuid, id: Uuid) -> OviaResult<Option<AskSession>> {
        let row = sqlx::query(
            "select id, org_id, query, answer, confidence, assumptions, citations,
                    filters, model, prompt_tokens, completion_tokens, latency_ms, created_at
             from ask_sessions
             where org_id = $1 and id = $2",
        )
        .bind(org_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(row.as_ref().map(map_session_row))
    }

    async fn list_sessions(&self, filter: AskFilter) -> OviaResult<Vec<AskSession>> {
        let mut qb = QueryBuilder::new(
            "select id, org_id, query, answer, confidence, assumptions, citations, \
             filters, model, prompt_tokens, completion_tokens, latency_ms, created_at \
             from ask_sessions where 1=1",
        );

        if let Some(org_id) = filter.org_id {
            qb.push(" and org_id = ").push_bind(org_id);
        }

        qb.push(" order by created_at desc");
        qb.push(" limit ").push_bind(filter.limit.unwrap_or(50));
        qb.push(" offset ").push_bind(filter.offset.unwrap_or(0));

        let rows = qb
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(rows.iter().map(map_session_row).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;
    use chrono::Utc;

    async fn test_repo() -> Option<(PgAskRepository, PgPool)> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = create_pool(&url).await.expect("db should connect");

        // Ensure table exists
        sqlx::query(
            "create table if not exists ask_sessions (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null,
              query text not null,
              answer text,
              confidence text,
              assumptions text,
              citations jsonb,
              filters jsonb,
              model text,
              prompt_tokens integer,
              completion_tokens integer,
              latency_ms integer,
              created_at timestamptz not null default now()
            )",
        )
        .execute(&pool)
        .await
        .expect("create ask_sessions");

        sqlx::query(
            "create index if not exists ask_sessions_org_idx on ask_sessions(org_id, created_at desc)",
        )
        .execute(&pool)
        .await
        .expect("create ask_sessions index");

        Some((PgAskRepository::new(pool.clone()), pool))
    }

    fn make_session(org_id: Uuid) -> AskSession {
        AskSession {
            id: Uuid::new_v4(),
            org_id,
            query: "What is our delivery health?".to_string(),
            answer: Some("Your delivery health score is 75.5.".to_string()),
            confidence: Some("medium".to_string()),
            assumptions: Some("Based on current period data.".to_string()),
            citations: Some(vec![Citation {
                source: "kpi_snapshot".to_string(),
                url: None,
                excerpt: "delivery_health_score: 75.5".to_string(),
            }]),
            filters: Some(serde_json::json!({"team": "backend"})),
            model: Some("stub-v1".to_string()),
            prompt_tokens: Some(100),
            completion_tokens: Some(50),
            latency_ms: Some(200),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn save_and_get_session() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let session = make_session(org);
        let session_id = session.id;

        let saved = repo.save_session(session).await.expect("save");
        assert_eq!(saved.id, session_id);
        assert_eq!(saved.query, "What is our delivery health?");

        let fetched = repo.get_session(org, session_id).await.expect("get");
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, session_id);
        assert_eq!(fetched.confidence.as_deref(), Some("medium"));
        assert!(fetched.citations.is_some());
        assert_eq!(fetched.citations.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn get_session_returns_none_for_nonexistent() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };

        let result = repo
            .get_session(Uuid::new_v4(), Uuid::new_v4())
            .await
            .expect("get");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_session_returns_none_for_wrong_org() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let session = make_session(org);
        let session_id = session.id;

        repo.save_session(session).await.expect("save");

        // Try to get with different org
        let result = repo
            .get_session(Uuid::new_v4(), session_id)
            .await
            .expect("get");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_sessions_filters_by_org() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org1 = Uuid::new_v4();
        let org2 = Uuid::new_v4();

        repo.save_session(make_session(org1)).await.expect("save 1");
        repo.save_session(make_session(org2)).await.expect("save 2");

        let filter = AskFilter {
            org_id: Some(org1),
            ..Default::default()
        };
        let results = repo.list_sessions(filter).await.expect("list");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].org_id, org1);
    }

    #[tokio::test]
    async fn list_sessions_respects_limit_and_offset() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        for _ in 0..3 {
            repo.save_session(make_session(org)).await.expect("save");
        }

        let filter = AskFilter {
            org_id: Some(org),
            limit: Some(2),
            ..Default::default()
        };
        let results = repo.list_sessions(filter).await.expect("list");
        assert_eq!(results.len(), 2);

        let filter = AskFilter {
            org_id: Some(org),
            limit: Some(2),
            offset: Some(2),
        };
        let results = repo.list_sessions(filter).await.expect("list");
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn save_session_with_no_optional_fields() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let session = AskSession {
            id: Uuid::new_v4(),
            org_id: org,
            query: "Simple question".to_string(),
            answer: None,
            confidence: None,
            assumptions: None,
            citations: None,
            filters: None,
            model: None,
            prompt_tokens: None,
            completion_tokens: None,
            latency_ms: None,
            created_at: Utc::now(),
        };

        let saved = repo.save_session(session).await.expect("save");
        assert_eq!(saved.query, "Simple question");
        assert!(saved.answer.is_none());
        assert!(saved.citations.is_none());
    }
}

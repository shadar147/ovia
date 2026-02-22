use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

use crate::kpi::models::{KpiFilter, KpiSnapshot, RiskItem};
use crate::kpi::repositories::KpiRepository;
use ovia_common::error::{OviaError, OviaResult};

#[derive(Clone)]
pub struct PgKpiRepository {
    pool: PgPool,
}

impl PgKpiRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl KpiRepository for PgKpiRepository {
    async fn save_snapshot(&self, snapshot: KpiSnapshot) -> OviaResult<KpiSnapshot> {
        let row = sqlx::query(
            "insert into kpi_snapshots
             (id, org_id, period_start, period_end, delivery_health_score, release_risk_score,
              throughput_total, throughput_bugs, throughput_features, throughput_chores,
              review_latency_median_hours, review_latency_p90_hours,
              blocker_count, spillover_rate, cycle_time_p50_hours, cycle_time_p90_hours,
              computed_at, created_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
             on conflict (org_id, period_start, period_end)
             do update set
               delivery_health_score = excluded.delivery_health_score,
               release_risk_score = excluded.release_risk_score,
               throughput_total = excluded.throughput_total,
               throughput_bugs = excluded.throughput_bugs,
               throughput_features = excluded.throughput_features,
               throughput_chores = excluded.throughput_chores,
               review_latency_median_hours = excluded.review_latency_median_hours,
               review_latency_p90_hours = excluded.review_latency_p90_hours,
               blocker_count = excluded.blocker_count,
               spillover_rate = excluded.spillover_rate,
               cycle_time_p50_hours = excluded.cycle_time_p50_hours,
               cycle_time_p90_hours = excluded.cycle_time_p90_hours,
               computed_at = excluded.computed_at
             returning id, org_id, period_start, period_end,
                       delivery_health_score::float8 as delivery_health_score,
                       release_risk_score::float8 as release_risk_score,
                       throughput_total, throughput_bugs, throughput_features, throughput_chores,
                       review_latency_median_hours::float8 as review_latency_median_hours,
                       review_latency_p90_hours::float8 as review_latency_p90_hours,
                       blocker_count,
                       spillover_rate::float8 as spillover_rate,
                       cycle_time_p50_hours::float8 as cycle_time_p50_hours,
                       cycle_time_p90_hours::float8 as cycle_time_p90_hours,
                       computed_at, created_at",
        )
        .bind(snapshot.id)
        .bind(snapshot.org_id)
        .bind(snapshot.period_start)
        .bind(snapshot.period_end)
        .bind(snapshot.delivery_health_score)
        .bind(snapshot.release_risk_score)
        .bind(snapshot.throughput_total)
        .bind(snapshot.throughput_bugs)
        .bind(snapshot.throughput_features)
        .bind(snapshot.throughput_chores)
        .bind(snapshot.review_latency_median_hours)
        .bind(snapshot.review_latency_p90_hours)
        .bind(snapshot.blocker_count)
        .bind(snapshot.spillover_rate)
        .bind(snapshot.cycle_time_p50_hours)
        .bind(snapshot.cycle_time_p90_hours)
        .bind(snapshot.computed_at)
        .bind(snapshot.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(map_snapshot_row(&row))
    }

    async fn get_latest(&self, org_id: Uuid) -> OviaResult<Option<KpiSnapshot>> {
        let row = sqlx::query(
            "select id, org_id, period_start, period_end,
                    delivery_health_score::float8 as delivery_health_score,
                    release_risk_score::float8 as release_risk_score,
                    throughput_total, throughput_bugs, throughput_features, throughput_chores,
                    review_latency_median_hours::float8 as review_latency_median_hours,
                    review_latency_p90_hours::float8 as review_latency_p90_hours,
                    blocker_count,
                    spillover_rate::float8 as spillover_rate,
                    cycle_time_p50_hours::float8 as cycle_time_p50_hours,
                    cycle_time_p90_hours::float8 as cycle_time_p90_hours,
                    computed_at, created_at
             from kpi_snapshots
             where org_id = $1
             order by period_end desc, computed_at desc
             limit 1",
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(row.map(|r| map_snapshot_row(&r)))
    }

    async fn list_snapshots(&self, filter: KpiFilter) -> OviaResult<Vec<KpiSnapshot>> {
        let mut qb = QueryBuilder::new(
            "select id, org_id, period_start, period_end, \
             delivery_health_score::float8 as delivery_health_score, \
             release_risk_score::float8 as release_risk_score, \
             throughput_total, throughput_bugs, throughput_features, throughput_chores, \
             review_latency_median_hours::float8 as review_latency_median_hours, \
             review_latency_p90_hours::float8 as review_latency_p90_hours, \
             blocker_count, \
             spillover_rate::float8 as spillover_rate, \
             cycle_time_p50_hours::float8 as cycle_time_p50_hours, \
             cycle_time_p90_hours::float8 as cycle_time_p90_hours, \
             computed_at, created_at \
             from kpi_snapshots where 1=1",
        );

        if let Some(org_id) = filter.org_id {
            qb.push(" and org_id = ").push_bind(org_id);
        }
        if let Some(start) = filter.period_start {
            qb.push(" and period_start >= ").push_bind(start);
        }
        if let Some(end) = filter.period_end {
            qb.push(" and period_end <= ").push_bind(end);
        }

        qb.push(" order by period_end desc, computed_at desc");
        qb.push(" limit ").push_bind(filter.limit.unwrap_or(50));
        qb.push(" offset ").push_bind(filter.offset.unwrap_or(0));

        let rows = qb
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(rows.iter().map(map_snapshot_row).collect())
    }

    async fn save_risk_items(&self, items: Vec<RiskItem>) -> OviaResult<Vec<RiskItem>> {
        let mut saved = Vec::with_capacity(items.len());

        for item in items {
            let row = sqlx::query(
                "insert into risk_items
                 (id, org_id, snapshot_id, entity_type, title, owner, age_days,
                  impact_scope, status, source_url, created_at)
                 values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                 returning id, org_id, snapshot_id, entity_type, title, owner,
                           age_days, impact_scope, status, source_url, created_at",
            )
            .bind(item.id)
            .bind(item.org_id)
            .bind(item.snapshot_id)
            .bind(&item.entity_type)
            .bind(&item.title)
            .bind(&item.owner)
            .bind(item.age_days)
            .bind(&item.impact_scope)
            .bind(&item.status)
            .bind(&item.source_url)
            .bind(item.created_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

            saved.push(RiskItem {
                id: row.get("id"),
                org_id: row.get("org_id"),
                snapshot_id: row.get("snapshot_id"),
                entity_type: row.get("entity_type"),
                title: row.get("title"),
                owner: row.get("owner"),
                age_days: row.get("age_days"),
                impact_scope: row.get("impact_scope"),
                status: row.get("status"),
                source_url: row.get("source_url"),
                created_at: row.get("created_at"),
            });
        }

        Ok(saved)
    }

    async fn list_risk_items(&self, snapshot_id: Uuid) -> OviaResult<Vec<RiskItem>> {
        let rows = sqlx::query(
            "select id, org_id, snapshot_id, entity_type, title, owner,
                    age_days, impact_scope, status, source_url, created_at
             from risk_items
             where snapshot_id = $1
             order by age_days desc, created_at desc",
        )
        .bind(snapshot_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| RiskItem {
                id: r.get("id"),
                org_id: r.get("org_id"),
                snapshot_id: r.get("snapshot_id"),
                entity_type: r.get("entity_type"),
                title: r.get("title"),
                owner: r.get("owner"),
                age_days: r.get("age_days"),
                impact_scope: r.get("impact_scope"),
                status: r.get("status"),
                source_url: r.get("source_url"),
                created_at: r.get("created_at"),
            })
            .collect())
    }
}

fn map_snapshot_row(row: &sqlx::postgres::PgRow) -> KpiSnapshot {
    KpiSnapshot {
        id: row.get("id"),
        org_id: row.get("org_id"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        delivery_health_score: row.get("delivery_health_score"),
        release_risk_score: row.get("release_risk_score"),
        throughput_total: row.get("throughput_total"),
        throughput_bugs: row.get("throughput_bugs"),
        throughput_features: row.get("throughput_features"),
        throughput_chores: row.get("throughput_chores"),
        review_latency_median_hours: row.get("review_latency_median_hours"),
        review_latency_p90_hours: row.get("review_latency_p90_hours"),
        blocker_count: row.get("blocker_count"),
        spillover_rate: row.get("spillover_rate"),
        cycle_time_p50_hours: row.get("cycle_time_p50_hours"),
        cycle_time_p90_hours: row.get("cycle_time_p90_hours"),
        computed_at: row.get("computed_at"),
        created_at: row.get("created_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;
    use chrono::{NaiveDate, Utc};

    async fn test_repo() -> Option<(PgKpiRepository, PgPool)> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = create_pool(&url).await.expect("db should connect");

        // Ensure tables exist
        sqlx::query(
            "create table if not exists kpi_snapshots (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null,
              period_start date not null,
              period_end date not null,
              delivery_health_score numeric(5,2),
              release_risk_score numeric(5,2),
              throughput_total integer not null default 0,
              throughput_bugs integer not null default 0,
              throughput_features integer not null default 0,
              throughput_chores integer not null default 0,
              review_latency_median_hours numeric(8,2),
              review_latency_p90_hours numeric(8,2),
              computed_at timestamptz not null default now(),
              created_at timestamptz not null default now()
            )",
        )
        .execute(&pool)
        .await
        .expect("create kpi_snapshots");

        sqlx::query(
            "create unique index if not exists kpi_snapshots_org_period_uidx
             on kpi_snapshots(org_id, period_start, period_end)",
        )
        .execute(&pool)
        .await
        .expect("create kpi_snapshots index");

        // Jira KPI columns (migration 0007)
        for stmt in &[
            "alter table kpi_snapshots add column if not exists blocker_count integer not null default 0",
            "alter table kpi_snapshots add column if not exists spillover_rate numeric(5,4)",
            "alter table kpi_snapshots add column if not exists cycle_time_p50_hours numeric(8,2)",
            "alter table kpi_snapshots add column if not exists cycle_time_p90_hours numeric(8,2)",
        ] {
            sqlx::query(stmt)
                .execute(&pool)
                .await
                .expect("alter kpi_snapshots");
        }

        sqlx::query(
            "create table if not exists risk_items (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null,
              snapshot_id uuid not null references kpi_snapshots(id) on delete cascade,
              entity_type text not null,
              title text not null,
              owner text,
              age_days integer not null default 0,
              impact_scope text,
              status text not null,
              source_url text,
              created_at timestamptz not null default now()
            )",
        )
        .execute(&pool)
        .await
        .expect("create risk_items");

        sqlx::query(
            "create index if not exists risk_items_snapshot_idx on risk_items(snapshot_id)",
        )
        .execute(&pool)
        .await
        .expect("create risk_items index");

        Some((PgKpiRepository::new(pool.clone()), pool))
    }

    fn make_snapshot(org_id: Uuid) -> KpiSnapshot {
        let now = Utc::now();
        KpiSnapshot {
            id: Uuid::new_v4(),
            org_id,
            period_start: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 2, 14).unwrap(),
            delivery_health_score: Some(75.5),
            release_risk_score: Some(30.0),
            throughput_total: 42,
            throughput_bugs: 10,
            throughput_features: 25,
            throughput_chores: 7,
            review_latency_median_hours: Some(4.5),
            review_latency_p90_hours: Some(12.0),
            blocker_count: 2,
            spillover_rate: Some(0.15),
            cycle_time_p50_hours: Some(36.0),
            cycle_time_p90_hours: Some(72.0),
            computed_at: now,
            created_at: now,
        }
    }

    fn make_risk_item(org_id: Uuid, snapshot_id: Uuid) -> RiskItem {
        RiskItem {
            id: Uuid::new_v4(),
            org_id,
            snapshot_id,
            entity_type: "pull_request".to_string(),
            title: "Stale PR: fix auth bug".to_string(),
            owner: Some("alice".to_string()),
            age_days: 14,
            impact_scope: Some("auth-service".to_string()),
            status: "open".to_string(),
            source_url: Some("https://github.com/org/repo/pull/123".to_string()),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn save_and_get_latest_snapshot() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let snapshot = make_snapshot(org);

        let saved = repo.save_snapshot(snapshot.clone()).await.expect("save");
        assert_eq!(saved.org_id, org);
        assert_eq!(saved.throughput_total, 42);

        let latest = repo.get_latest(org).await.expect("get_latest");
        assert!(latest.is_some());
        let latest = latest.unwrap();
        assert_eq!(latest.id, saved.id);
        assert!((latest.delivery_health_score.unwrap() - 75.5).abs() < 0.1);
    }

    #[tokio::test]
    async fn get_latest_returns_none_for_new_org() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };

        let result = repo.get_latest(Uuid::new_v4()).await.expect("get_latest");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_snapshots_filters_by_org() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org1 = Uuid::new_v4();
        let org2 = Uuid::new_v4();

        repo.save_snapshot(make_snapshot(org1))
            .await
            .expect("save org1");
        repo.save_snapshot(make_snapshot(org2))
            .await
            .expect("save org2");

        let filter = KpiFilter {
            org_id: Some(org1),
            ..Default::default()
        };
        let results = repo.list_snapshots(filter).await.expect("list");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].org_id, org1);
    }

    #[tokio::test]
    async fn list_snapshots_filters_by_date_range() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        let mut s1 = make_snapshot(org);
        s1.period_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        s1.period_end = NaiveDate::from_ymd_opt(2026, 1, 14).unwrap();
        repo.save_snapshot(s1).await.expect("save s1");

        let s2 = make_snapshot(org);
        repo.save_snapshot(s2).await.expect("save s2");

        let filter = KpiFilter {
            org_id: Some(org),
            period_start: Some(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()),
            ..Default::default()
        };
        let results = repo.list_snapshots(filter).await.expect("list");
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].period_start,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()
        );
    }

    #[tokio::test]
    async fn save_and_list_risk_items() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let snapshot = repo.save_snapshot(make_snapshot(org)).await.expect("save");

        let item1 = make_risk_item(org, snapshot.id);
        let mut item2 = make_risk_item(org, snapshot.id);
        item2.title = "Another risk".to_string();
        item2.age_days = 7;

        let saved = repo
            .save_risk_items(vec![item1, item2])
            .await
            .expect("save risk items");
        assert_eq!(saved.len(), 2);

        let listed = repo
            .list_risk_items(snapshot.id)
            .await
            .expect("list risk items");
        assert_eq!(listed.len(), 2);
        // Ordered by age_days desc
        assert!(listed[0].age_days >= listed[1].age_days);
    }

    #[tokio::test]
    async fn list_risk_items_empty_for_nonexistent_snapshot() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };

        let listed = repo
            .list_risk_items(Uuid::new_v4())
            .await
            .expect("list risk items");
        assert!(listed.is_empty());
    }

    #[tokio::test]
    async fn save_snapshot_upserts_on_conflict() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let mut snapshot = make_snapshot(org);
        snapshot.throughput_total = 10;

        let saved1 = repo.save_snapshot(snapshot.clone()).await.expect("save 1");
        assert_eq!(saved1.throughput_total, 10);

        // Save again with same org/period but different data — should upsert
        let mut snapshot2 = make_snapshot(org);
        snapshot2.id = Uuid::new_v4(); // different id, same org+period
        snapshot2.throughput_total = 99;

        let saved2 = repo.save_snapshot(snapshot2).await.expect("save 2");
        assert_eq!(saved2.throughput_total, 99);
        // Should return the original id (from the existing row)
        assert_eq!(saved2.id, saved1.id);
    }
}

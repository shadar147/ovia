mod kpi;

use chrono::{Datelike, NaiveDate, Utc};
use ovia_config::{init_tracing, AppConfig};
use ovia_db::kpi::pg_repository::PgKpiRepository;
use uuid::Uuid;

use kpi::service::KpiService;

#[tokio::main]
async fn main() {
    init_tracing("info");

    let config = AppConfig::from_env().expect("failed to load config");
    tracing::info!(service = "ovia-metrics", "starting");

    let pool = ovia_db::create_pool(&config.database_url)
        .await
        .expect("failed to create database pool");

    let kpi_repo = PgKpiRepository::new(pool.clone());
    let kpi_service = KpiService::new(kpi_repo, pool);

    // One-shot computation: compute current period snapshot
    let today = Utc::now().date_naive();
    let period_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
    let period_end = today;

    // Use a configurable org_id for the one-shot run
    let org_id = std::env::var("ORG_ID")
        .ok()
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::nil);

    if org_id.is_nil() {
        tracing::warn!("ORG_ID not set or invalid — skipping KPI computation");
    } else {
        match kpi_service
            .compute_and_save(org_id, period_start, period_end)
            .await
        {
            Ok(snapshot) => {
                tracing::info!(
                    snapshot_id = %snapshot.id,
                    org_id = %snapshot.org_id,
                    health = ?snapshot.delivery_health_score,
                    risk = ?snapshot.release_risk_score,
                    "KPI snapshot saved"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to compute KPI snapshot");
            }
        }
    }

    tracing::info!("metrics service completed one-shot KPI computation");
}

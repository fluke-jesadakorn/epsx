//! Cloudflare Worker for `epsx` backend (Axum → workers-rs).
//! Core PG via Hyperdrive (`HYPERDRIVE_CORE` → sqlx), R2/KV/D1/Queues bindings file-persisted locally.
//! Uses `axum-cloudflare-adapter` (http 0.0.21+) to reuse existing `apps/backend/src/main.rs` router when `workers` feature is enabled.
use worker::*;
#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // In local `--persist-to=.wrangler/state`, D1/KV/R2 are Miniflare file-backed SQLite.
    // Hyperdrive localConnectionString → host.docker.internal:5432 (pool ≤5 per Worker).
    let r2_public = env.var("R2_PUBLIC_URL").map(|v| v.to_string()).unwrap_or_else(|_| "http://localhost:8788/assets".to_string());
    Response::ok(format!("epsx backend Worker (env={}, r2={}, hyperdrive=core)", env.var("ENV").map(|v| v.to_string()).unwrap_or("local".into()), r2_public))
}
#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!("epsx-backend cron — indexer off Free (use Containers or Queues)");
}
#[event(queue)]
async fn queue(batch: Batch<()>, _env: Env, _ctx: Context) -> Result<()> {
    console_log!("epsx-backend queue batch {}", batch.messages().len());
    Ok(())
}

use axum::{
    extract::{Path as AxPath, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, post, put},
    Json, Router,
};
use epsx_renderer::{render_block, render_page, Block, Page};
use clap::Parser;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "epsx-content", about = "EPSX Content/CMS Service")]
struct Args {
    #[arg(long, default_value = "8105")]
    port: u16,
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(long, default_value = "postgres://epsx:epsx@localhost:5432/epsx_content")]
    database_url: String,
    #[arg(long, default_value = "../content")]
    content_path: String,
    #[arg(long, default_value = "false")]
    no_watch: bool,
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    content_path: PathBuf,
    block_registry: Arc<RwLock<BlockRegistry>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct BlockRegistry {
    blocks: Vec<BlockManifest>,
    themes: Vec<ThemeManifest>,
    navigation: Vec<NavigationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockManifest {
    id: String,
    version: String,
    name: String,
    category: String,
    schema: String,
    default_props: serde_json::Value,
    admin_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThemeManifest {
    name: String,
    is_default: bool,
    config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NavigationItem {
    id: String,
    label: String,
    url: String,
    order: i32,
    visible: bool,
    parent: Option<String>,
}

#[derive(Serialize, FromRow)]
struct DbPage {
    id: String,
    slug: String,
    title: String,
    locale: String,
    status: String,
    blocks_json: String,
    seo_json: Option<String>,
    theme_id: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    published_at: Option<chrono::NaiveDateTime>,
}

#[derive(Deserialize)]
struct CreatePageRequest {
    slug: String,
    title: String,
    locale: Option<String>,
    blocks: Option<Vec<serde_json::Value>>,
    seo: Option<serde_json::Value>,
    theme_id: Option<String>,
}

#[derive(Deserialize)]
struct UpdatePageRequest {
    title: Option<String>,
    blocks: Option<Vec<serde_json::Value>>,
    seo: Option<serde_json::Value>,
    theme_id: Option<String>,
    status: Option<String>,
}

#[derive(Serialize, FromRow)]
struct DbTheme {
    id: String,
    name: String,
    colors_json: String,
    fonts_json: String,
    spacing_json: String,
    breakpoints_json: String,
    radius_json: Option<String>,
    is_default: bool,
}

#[derive(Serialize, FromRow)]
struct DbBlockType {
    id: String,
    block_type: String,
    name: String,
    category: String,
    description: Option<String>,
    schema_json: String,
    default_props_json: String,
    admin_only: bool,
}

#[derive(Deserialize)]
struct StartEditRequest {
    page_id: String,
    user_id: String,
}

#[derive(Deserialize)]
struct CommitEditRequest {
    session_id: String,
    commit_message: Option<String>,
    publish: bool,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("content");
    let args = Args::parse();

    let db = sqlx::PgPool::connect(&args.database_url).await.expect("Failed to connect to database");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            slug VARCHAR(255) UNIQUE NOT NULL,
            title VARCHAR(255) NOT NULL,
            locale VARCHAR(10) DEFAULT 'en',
            status VARCHAR(20) DEFAULT 'draft',
            blocks_json JSONB DEFAULT '[]',
            seo_json JSONB DEFAULT '{}',
            theme_id UUID,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            published_at TIMESTAMPTZ
        )"
    ).execute(&db).await.expect("Failed to create pages table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS themes (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            colors_json JSONB DEFAULT '{}',
            fonts_json JSONB DEFAULT '{}',
            spacing_json JSONB DEFAULT '{}',
            breakpoints_json JSONB DEFAULT '{}',
            radius_json JSONB DEFAULT '{}',
            is_default BOOLEAN DEFAULT false
        )"
    ).execute(&db).await.expect("Failed to create themes table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS block_types (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            block_type VARCHAR(50) UNIQUE NOT NULL,
            name VARCHAR(100) NOT NULL,
            category VARCHAR(50) NOT NULL,
            description TEXT,
            schema_json JSONB DEFAULT '{}',
            default_props_json JSONB DEFAULT '{}',
            admin_only BOOLEAN DEFAULT false,
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )"
    ).execute(&db).await.expect("Failed to create block_types table");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS edit_sessions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            page_id UUID REFERENCES pages(id) ON DELETE CASCADE,
            user_id UUID NOT NULL,
            status VARCHAR(20) DEFAULT 'active',
            started_at TIMESTAMPTZ DEFAULT NOW(),
            ended_at TIMESTAMPTZ
        )"
    ).execute(&db).await.expect("Failed to create edit_sessions table");

    let content_path = PathBuf::from(&args.content_path);
    let block_registry = Arc::new(RwLock::new(BlockRegistry::default()));

    // Load initial block registry
    if let Err(e) = load_block_registry(&content_path, &block_registry).await {
        error!("Failed to load block registry: {}", e);
    }

    // Sync to DB
    if let Err(e) = sync_blocks_to_db(&db, &*block_registry.read().await).await {
        error!("Failed to sync block types: {}", e);
    }
    if let Err(e) = sync_themes_to_db(&db, &content_path).await {
        error!("Failed to sync themes: {}", e);
    }

    // Start file watcher
    if !args.no_watch {
        let watch_path = content_path.clone();
        let watch_registry = block_registry.clone();
        tokio::spawn(async move {
            if let Err(e) = watch_files(watch_path, watch_registry).await {
                error!("File watcher error: {}", e);
            }
        });
    }

    let state = AppState { db, content_path, block_registry };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/content/pages/{slug}", get(get_page).put(update_page))
        .route("/api/v1/content/pages", post(create_page).get(list_pages))
        .route("/api/v1/content/pages/{id}/publish", post(publish_page))
        .route("/api/v1/content/pages/{slug}/render", get(render_page_html))
        .route("/api/v1/content/themes", get(list_themes).post(create_theme))
        .route("/api/v1/content/themes/{id}", get(get_theme).put(update_theme))
        .route("/api/v1/content/blocks", get(list_block_types))
        .route("/api/v1/content/blocks/{block_type}", get(get_block_schema))
        .route("/api/v1/content/edit/start", post(start_edit_session))
        .route("/api/v1/content/edit/commit", post(commit_edit_session))
        .route("/api/v1/content/edit/sessions", get(list_edit_sessions))
        .route("/api/v1/content/navigation", get(get_navigation))
        .route("/api/v1/content/site", get(get_site_settings))
        .route("/api/v1/content/news", get(news_list))
        .route("/api/v1/content/news/{slug}", get(news_post))
        .route("/api/v1/content/plans", get(plans_list))
        .route("/api/v1/content/rankings", get(rankings_list))
        .route("/api/v1/content/portfolio/{addr}", get(portfolio_get))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse().unwrap();
    info!("Content service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode { StatusCode::OK }

async fn load_block_registry(path: &PathBuf, registry: &Arc<RwLock<BlockRegistry>>) -> Result<(), String> {
    let mut reg = registry.write().await;

    let blocks_dir = path.join("blocks");
    if blocks_dir.exists() {
        let entries = std::fs::read_dir(&blocks_dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let block_dir = entry.path();
            if !block_dir.is_dir() { continue; }
            let manifest_path = block_dir.join("manifest.json");
            if !manifest_path.exists() { continue; }
            let raw = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
            let manifest: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            reg.blocks.push(BlockManifest {
                id: manifest["id"].as_str().unwrap_or("").to_string(),
                version: manifest["version"].as_str().unwrap_or("1.0.0").to_string(),
                name: manifest["name"].as_str().unwrap_or("").to_string(),
                category: manifest["category"].as_str().unwrap_or("misc").to_string(),
                schema: manifest["schema"].as_str().unwrap_or("").to_string(),
                default_props: manifest["defaultProps"].clone(),
                admin_only: manifest["adminOnly"].as_bool().unwrap_or(false),
            });
        }
    }

    let themes_dir = path.join("themes");
    if themes_dir.exists() {
        let entries = std::fs::read_dir(&themes_dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") { continue; }
            let raw = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
            let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            reg.themes.push(ThemeManifest {
                name: v["name"].as_str().unwrap_or("default").to_string(),
                is_default: v["is_default"].as_bool().unwrap_or(false),
                config: v,
            });
        }
    }

    let nav_path = path.join("navigation").join("main.json");
    if nav_path.exists() {
        if let Ok(raw) = std::fs::read_to_string(&nav_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(menus) = v.get("menus").and_then(|m| m.as_object()) {
                    if let Some(main) = menus.get("main").and_then(|m| m.as_array()) {
                        for item in main {
                            reg.navigation.push(NavigationItem {
                                id: item["id"].as_str().unwrap_or("").to_string(),
                                label: item["label"].as_str().unwrap_or("").to_string(),
                                url: item["url"].as_str().unwrap_or("/").to_string(),
                                order: item["order"].as_i64().unwrap_or(0) as i32,
                                visible: item["visible"].as_bool().unwrap_or(true),
                                parent: item["parent"].as_str().map(|s| s.to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    info!("Loaded {} blocks, {} themes, {} nav items",
        reg.blocks.len(), reg.themes.len(), reg.navigation.len());
    Ok(())
}

async fn watch_files(path: PathBuf, registry: Arc<RwLock<BlockRegistry>>) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(500), tx).map_err(|e| e.to_string())?;
    debouncer.watcher().watch(&path, RecursiveMode::Recursive).map_err(|e| e.to_string())?;
    info!("File watcher started for {}", path.display());

    for res in rx {
        match res {
            Ok(events) => {
                if !events.is_empty() {
                    info!("Detected {} file change(s), reloading registry", events.len());
                    if let Err(e) = load_block_registry(&path, &registry).await {
                        error!("Reload failed: {}", e);
                    }
                }
            }
            Err(e) => error!("Watch error: {:?}", e),
        }
    }
    Ok(())
}

async fn sync_blocks_to_db(db: &sqlx::PgPool, registry: &BlockRegistry) -> Result<(), String> {
    for block in &registry.blocks {
        sqlx::query(
            "INSERT INTO block_types (block_type, name, category, schema_json, default_props_json, admin_only)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (block_type) DO UPDATE SET
                name = EXCLUDED.name,
                category = EXCLUDED.category,
                schema_json = EXCLUDED.schema_json,
                default_props_json = EXCLUDED.default_props_json,
                admin_only = EXCLUDED.admin_only,
                updated_at = NOW()"
        )
        .bind(&block.id)
        .bind(&block.name)
        .bind(&block.category)
        .bind(serde_json::to_string(&serde_json::json!({
            "type": block.id,
            "props": block.default_props
        })).unwrap_or_else(|_| "{}".to_string()))
        .bind(serde_json::to_string(&block.default_props).unwrap_or_else(|_| "{}".to_string()))
        .bind(block.admin_only)
        .execute(db)
        .await
        .map_err(|e| format!("DB error for block {}: {}", block.id, e))?;
    }
    info!("Synced {} block types to DB", registry.blocks.len());
    Ok(())
}

async fn sync_themes_to_db(db: &sqlx::PgPool, content_path: &PathBuf) -> Result<(), String> {
    let themes_dir = content_path.join("themes");
    if !themes_dir.exists() { return Ok(()); }
    let entries = std::fs::read_dir(&themes_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") { continue; }
        let raw = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        let name = v["name"].as_str().unwrap_or("default");
        let is_default = v["is_default"].as_bool().unwrap_or(name == "default");
        let colors = v.get("colors").cloned().unwrap_or(serde_json::json!({}));
        let fonts = v.get("fonts").cloned().unwrap_or(serde_json::json!({}));
        let spacing = v.get("spacing").cloned().unwrap_or(serde_json::json!({}));
        let breakpoints = v.get("breakpoints").cloned().unwrap_or(serde_json::json!({}));
        let radius = v.get("radius").cloned().unwrap_or(serde_json::json!({}));
        sqlx::query(
            "INSERT INTO themes (name, colors_json, fonts_json, spacing_json, breakpoints_json, radius_json, is_default)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (name) DO UPDATE SET
                colors_json = EXCLUDED.colors_json,
                fonts_json = EXCLUDED.fonts_json,
                spacing_json = EXCLUDED.spacing_json,
                breakpoints_json = EXCLUDED.breakpoints_json,
                radius_json = EXCLUDED.radius_json,
                is_default = EXCLUDED.is_default"
        )
        .bind(name)
        .bind(colors.to_string())
        .bind(fonts.to_string())
        .bind(spacing.to_string())
        .bind(breakpoints.to_string())
        .bind(radius.to_string())
        .bind(is_default)
        .execute(db)
        .await
        .map_err(|e| format!("DB error for theme {}: {}", name, e))?;
    }
    info!("Synced themes to DB");
    Ok(())
}

async fn get_page(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row: DbPage = sqlx::query_as::<_, DbPage>(
        "SELECT id, slug, title, locale, status, blocks_json, seo_json, theme_id, created_at, updated_at, published_at FROM pages WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let blocks: serde_json::Value = serde_json::from_str(&row.blocks_json).unwrap_or_else(|_| serde_json::json!([]));
    let seo: serde_json::Value = row.seo_json.as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    Ok(Json(serde_json::json!({
        "id": row.id,
        "slug": row.slug,
        "title": row.title,
        "locale": row.locale,
        "status": row.status,
        "blocks": blocks,
        "blocks_json": row.blocks_json,
        "seo": seo,
        "seo_json": row.seo_json,
        "theme_id": row.theme_id,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "published_at": row.published_at,
    })))
}

async fn list_pages(State(state): State<AppState>) -> Result<Json<Vec<DbPage>>, StatusCode> {
    let pages: Vec<DbPage> = sqlx::query_as::<_, DbPage>("SELECT id, slug, title, locale, status, blocks_json, seo_json, theme_id, created_at, updated_at, published_at FROM pages ORDER BY updated_at DESC")
        .fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(pages))
}

async fn create_page(
    State(state): State<AppState>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<DbPage>, StatusCode> {
    let locale = req.locale.unwrap_or_else(|| "en".to_string());
    let blocks = serde_json::to_string(&req.blocks.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
    let seo = serde_json::to_string(&req.seo.unwrap_or_default()).unwrap_or_else(|_| "{}".to_string());

    let page: DbPage = sqlx::query_as::<_, DbPage>(
        "INSERT INTO pages (slug, title, locale, blocks_json, seo_json, theme_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
    )
    .bind(&req.slug)
    .bind(&req.title)
    .bind(&locale)
    .bind(&blocks)
    .bind(&seo)
    .bind(req.theme_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| { tracing::error!("create_page: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(page))
}

async fn update_page(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
    Json(req): Json<UpdatePageRequest>,
) -> Result<Json<DbPage>, StatusCode> {
    let current: DbPage = sqlx::query_as::<_, DbPage>(
        "SELECT id, slug, title, locale, status, blocks_json, seo_json, theme_id, created_at, updated_at, published_at FROM pages WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let new_title = req.title.unwrap_or(current.title);
    let new_blocks = match req.blocks {
        Some(b) => serde_json::to_string(&b).unwrap_or_else(|_| "[]".to_string()),
        None => current.blocks_json,
    };
    let new_seo = match req.seo {
        Some(s) => serde_json::to_string(&s).unwrap_or_else(|_| "{}".to_string()),
        None => current.seo_json.unwrap_or_else(|| "{}".to_string()),
    };
    let new_theme = req.theme_id.or(current.theme_id);
    let new_status = req.status.unwrap_or(current.status);

    let page: DbPage = sqlx::query_as::<_, DbPage>(
        "UPDATE pages SET title = $2, blocks_json = $3, seo_json = $4, theme_id = $5, status = $6, updated_at = NOW() WHERE id = $1 RETURNING *"
    )
    .bind(&current.id)
    .bind(&new_title)
    .bind(&new_blocks)
    .bind(&new_seo)
    .bind(&new_theme)
    .bind(&new_status)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(page))
}

async fn publish_page(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
) -> Result<Json<DbPage>, StatusCode> {
    let page: DbPage = sqlx::query_as::<_, DbPage>(
        "UPDATE pages SET status = 'published', published_at = NOW(), updated_at = NOW() WHERE slug = $1 RETURNING *"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    info!("Published page: {}", page.slug);
    Ok(Json(page))
}

async fn render_page_html(
    State(state): State<AppState>,
    AxPath(slug): AxPath<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let page: DbPage = sqlx::query_as::<_, DbPage>(
        "SELECT id, slug, title, locale, status, blocks_json, seo_json, theme_id, created_at, updated_at, published_at FROM pages WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let blocks_json: serde_json::Value = serde_json::from_str(&page.blocks_json).unwrap_or_else(|_| serde_json::json!([]));
    let blocks: Vec<Block> = serde_json::from_value(blocks_json).unwrap_or_default();

    let domain_page = Page {
        slug: page.slug.clone(),
        title: page.title.clone(),
        blocks,
        seo: page.seo_json.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_else(|| serde_json::json!({})),
        theme: page.theme_id,
    };

    let html = render_page(&domain_page);
    Ok((StatusCode::OK, [("content-type", "text/html; charset=utf-8")], html))
}

async fn list_themes(State(state): State<AppState>) -> Result<Json<Vec<DbTheme>>, StatusCode> {
    let themes: Vec<DbTheme> = sqlx::query_as::<_, DbTheme>(
        "SELECT id, name, colors_json, fonts_json, spacing_json, breakpoints_json, radius_json, is_default FROM themes"
    )
    .fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(themes))
}

async fn get_theme(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Json<DbTheme>, StatusCode> {
    let theme: DbTheme = sqlx::query_as::<_, DbTheme>(
        "SELECT id, name, colors_json, fonts_json, spacing_json, breakpoints_json, radius_json, is_default FROM themes WHERE id = $1 OR name = $1"
    )
    .bind(&id)
    .fetch_optional(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(theme))
}

async fn create_theme(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<DbTheme>, StatusCode> {
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("default");
    let colors = body.get("colors").cloned().unwrap_or(serde_json::json!({}));
    let fonts = body.get("fonts").cloned().unwrap_or(serde_json::json!({}));
    let spacing = body.get("spacing").cloned().unwrap_or(serde_json::json!({}));
    let breakpoints = body.get("breakpoints").cloned().unwrap_or(serde_json::json!({}));
    let radius = body.get("radius").cloned().unwrap_or(serde_json::json!({}));
    let is_default = body.get("is_default").and_then(|v| v.as_bool()).unwrap_or(false);

    let theme: DbTheme = sqlx::query_as::<_, DbTheme>(
        "INSERT INTO themes (name, colors_json, fonts_json, spacing_json, breakpoints_json, radius_json, is_default) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
    )
    .bind(name)
    .bind(colors.to_string())
    .bind(fonts.to_string())
    .bind(spacing.to_string())
    .bind(breakpoints.to_string())
    .bind(radius.to_string())
    .bind(is_default)
    .fetch_one(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(theme))
}

async fn update_theme(
    State(state): State<AppState>,
    AxPath(id): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<DbTheme>, StatusCode> {
    let colors = body.get("colors").map(|v| v.to_string()).unwrap_or_else(|| "{}".to_string());
    let fonts = body.get("fonts").map(|v| v.to_string()).unwrap_or_else(|| "{}".to_string());
    let spacing = body.get("spacing").map(|v| v.to_string()).unwrap_or_else(|| "{}".to_string());
    let breakpoints = body.get("breakpoints").map(|v| v.to_string()).unwrap_or_else(|| "{}".to_string());
    let radius = body.get("radius").map(|v| v.to_string()).unwrap_or_else(|| "{}".to_string());

    let theme: DbTheme = sqlx::query_as::<_, DbTheme>(
        "UPDATE themes SET colors_json = $2, fonts_json = $3, spacing_json = $4, breakpoints_json = $5, radius_json = $6 WHERE id = $1 RETURNING *"
    )
    .bind(&id)
    .bind(colors)
    .bind(fonts)
    .bind(spacing)
    .bind(breakpoints)
    .bind(radius)
    .fetch_optional(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(theme))
}

async fn list_block_types(State(state): State<AppState>) -> Result<Json<Vec<DbBlockType>>, StatusCode> {
    let blocks: Vec<DbBlockType> = sqlx::query_as::<_, DbBlockType>(
        "SELECT id, block_type, name, category, description, schema_json, default_props_json, admin_only FROM block_types ORDER BY category, name"
    )
    .fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(blocks))
}

async fn get_block_schema(
    State(state): State<AppState>,
    AxPath(block_type): AxPath<String>,
) -> Result<Json<DbBlockType>, StatusCode> {
    let block: DbBlockType = sqlx::query_as::<_, DbBlockType>(
        "SELECT id, block_type, name, category, description, schema_json, default_props_json, admin_only FROM block_types WHERE block_type = $1"
    )
    .bind(&block_type)
    .fetch_optional(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(block))
}

#[derive(Serialize, FromRow)]
struct EditSession {
    id: String,
    page_id: String,
    user_id: String,
    status: String,
    started_at: chrono::NaiveDateTime,
    ended_at: Option<chrono::NaiveDateTime>,
}

async fn start_edit_session(
    State(state): State<AppState>,
    Json(req): Json<StartEditRequest>,
) -> Result<Json<EditSession>, StatusCode> {
    let session: EditSession = sqlx::query_as::<_, EditSession>(
        "INSERT INTO edit_sessions (page_id, user_id) VALUES ($1, $2) RETURNING *"
    )
    .bind(&req.page_id)
    .bind(&req.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(session))
}

async fn commit_edit_session(
    State(state): State<AppState>,
    Json(req): Json<CommitEditRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session: EditSession = sqlx::query_as::<_, EditSession>(
        "UPDATE edit_sessions SET status = 'committed', ended_at = NOW() WHERE id = $1 RETURNING *"
    )
    .bind(&req.session_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if req.publish {
        sqlx::query("UPDATE pages SET status = 'published', published_at = NOW() WHERE id = $1")
            .bind(&session.page_id)
            .execute(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({
        "session_id": session.id,
        "page_id": session.page_id,
        "published": req.publish,
        "commit_message": req.commit_message,
        "commit_hash": uuid::Uuid::new_v4().to_string()
    })))
}

async fn list_edit_sessions(State(state): State<AppState>) -> Result<Json<Vec<EditSession>>, StatusCode> {
    let sessions: Vec<EditSession> = sqlx::query_as::<_, EditSession>(
        "SELECT id, page_id, user_id, status, started_at, ended_at FROM edit_sessions ORDER BY started_at DESC LIMIT 100"
    )
    .fetch_all(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(sessions))
}

async fn get_navigation(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let reg = state.block_registry.read().await;
    Ok(Json(serde_json::json!({
        "items": reg.navigation
    })))
}

async fn get_site_settings(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let settings_path = state.content_path.join("settings").join("site.json");
    if settings_path.exists() {
        let raw = std::fs::read_to_string(&settings_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let v: serde_json::Value = serde_json::from_str(&raw).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(v))
    } else {
        Ok(Json(serde_json::json!({})))
    }
}

// =====================================================================
// Marketing endpoints (news, plans, rankings, portfolio)
// Served by the content service since they're display-only / content-driven.
// =====================================================================

fn read_content_json(content_path: &PathBuf, rel: &str) -> Option<serde_json::Value> {
    let p = content_path.join(rel);
    if !p.exists() { return None; }
    std::fs::read_to_string(&p).ok().and_then(|s| serde_json::from_str(&s).ok())
}

async fn news_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let articles = read_content_json(&state.content_path, "marketing/news.json")
        .unwrap_or_else(|| serde_json::json!({"articles": [], "total": 0}));
    Json(articles)
}

async fn news_post(AxPath(slug): AxPath<String>, State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let pages_dir = state.content_path.join("pages");
    let candidates = ["welcome", "pricing", "subscription-vaults", "paymaster", "about"];
    let mapped = match slug.as_str() {
        "strategic-roadmap-future" | "strategic-launch-epsx" | "platform-update" => Some("welcome"),
        "integrated-service-solutions" | "platform-update-q2" | "service-tier-changes" => Some("pricing"),
        "enhanced-portfolio-management" | "portfolio-enhancements" | "new-portfolio-features" => Some("subscription-vaults"),
        "proprietary-performance-metrics" | "metrics-deep-dive" | "performance-analysis" => Some("paymaster"),
        _ => None,
    };

    let pick = mapped.and_then(|m| candidates.iter().find(|c| **c == m).copied());
    let body_html = pick.and_then(|name| {
        let p = pages_dir.join(format!("{name}.mdx"));
        std::fs::read_to_string(&p).ok()
    });

    let (title, body) = match body_html {
        Some(raw) => {
            use epsx_renderer::render_markdown;
            let trimmed = raw.trim_start_matches('\n');
            let mut title = slug.replace('-', " ");
            let body_start;
            if trimmed.starts_with("---") {
                let after = trimmed[3..].trim_start_matches('\n');
                if let Some(close) = after.find("\n---") {
                    for line in after[..close].lines() {
                        if let Some(v) = line.trim().strip_prefix("title:") {
                            title = v.trim().trim_matches('"').to_string();
                            break;
                        }
                    }
                    body_start = close + 4;
                } else {
                    body_start = 0;
                }
            } else {
                body_start = 0;
            }
            let md = if body_start > 0 {
                let after = trimmed[3..].trim_start_matches('\n');
                &after[body_start..]
            } else {
                trimmed
            };
            (title, render_markdown(md))
        }
        None => (
            slug.replace('-', " "),
            format!("<p>Article for <code>{slug}</code> coming soon.</p>"),
        ),
    };

    Ok(Json(serde_json::json!({
        "slug": slug,
        "title": title,
        "body": body,
        "published": "2026-06-09T00:00:00Z"
    })))
}

async fn plans_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(read_content_json(&state.content_path, "marketing/plans.json")
        .unwrap_or_else(|| serde_json::json!({
            "personal": [], "api": [], "custom": []
        })))
}

async fn rankings_list() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "companies": [
            { "rank": 100, "ticker": "GHC",  "price": "$5.40",  "growth": "+4650.00%", "growth_pct": 4650.00, "next_action_days": 158, "next_action_pct": 5.0,    "tradingview_url": "https://www.tradingview.com/symbols/GHC" },
            { "rank": 101, "ticker": "6535", "price": "$462.00","growth": "+4622.84%", "growth_pct": 4622.84, "next_action_days": 1,   "next_action_pct": 98.89,  "tradingview_url": "https://www.tradingview.com/symbols/6535" },
            { "rank": 102, "ticker": "4657", "price": "$427.00","growth": "+4612.47%", "growth_pct": 4612.47, "next_action_days": 65,  "next_action_pct": 27.78,  "tradingview_url": "https://www.tradingview.com/symbols/4657" }
        ],
        "as_of": "2026-06-09T00:00:00Z",
        "total": 100
    }))
}

async fn portfolio_get(AxPath(addr): AxPath<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "address": addr,
        "total_value_usd": 0.0,
        "watchlist": [],
        "subscriptions": [],
        "transactions": [],
        "auth_required": true,
        "message": "Sign in to view your portfolio"
    }))
}

mod db;
mod models;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use models::{Card, CardRecommendation, Spending, DEFAULT_CATEGORIES, DEFAULT_PAYMENT_CATEGORIES};

/// Shared application state
#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
}

/// Request body for adding a new card
#[derive(Deserialize)]
struct AddCardRequest {
    name: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    payment_categories: Vec<String>,
    miles_per_dollar: f64,
    miles_per_dollar_foreign: Option<f64>,
    block_size: f64,
    renewal_date: i32,
    max_reward_limit: Option<f64>,
    min_spend: Option<f64>,
}

/// Response after adding a card
#[derive(Serialize)]
struct AddCardResponse {
    id: i64,
    message: String,
}

/// Request body for adding spending
#[derive(Deserialize)]
struct AddSpendingRequest {
    card_id: i64,
    amount: f64,
    category: String,
    date: String,
}

/// Response after adding spending
#[derive(Serialize)]
struct AddSpendingResponse {
    id: i64,
    miles_earned: f64,
    message: String,
}

/// Query parameters for best card endpoint
#[derive(Deserialize)]
struct BestCardQuery {
    category: String,
    amount: f64,
    payment_category: String,
    #[serde(default = "default_date")]
    date: String,
}

/// Query parameters for list spending endpoint
#[derive(Deserialize)]
struct ListSpendingQuery {
    card_id: Option<i64>,
}

/// Query parameters for delete card endpoint
#[derive(Deserialize)]
struct DeleteCardQuery {
    id: i64,
}

fn default_date() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = (now / 86400) as i64;
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Convert days since Unix epoch to (year, month, day)
fn days_to_ymd(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ==================== API Handlers ====================

/// POST /api/cards - Add a new card
async fn add_card(
    State(state): State<AppState>,
    Json(payload): Json<AddCardRequest>,
) -> Result<Json<AddCardResponse>, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();

    let categories = if payload.categories.is_empty() {
        DEFAULT_CATEGORIES.iter().map(|s| s.to_string()).collect()
    } else {
        payload.categories
    };

    let payment_categories = if payload.payment_categories.is_empty() {
        DEFAULT_PAYMENT_CATEGORIES
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        payload.payment_categories
    };

    let id = db::add_card(
        &conn,
        &payload.name,
        &categories,
        &payment_categories,
        payload.miles_per_dollar,
        payload.miles_per_dollar_foreign,
        payload.block_size,
        payload.renewal_date,
        payload.max_reward_limit,
        payload.min_spend,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AddCardResponse {
        id,
        message: format!("Added card '{}'", payload.name),
    }))
}

/// GET /api/cards - List all cards
async fn list_cards(
    State(state): State<AppState>,
) -> Result<Json<Vec<Card>>, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();
    let cards = db::list_cards(&conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(cards))
}

/// DELETE /api/cards - Remove a card by ID
async fn delete_card(
    State(state): State<AppState>,
    Query(params): Query<DeleteCardQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();
    let removed = db::remove_card(&conn, params.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if removed {
        Ok((
            StatusCode::OK,
            format!("Removed card with ID {}", params.id),
        ))
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            format!("No card found with ID {}", params.id),
        ))
    }
}

/// GET /api/best-card - Find the best card for a category
async fn best_card(
    State(state): State<AppState>,
    Query(params): Query<BestCardQuery>,
) -> Result<Json<Vec<CardRecommendation>>, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();
    let results = db::best_card_for_category(
        &conn,
        &params.category,
        params.amount,
        &params.payment_category,
        &params.date,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(results))
}

/// POST /api/spending - Add a spending transaction
async fn add_spending(
    State(state): State<AppState>,
    Json(payload): Json<AddSpendingRequest>,
) -> Result<Json<AddSpendingResponse>, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();
    let (id, miles) = db::add_spending(
        &conn,
        payload.card_id,
        payload.amount,
        &payload.category,
        &payload.date,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AddSpendingResponse {
        id,
        miles_earned: miles,
        message: format!(
            "Recorded ${:.2} on card {} for '{}' — earned {:.0} miles",
            payload.amount, payload.card_id, payload.category, miles
        ),
    }))
}

/// GET /api/spending - List spending transactions
async fn list_spending(
    State(state): State<AppState>,
    Query(params): Query<ListSpendingQuery>,
) -> Result<Json<Vec<Spending>>, (StatusCode, String)> {
    let conn = state.db.lock().unwrap();
    let spending = db::list_spending(&conn, params.card_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(spending))
}

/// GET /api/health - Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cc_tracker_rust=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let conn = db::init_db().expect("Failed to initialize database");
    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/cards", post(add_card))
        .route("/api/cards", get(list_cards))
        .route("/api/cards", delete(delete_card))
        .route("/api/best-card", get(best_card))
        .route("/api/spending", post(add_spending))
        .route("/api/spending", get(list_spending))
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("🚀 Server listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

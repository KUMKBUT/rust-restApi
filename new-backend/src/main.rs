// ============================================================
//  Sweet Bananza — монолитный main.rs
//  Все секции обозначены комментариями для удобной навигации
// ============================================================

// ───────────────────────── ЗАВИСИМОСТИ ──────────────────────
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use dotenvy::dotenv;
use mongodb::{bson::oid::ObjectId, Client, options::ClientOptions};
use rand::{seq::SliceRandom, thread_rng, Rng};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;
use serde_json;

// ════════════════════════════════════════════════════════════
//  КОНСТАНТЫ
// ════════════════════════════════════════════════════════════

pub const GRID_COLS: usize = 6;
pub const GRID_ROWS: usize = 5;
pub const WIN_THRESHOLD: usize = 8;
pub const SCATTER_ID: u32 = 10;
pub const BOMB_ID: u32 = 11;
pub const BONUS_BUY_COST: Decimal = dec!(100);

// ════════════════════════════════════════════════════════════
//  КОНФИГУРАЦИЯ СИМВОЛОВ
// ════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Clone)]
pub struct SymbolConfig {
    pub id: u32,
    pub value: Decimal,
    pub weight: u32,
}

pub fn get_symbols_config() -> Vec<SymbolConfig> {
    vec![
        SymbolConfig { id: 10, value: dec!(100), weight: 2  }, // SCATTER
        SymbolConfig { id: 9,  value: dec!(50),  weight: 4  }, // HEART
        SymbolConfig { id: 8,  value: dec!(25),  weight: 6  }, // PURPLE
        SymbolConfig { id: 7,  value: dec!(15),  weight: 8  }, // GREEN
        SymbolConfig { id: 6,  value: dec!(12),  weight: 10 }, // BLUE
        SymbolConfig { id: 5,  value: dec!(10),  weight: 12 }, // APPLE
        SymbolConfig { id: 3,  value: dec!(5),   weight: 16 }, // WATERMELON
        SymbolConfig { id: 2,  value: dec!(4),   weight: 18 }, // GRAPE
        SymbolConfig { id: 1,  value: dec!(2),   weight: 20 }, // BANANA
    ]
}

// ════════════════════════════════════════════════════════════
//  СИМВОЛЫ / ЯЧЕЙКИ СЕТКИ
// ════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Clone)]
pub struct GameCell {
    pub id: u32,
    pub multiplier: Option<u32>,
    pub uid: String,
    pub is_new: bool,
}

pub fn generate_random_symbol(is_bonus: bool) -> GameCell {
    let mut rng = thread_rng();
    let uid = Uuid::new_v4().to_string()[..8].to_string();

    if is_bonus && rng.gen_bool(0.05) {
        return GameCell {
            id: BOMB_ID,
            multiplier: Some(generate_multiplier()),
            uid,
            is_new: true,
        };
    }

    let config = get_symbols_config();
    let total_weight: u32 = config.iter().map(|s| s.weight).sum();
    let mut choice = rng.gen_range(0..total_weight);

    for sym in config {
        if choice < sym.weight {
            return GameCell { id: sym.id, multiplier: None, uid, is_new: true };
        }
        choice -= sym.weight;
    }

    GameCell { id: 1, multiplier: None, uid, is_new: true }
}

fn generate_multiplier() -> u32 {
    let mut rng = thread_rng();
    let r = rng.gen_range(0..100);
    if r < 60 {
        rng.gen_range(2..6)
    } else if r < 90 {
        *[10u32, 15, 20, 25].choose(&mut rng).unwrap()
    } else if rng.gen_bool(0.5) {
        50
    } else {
        100
    }
}

// ════════════════════════════════════════════════════════════
//  ДВИЖОК ВЫПЛАТ (CASCADE / GRAVITY)
// ════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct CascadeStep {
    pub grid: Vec<Vec<GameCell>>,
    pub winning_ids: Vec<u32>,
    pub step_win: Decimal,
}

#[derive(Debug, Serialize)]
pub struct SpinResult {
    pub initial_grid: Vec<Vec<GameCell>>,
    pub cascades: Vec<CascadeStep>,
    pub total_win: Decimal,
    pub total_multiplier: u32,
    pub free_spins_won: u32,
}

pub fn process_full_round(mut grid: Vec<Vec<GameCell>>, bet: Decimal, is_bonus: bool) -> SpinResult {
    let mut cascades = Vec::new();
    let mut total_raw_win = dec!(0);
    let mut total_multiplier = 0u32;

    let initial_grid_state = grid.clone();

    for row in &grid {
        for cell in row {
            if cell.id == BOMB_ID {
                total_multiplier += cell.multiplier.unwrap_or(0);
            }
        }
    }

    loop {
        let (winning_ids, win) = calculate_wins(&grid, bet);
        if winning_ids.is_empty() {
            break;
        }

        total_raw_win += win;
        grid = apply_gravity(grid, &winning_ids, is_bonus);

        cascades.push(CascadeStep {
            grid: grid.clone(),
            winning_ids,
            step_win: win,
        });
    }

    let scatter_count = count_scatters(&grid);
    let free_spins_won = if scatter_count >= 4 { 10 } else { 0 };

    SpinResult {
        initial_grid: initial_grid_state,
        cascades,
        total_win: if total_multiplier > 0 {
            total_raw_win * Decimal::from(total_multiplier)
        } else {
            total_raw_win
        },
        total_multiplier,
        free_spins_won,
    }
}

fn calculate_wins(grid: &Vec<Vec<GameCell>>, bet: Decimal) -> (Vec<u32>, Decimal) {
    let mut counts: HashMap<u32, usize> = HashMap::new();
    for row in grid {
        for cell in row {
            if cell.id != BOMB_ID && cell.id != SCATTER_ID {
                *counts.entry(cell.id).or_insert(0) += 1;
            }
        }
    }

    let winning_ids: Vec<u32> = counts
        .into_iter()
        .filter(|&(_, count)| count >= WIN_THRESHOLD)
        .map(|(id, _)| id)
        .collect();

    let mut win = dec!(0);
    let config = get_symbols_config();
    for row in grid {
        for cell in row {
            if winning_ids.contains(&cell.id) {
                let val = config
                    .iter()
                    .find(|s| s.id == cell.id)
                    .map(|s| s.value)
                    .unwrap_or(dec!(0));
                win += val * (bet / dec!(10));
            }
        }
    }

    (winning_ids, win)
}

fn apply_gravity(
    mut grid: Vec<Vec<GameCell>>,
    winning_ids: &Vec<u32>,
    is_bonus: bool,
) -> Vec<Vec<GameCell>> {
    for c in 0..GRID_COLS {
        let mut col_items: Vec<GameCell> = Vec::new();

        for r in (0..GRID_ROWS).rev() {
            if !winning_ids.contains(&grid[r][c].id) {
                let mut cell = grid[r][c].clone();
                cell.is_new = false;
                col_items.push(cell);
            }
        }

        while col_items.len() < GRID_ROWS {
            col_items.push(generate_random_symbol(is_bonus));
        }

        for r in 0..GRID_ROWS {
            grid[r][c] = col_items[GRID_ROWS - 1 - r].clone();
        }
    }
    grid
}

fn count_scatters(grid: &Vec<Vec<GameCell>>) -> usize {
    grid.iter()
        .flat_map(|r| r.iter())
        .filter(|c| c.id == SCATTER_ID)
        .count()
}

// ════════════════════════════════════════════════════════════
//  МОДЕЛИ БАЗЫ ДАННЫХ
// ════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub external_id: String,
    pub balance: Decimal,
    pub free_spins_left: i32,
    pub is_bonus_active: bool,
    pub bonus_game: bool,
}

#[derive(Debug, Serialize)]
pub struct UserDataResponse {
    pub id: String,
    pub balance: Decimal,
    pub bonus_game: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSession {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub last_bet: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ════════════════════════════════════════════════════════════
//  РЕПОЗИТОРИЙ (доступ к MongoDB)
// ════════════════════════════════════════════════════════════

pub struct GameRepository {
    pub db: mongodb::Database,
}

impl GameRepository {
    pub fn new(db: mongodb::Database) -> Self {
        Self { db }
    }

    pub async fn get_user_balance(
        &self,
        username: &str,
    ) -> mongodb::error::Result<Option<User>> {
        let collection = self.db.collection::<User>("users");
        let filter = mongodb::bson::doc! { "username": username };
        collection.find_one(filter, None).await
    }

    pub async fn update_balance(
        &self,
        username: &str,
        amount: Decimal,
    ) -> mongodb::error::Result<()> {
        let collection = self.db.collection::<User>("users");
        let filter = mongodb::bson::doc! { "username": username };
        let update = mongodb::bson::doc! { "$inc": { "balance": amount.to_string() } };
        collection.update_one(filter, update, None).await?;
        Ok(())
    }
}

// ════════════════════════════════════════════════════════════
//  СОСТОЯНИЕ ПРИЛОЖЕНИЯ
// ════════════════════════════════════════════════════════════

pub struct AppState {
    pub db: mongodb::Database,
}

// ════════════════════════════════════════════════════════════
//  HTTP ОБРАБОТЧИКИ
// ════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct SpinPayload {
    pub bet: Decimal,
    pub is_buy_bonus: Option<bool>,
}

pub async fn spin_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SpinPayload>,
) -> Json<SpinResult> {
    let mut rng = thread_rng();
    let is_buy = payload.is_buy_bonus.unwrap_or(false);

    let mut initial_grid: Vec<Vec<GameCell>> = (0..GRID_ROWS)
        .map(|_| (0..GRID_COLS).map(|_| generate_random_symbol(false)).collect())
        .collect();

    if is_buy {
        let mut positions: Vec<(usize, usize)> = (0..GRID_ROWS)
            .flat_map(|r| (0..GRID_COLS).map(move |c| (r, c)))
            .collect();
        positions.shuffle(&mut rng);

        let scatter_count = rng.gen_range(4..=6);
        for i in 0..scatter_count {
            let (r, c) = positions[i];
            initial_grid[r][c] = GameCell {
                id: SCATTER_ID,
                multiplier: None,
                uid: Uuid::new_v4().to_string()[..8].to_string(),
                is_new: true,
            };
        }
    }

    let result = process_full_round(initial_grid, payload.bet, is_buy);
    Json(result)
}

pub async fn get_data_handler(

    State(state): State<Arc<AppState>>,

    headers: HeaderMap,
) -> Result<Json<UserDataResponse>, (StatusCode, Json<serde_json::Value>)> {

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing Authorization header" })),
            )
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid Authorization format, expected: Bearer <token>" })),
            )
        })?;

    let collection = state.db.collection::<User>("users");

    let filter = mongodb::bson::doc! { "external_id": token };

    let maybe_user = collection
        .find_one(filter, None)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database error: {}", e) })),
            )
        })?;

    let user = match maybe_user {

        Some(existing) => existing,

        None => {
            let new_user = User {
                id: None,
                external_id: token.to_string(),
                balance: dec!(1000),
                free_spins_left: 0,
                is_bonus_active: false,
                bonus_game: false,
            };

            let result = collection
                .insert_one(new_user.clone(), None)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": format!("Failed to create user: {}", e) })),
                    )
                })?;

            let inserted_id = result.inserted_id.as_object_id().unwrap_or_default();

            User {
                id: Some(inserted_id),
                ..new_user 
            }
        }
    };

    let response = UserDataResponse {
        id: user.id.unwrap_or_default().to_hex(),
        balance: user.balance,
        bonus_game: user.bonus_game,
    };

    Ok(Json(response))
}

// ════════════════════════════════════════════════════════════
//  ТОЧКА ВХОДА
// ════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mongo_uri = std::env::var("MONGO_URI")
        .expect("MONGO_URI must be set in .env or environment");
    let db_name = std::env::var("DATABASE_NAME")
        .unwrap_or_else(|_| "sweet_bananza".to_string());

    let client_options = ClientOptions::parse(mongo_uri).await?;
    let client = Client::with_options(client_options)?;
    let db = client.database(&db_name);

    let shared_state = Arc::new(AppState { db });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "Sweet Bananza API is running!" }))
        .route("/api/spin", post(spin_handler))
        .route("/api/data", get(get_data_handler)) // ← новый роут
        .layer(cors)
        .with_state(shared_state);

    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    println!("Server started at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
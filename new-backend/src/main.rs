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
use rand::prelude::*;
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

    // Суммируем множители со всех бомб на начальной сетке
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

        // Собираем выжившие ячейки снизу вверх
        for r in (0..GRID_ROWS).rev() {
            if !winning_ids.contains(&grid[r][c].id) {
                let mut cell = grid[r][c].clone();
                cell.is_new = false;
                col_items.push(cell);
            }
        }

        // Добавляем новые символы сверху
        while col_items.len() < GRID_ROWS {
            col_items.push(generate_random_symbol(is_bonus));
        }

        // Записываем обратно (индекс 0 — верхняя строка)
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
    pub external_id: String, // значение из Bearer token (например, "1")
    pub balance: Decimal,
    pub free_spins_left: i32,
    pub is_bonus_active: bool,
    pub bonus_game: bool,       // ← новое поле: находится ли игрок в бонусной игре
}

// Этот struct описывает ЧТО именно мы вернём клиенту в ответе /api/data.
// Мы не отдаём весь User целиком — только нужные поля.
// Derive(Serialize) — значит Rust умеет автоматически превратить это в JSON.
#[derive(Debug, Serialize)]
pub struct UserDataResponse {
    pub id: String,       // _id пользователя в виде строки
    pub balance: Decimal, // текущий баланс
    pub bonus_game: bool, // статус бонусной игры
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

// ─────────────────────────────────────────────────────────────
//  GET /api/data
//
//  Клиент присылает заголовок:  Authorization: Bearer <token>
//  Мы берём <token>, ищем пользователя у которого external_id == token,
//  и возвращаем его id, balance и bonus_game.
//
//  Возвращаемый тип:  Result<Json<...>, (StatusCode, Json<...>)>
//  Это означает: либо успех (200 + JSON), либо ошибка (код + JSON с сообщением).
// ─────────────────────────────────────────────────────────────
pub async fn get_data_handler(
    // State — это наше общее состояние приложения (содержит подключение к БД).
    // Arc<AppState> — умный указатель, позволяет безопасно делить состояние
    // между несколькими запросами одновременно.
    State(state): State<Arc<AppState>>,

    // HeaderMap — это map всех HTTP-заголовков запроса.
    // Из него мы достанем Authorization.
    headers: HeaderMap,
) -> Result<Json<UserDataResponse>, (StatusCode, Json<serde_json::Value>)> {

    // ── Шаг 1: извлекаем токен из заголовка Authorization ──────

    // .get("authorization") — ищем заголовок (axum приводит имена к lowercase).
    // Если заголовка нет — возвращаем ошибку 401 Unauthorized.
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok()) // конвертируем байты в &str
        .ok_or_else(|| {
            // ok_or_else: если None — превращаем в Err с нашим ответом об ошибке
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing Authorization header" })),
            )
        })?; // "?" — если Err, сразу выходим из функции и возвращаем эту ошибку

    // Заголовок выглядит как "Bearer abc123".
    // .strip_prefix("Bearer ") убирает префикс и оставляет только токен "abc123".
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid Authorization format, expected: Bearer <token>" })),
            )
        })?;

    // ── Шаг 2: ищем пользователя в MongoDB ─────────────────────

    // Получаем коллекцию "users" и указываем что документы там — это User.
    let collection = state.db.collection::<User>("users");

    // doc! — макрос для создания BSON-документа (формат запроса MongoDB).
    // Ищем документ где поле external_id равно нашему токену.
    let filter = mongodb::bson::doc! { "external_id": token };

    // .await — ждём асинхронного результата от БД.
    // Результат: Ok(Some(user)) | Ok(None) | Err(e)
    let maybe_user = collection
        .find_one(filter, None)
        .await
        .map_err(|e| {
            // map_err: если БД вернула ошибку — конвертируем её в HTTP 500
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database error: {}", e) })),
            )
        })?;

    // ── Шаг 3: если пользователь не найден — создаём его ──────

    // match — это как switch в других языках, но мощнее.
    // Мы проверяем два варианта: Some(user) — нашли, None — не нашли.
    let user = match maybe_user {
        // Пользователь уже есть в БД — просто возвращаем его
        Some(existing) => existing,

        // Пользователя нет — создаём нового с дефолтными значениями
        None => {
            let new_user = User {
                id: None, // None означает что MongoDB сам сгенерирует _id
                external_id: token.to_string(),
                balance: dec!(1000), // стартовый баланс
                free_spins_left: 0,
                is_bonus_active: false,
                bonus_game: false,
            };

            // .insert_one() — вставляем документ в коллекцию.
            // Возвращает InsertOneResult у которого есть inserted_id.
            let result = collection
                .insert_one(new_user.clone(), None)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": format!("Failed to create user: {}", e) })),
                    )
                })?;

            // inserted_id имеет тип Bson — нам нужно достать из него ObjectId.
            // as_object_id() возвращает Option<ObjectId>, unwrap_or_default() на случай если что-то пошло не так.
            let inserted_id = result.inserted_id.as_object_id().unwrap_or_default();

            // Возвращаем того же нового пользователя но уже с присвоенным id
            User {
                id: Some(inserted_id),
                ..new_user // ".." означает "остальные поля взять из new_user"
            }
        }
    };

    // ── Шаг 4: формируем и отправляем ответ ────────────────────

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
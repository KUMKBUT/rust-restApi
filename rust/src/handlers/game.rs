use axum::{extract::State, Json};
use crate::AppState;
use crate::engine::payout::{process_full_round, SpinResult};
use crate::engine::symbols::{generate_random_symbol, GameCell};
use crate::engine::constant::*;
use serde::Deserialize;
use std::sync::Arc;
use rust_decimal::Decimal;
use rand::seq::SliceRandom; // Не забудь добавить rand в Cargo.toml

#[derive(Deserialize)]
pub struct SpinPayload {
    pub bet: Decimal,
}

pub async fn spin_handler(
    State(_state): State<Arc<AppState>>, // Добавили подчеркивание, чтобы убрать warning
    Json(payload): Json<SpinPayload>,
) -> Json<SpinResult> {
    let initial_grid: Vec<Vec<GameCell>> = (0..GRID_ROWS)
        .map(|_| (0..GRID_COLS).map(|_| generate_random_symbol(false)).collect())
        .collect();

    let result = process_full_round(initial_grid, payload.bet, false);
    Json(result)
}

pub async fn buy_bonus_handler(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SpinPayload>,
) -> Json<SpinResult> {
    let mut initial_grid: Vec<Vec<GameCell>> = (0..GRID_ROWS)
        .map(|_| (0..GRID_COLS).map(|_| generate_random_symbol(false)).collect())
        .collect();

    // Реализация логики из твоего useGameLogic.js: расставляем 4 скаттера
    let mut positions: Vec<(usize, usize)> = Vec::new();
    for r in 0..GRID_ROWS {
        for c in 0..GRID_COLS {
            positions.push((r, c));
        }
    }
    
    let mut rng = rand::thread_rng();
    positions.shuffle(&mut rng);

    for i in 0..4 {
        let (r, c) = positions[i];
        initial_grid[r][c] = GameCell {
            id: SCATTER_ID,
            multiplier: None,
            uid: uuid::Uuid::new_v4().to_string()[..8].to_string(),
        };
    }

    let result = process_full_round(initial_grid, payload.bet, true);
    Json(result)
}
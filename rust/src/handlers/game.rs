use axum::{extract::State, Json};
use axum::http::{HeaderMap, StatusCode};
use crate::AppState;
use crate::engine::payout::{process_full_round, SpinResult};
use crate::engine::symbols::{generate_random_symbol, GameCell};
use crate::engine::constant::*;
use serde::Deserialize;
use std::sync::Arc;
use rust_decimal::Decimal;
use rand::{thread_rng, Rng, seq::SliceRandom};

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
        let mut positions: Vec<(usize, usize)> = Vec::new();
        for r in 0..GRID_ROWS {
            for c in 0..GRID_COLS {
                positions.push((r, c));
            }
        }
        positions.shuffle(&mut rng);

        let scatter_count = rng.gen_range(4..=6); 
        for i in 0..scatter_count {
            let (r, c) = positions[i];
            initial_grid[r][c] = GameCell {
                id: SCATTER_ID,
                multiplier: None,
                uid: uuid::Uuid::new_v4().to_string()[..8].to_string(),
                is_new: true,
            };
        }
    }

    let result = process_full_round(initial_grid, payload.bet, is_buy);
    Json(result)
}
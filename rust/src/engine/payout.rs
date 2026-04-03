use crate::engine::constant::*;
use crate::engine::symbols::{GameCell, generate_random_symbol};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Serialize;

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
    let mut total_multiplier = 0;

    for row in &grid {
        for cell in row {
            if cell.id == BOMB_ID {
                total_multiplier += cell.multiplier.unwrap_or(0);
            }
        }
    }

    loop {
        let (winning_ids, win) = calculate_wins(&grid, bet);
        if winning_ids.is_empty() { break; }

        total_raw_win += win;
        let prev_grid = grid.clone();

        grid = apply_gravity(grid, &winning_ids, is_bonus);
        
        cascades.push(CascadeStep {
            grid: prev_grid,
            winning_ids,
            step_win: win,
        });
    }

    let scatter_count = count_scatters(&grid);
    let free_spins_won = if scatter_count >= 4 { 10 } else { 0 };

    SpinResult {
        initial_grid: grid,
        cascades,
        total_win: if total_multiplier > 0 { total_raw_win * Decimal::from(total_multiplier) } else { total_raw_win },
        total_multiplier,
        free_spins_won,
    }
}

fn calculate_wins(grid: &Vec<Vec<GameCell>>, bet: Decimal) -> (Vec<u32>, Decimal) {
    let mut counts = std::collections::HashMap::new();
    for row in grid {
        for cell in row {
            if cell.id != BOMB_ID && cell.id != SCATTER_ID {
                *counts.entry(cell.id).or_insert(0) += 1;
            }
        }
    }

    let winning_ids: Vec<u32> = counts.into_iter()
        .filter(|&(_, count)| count >= WIN_THRESHOLD)
        .map(|(id, _)| id)
        .collect();

    let mut win = dec!(0);
    let config = get_symbols_config();
    for row in grid {
        for cell in row {
            if winning_ids.contains(&cell.id) {
                let val = config.iter().find(|s| s.id == cell.id).map(|s| s.value).unwrap_or(dec!(0));
                win += val * (bet / dec!(10));
            }
        }
    }
    (winning_ids, win)
}

fn apply_gravity(mut grid: Vec<Vec<GameCell>>, winning_ids: &Vec<u32>, is_bonus: bool) -> Vec<Vec<GameCell>> {
    for c in 0..GRID_COLS {
        let mut col_items = Vec::new();
        for r in (0..GRID_ROWS).rev() {
            if !winning_ids.contains(&grid[r][c].id) {
                col_items.push(grid[r][c].clone());
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
    grid.iter().flat_map(|r| r.iter()).filter(|c| c.id == SCATTER_ID).count()
}

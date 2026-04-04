use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Serialize;

pub const GRID_COLS: usize = 6;
pub const GRID_ROWS: usize = 5;
pub const WIN_THRESHOLD: usize = 8;
pub const SCATTER_ID: u32 = 10;
pub const BOMB_ID: u32 = 11;
pub const BONUS_BUY_COST: Decimal = dec!(100);

#[derive(Debug, Serialize, Clone)]
pub struct SymbolConfig {
    pub id: u32,
    pub value: Decimal,
    pub weight: u32,
}

pub fn get_symbols_config() -> Vec<SymbolConfig> {
    vec![
        SymbolConfig { id: 10, value: dec!(100), weight: 2 },  // SCATTER
        SymbolConfig { id: 9,  value: dec!(50),  weight: 4 },  // HEART
        SymbolConfig { id: 8,  value: dec!(25),  weight: 6 },  // PURPLE
        SymbolConfig { id: 7,  value: dec!(15),  weight: 8 },  // GREEN
        SymbolConfig { id: 6,  value: dec!(12),  weight: 10 }, // BLUE
        SymbolConfig { id: 5,  value: dec!(10),  weight: 12 }, // APPLE
        SymbolConfig { id: 3,  value: dec!(5),   weight: 16 }, // WATERMELON
        SymbolConfig { id: 2,  value: dec!(4),   weight: 18 }, // GRAPE
        SymbolConfig { id: 1,  value: dec!(2),   weight: 20 }, // BANANA
    ]
}

use rand::prelude::*;
use crate::engine::constant::*;
use serde::Serialize;
use uuid::Uuid;

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
        return GameCell { id: BOMB_ID, multiplier: Some(generate_multiplier()), uid, is_new: true};
    }

    let config = get_symbols_config();
    let total_weight: u32 = config.iter().map(|s| s.weight).sum();
    let mut choice = rng.gen_range(0..total_weight);

    for sym in config {
        if choice < sym.weight {
            return GameCell { id: sym.id, multiplier: None, uid, is_new: true};
        }
        choice -= sym.weight;
    }
    GameCell { id: 1, multiplier: None, uid , is_new: true}
}

fn generate_multiplier() -> u32 {
    let mut rng = thread_rng();
    let r = rng.gen_range(0..100);
    if r < 60 { rng.gen_range(2..6) }
    else if r < 90 { *vec![10, 15, 20, 25].choose(&mut rng).unwrap() }
    else { if rng.gen_bool(0.5) { 50 } else { 100 } }
}

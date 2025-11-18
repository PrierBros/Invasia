use serde::{Deserialize, Serialize};

use super::ai_entity::{AiEntity, AiState};

pub const SNAPSHOT_FIELD_COUNT: usize = 7;

#[derive(Clone, Copy)]
pub struct EntitySnapshot {
    pub position_x: f32,
    pub position_y: f32,
    pub state: AiState,
    pub military_strength: f32,
}

impl From<&AiEntity> for EntitySnapshot {
    fn from(entity: &AiEntity) -> Self {
        Self {
            position_x: entity.position_x,
            position_y: entity.position_y,
            state: entity.state,
            military_strength: entity.military_strength,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicEntitySnapshot {
    pub id: u32,
    pub military_strength: f32,
    pub position_x: f32,
    pub position_y: f32,
    pub state: AiState,
    pub territory: u32,
    pub money: f32,
}

pub type SimulationSnapshot = Vec<PublicEntitySnapshot>;

impl From<&AiEntity> for PublicEntitySnapshot {
    fn from(entity: &AiEntity) -> Self {
        Self {
            id: entity.id,
            military_strength: entity.military_strength,
            position_x: entity.position_x,
            position_y: entity.position_y,
            state: entity.state,
            territory: entity.territory,
            money: entity.money,
        }
    }
}

use crate::types::{AiEntity, AiState, EntitySnapshot};

use super::grid_update_builder::GridUpdateBuilder;

pub struct AiStateUpdater;

impl AiStateUpdater {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_entity(
        &mut self,
        entity: &mut AiEntity,
        _tick: u64,
        self_index: usize,
        self_snapshot: EntitySnapshot,
        entity_snapshots: &[EntitySnapshot],
        grid: &GridUpdateBuilder,
    ) {
        if entity.state == AiState::Dead {
            return;
        }

        let mut variation = entity.next_variation();
        if variation < 0.25 {
            variation = 0.25;
        }

        match entity.state {
            AiState::Active => {
                entity.military_strength = (entity.military_strength - 0.3 * variation).max(0.0);
                if entity.military_strength < 20.0 {
                    entity.state = AiState::Resting;
                }
            }
            AiState::Resting => {
                entity.military_strength = (entity.military_strength + 1.0 * variation).min(100.0);
                if entity.military_strength > 80.0 {
                    entity.state = AiState::Moving;
                }
            }
            AiState::Moving => {
                entity.military_strength = (entity.military_strength - 0.2 * variation).max(0.0);

                let movement_x = entity.random_symmetric() * 2.0 * variation;
                let movement_y = entity.random_symmetric() * 2.0 * variation;

                let new_x = entity.position_x + movement_x;
                let new_y = entity.position_y + movement_y;

                const WORLD_BOUND: f32 = 1230.0;
                entity.position_x = new_x.clamp(-WORLD_BOUND, WORLD_BOUND);
                entity.position_y = new_y.clamp(-WORLD_BOUND, WORLD_BOUND);

                if entity.military_strength > 60.0 {
                    let expansion_rate = (entity.military_strength / 100.0) * 0.1 * variation;
                    entity.territory = (entity.territory + expansion_rate).min(100.0);
                }

                if entity.military_strength < 50.0 {
                    entity.state = AiState::Active;
                }
            }
            AiState::Idle => {
                entity.military_strength = (entity.military_strength + 0.1 * variation).min(100.0);
                if entity.military_strength > 90.0 {
                    entity.state = AiState::Active;
                }
            }
            AiState::Dead => {
                return;
            }
        }

        let mut total_damage = 0.0;
        grid.for_each_neighbor(
            self_snapshot.position_x,
            self_snapshot.position_y,
            |other_index| {
                if other_index == self_index {
                    return;
                }
                debug_assert!(other_index < entity_snapshots.len());
                let other = unsafe { entity_snapshots.get_unchecked(other_index) };
                debug_assert_eq!(
                    other.state,
                    AiState::Active,
                    "Spatial grid should only contain Active entities",
                );

                let dx = entity.position_x - other.position_x;
                let dy = entity.position_y - other.position_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < 100.0 && dist_sq > 0.01 {
                    let damage = (other.military_strength / 100.0) * 0.5 * variation;
                    total_damage += damage;
                }
            },
        );

        if total_damage > 0.0 {
            entity.health = (entity.health - total_damage).max(0.0);
        } else if entity.health < 100.0 {
            entity.health = (entity.health + 0.05 * variation).min(100.0);
        }
    }
}

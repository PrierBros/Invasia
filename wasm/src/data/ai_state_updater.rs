use crate::types::{AiEntity, AiState, EntitySnapshot};

use super::grid_update_builder::GridUpdateBuilder;

// Resource generation rates per grid space per second
const MILITARY_STRENGTH_PER_SPACE_PER_SEC: f32 = 0.5;
const MONEY_PER_SPACE_PER_SEC: f32 = 1.0;
const ATTACK_COST_MILITARY: f32 = 10.0; // Cost to attempt conquering a grid space
const DEFEND_BONUS_MULTIPLIER: f32 = 1.5; // Defense bonus when defending

pub struct AiStateUpdater {
    current_time: f64,
}

impl AiStateUpdater {
    pub fn new() -> Self {
        Self { current_time: 0.0 }
    }

    pub fn update_time(&mut self, time_ms: f64) {
        self.current_time = time_ms;
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

        // Time-based resource accumulation (decoupled from tick rate)
        let time_delta_sec = if entity.last_update_time > 0.0 {
            (self.current_time - entity.last_update_time) / 1000.0 // Convert ms to seconds
        } else {
            0.0
        };
        entity.last_update_time = self.current_time;

        if time_delta_sec > 0.0 && entity.territory > 0 {
            // Generate resources based on owned territory and elapsed time
            let territory_count = entity.territory as f32;
            let time_delta_sec_f32 = time_delta_sec as f32;
            entity.military_strength += MILITARY_STRENGTH_PER_SPACE_PER_SEC * territory_count * time_delta_sec_f32;
            entity.money += MONEY_PER_SPACE_PER_SEC * territory_count * time_delta_sec_f32;
        }

        // AI decision making - choose action based on current state and resources
        let mut variation = entity.next_variation();
        if variation < 0.25 {
            variation = 0.25;
        }

        // Check for nearby enemies
        let mut nearest_enemy_idx: Option<usize> = None;
        let mut nearest_enemy_dist_sq = f32::INFINITY;

        grid.for_each_neighbor(
            self_snapshot.position_x,
            self_snapshot.position_y,
            |other_index| {
                if other_index == self_index {
                    return;
                }
                debug_assert!(other_index < entity_snapshots.len());
                let other = unsafe { entity_snapshots.get_unchecked(other_index) };
                
                // Only consider active (attacking) entities as threats
                if other.state != AiState::Attacking && other.state != AiState::Defending {
                    return;
                }

                let dx = entity.position_x - other.position_x;
                let dy = entity.position_y - other.position_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < nearest_enemy_dist_sq {
                    nearest_enemy_dist_sq = dist_sq;
                    nearest_enemy_idx = Some(other_index);
                }
            },
        );

        // Simple AI logic: attack if strong enough, defend if threatened, otherwise idle
        match entity.state {
            AiState::Idle => {
                if let Some(_enemy_idx) = nearest_enemy_idx {
                    if nearest_enemy_dist_sq < 10000.0 {
                        // Enemy nearby, decide to defend or attack
                        if entity.military_strength >= ATTACK_COST_MILITARY * 2.0 {
                            entity.state = AiState::Attacking;
                        } else {
                            entity.state = AiState::Defending;
                        }
                    }
                } else if entity.military_strength >= ATTACK_COST_MILITARY * 3.0 {
                    // Strong enough to attack
                    entity.state = AiState::Attacking;
                }
            }
            AiState::Attacking => {
                // Attacking consumes military strength
                if entity.military_strength < ATTACK_COST_MILITARY {
                    entity.state = AiState::Idle; // Not enough strength to continue attacking
                }
            }
            AiState::Defending => {
                // Stop defending if no nearby threats
                if nearest_enemy_idx.is_none() || nearest_enemy_dist_sq > 20000.0 {
                    entity.state = AiState::Idle;
                }
            }
            AiState::Dead => {
                return;
            }
        }
    }
}

use crate::constants::{ATTACK_COST, MILITARY_STRENGTH_PER_SPACE_PER_SEC, MONEY_PER_SPACE_PER_SEC};
use crate::types::{AiEntity, AiState, EntitySnapshot};

use super::grid_update_builder::GridUpdateBuilder;

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

        // AI decision making - greedy territory expansion while considering defense
        
        // Check for nearby enemies and threats
        let mut nearest_enemy_idx: Option<usize> = None;
        let mut nearest_enemy_dist_sq = f32::INFINITY;
        let mut nearby_attackers = 0;

        grid.for_each_neighbor(
            self_snapshot.position_x,
            self_snapshot.position_y,
            |other_index| {
                if other_index == self_index {
                    return;
                }
                debug_assert!(other_index < entity_snapshots.len());
                let other = unsafe { entity_snapshots.get_unchecked(other_index) };
                
                let dx = entity.position_x - other.position_x;
                let dy = entity.position_y - other.position_y;
                let dist_sq = dx * dx + dy * dy;

                // Count nearby attacking entities as immediate threats
                if other.state == AiState::Attacking && dist_sq < 5000.0 {
                    nearby_attackers += 1;
                }

                // Track nearest enemy for defensive purposes
                if other.state != AiState::Dead && dist_sq < nearest_enemy_dist_sq {
                    nearest_enemy_dist_sq = dist_sq;
                    nearest_enemy_idx = Some(other_index);
                }
            },
        );

        // Greedy AI logic: prioritize attacking to gain territory
        match entity.state {
            AiState::Idle => {
                // Be aggressive: attack if we have enough resources
                // Consider defense needs if under immediate threat
                if nearby_attackers > 0 && entity.military_strength < ATTACK_COST * 2.0 {
                    // Under threat and low on resources, defend
                    entity.state = AiState::Defending;
                } else if entity.military_strength >= ATTACK_COST {
                    // Greedy: attack whenever we have the minimum cost
                    // This ensures AIs actively try to expand their territory
                    entity.state = AiState::Attacking;
                } else if nearby_attackers > 0 {
                    // Not enough to attack but under threat, defend
                    entity.state = AiState::Defending;
                }
                // Otherwise stay idle and accumulate resources
            }
            AiState::Attacking => {
                // Continue attacking as long as we have resources
                if entity.military_strength < ATTACK_COST {
                    // Out of resources, switch to defending or idle
                    if nearby_attackers > 0 {
                        entity.state = AiState::Defending;
                    } else {
                        entity.state = AiState::Idle;
                    }
                }
            }
            AiState::Defending => {
                // Transition from defending to attacking when safe and strong enough
                if nearby_attackers == 0 && entity.military_strength >= ATTACK_COST * 1.5 {
                    // No immediate threats and good resources, go on offense
                    entity.state = AiState::Attacking;
                } else if entity.military_strength < ATTACK_COST * 0.5 {
                    // Very low on resources, stay idle to accumulate
                    entity.state = AiState::Idle;
                }
                // Otherwise keep defending if there are nearby threats
                if nearby_attackers == 0 && nearest_enemy_dist_sq > 15000.0 {
                    entity.state = AiState::Idle;
                }
            }
            AiState::Dead => {
                return;
            }
        }
    }
}

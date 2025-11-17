use crate::data::{AiState, SimulationData};
use crate::dependency::performance_now;

pub struct SimulationLogic;

impl Default for SimulationLogic {
    fn default() -> Self {
        Self
    }
}

impl SimulationLogic {
    pub fn step(&mut self, data: &mut SimulationData) {
        let start = performance_now();

        data.tick = data.tick.wrapping_add(1);
        data.rebuild_snapshot_buffer();
        let snapshots = data.snapshots().to_vec();
        data.grid.rebuild(&snapshots);

        let entity_count = data.entity_len();
        for i in 0..entity_count {
            if let Some(entity) = data.entities.get_mut(i) {
                let snapshot = snapshots[i];
                entity.update(data.tick, i, snapshot, &snapshots, &data.grid);
            }
        }

        data.resource_transfers.clear();
        data.dead_indices.clear();

        for i in 0..entity_count {
            let (state, health, pos_x, pos_y, military_strength, money) = {
                let entity = &data.entities()[i];
                (
                    entity.state,
                    entity.health,
                    entity.position_x,
                    entity.position_y,
                    entity.military_strength,
                    entity.money,
                )
            };

            if health <= 0.0 && state != AiState::Dead {
                data.dead_indices.push(i);

                if military_strength > 0.0 || money > 0.0 {
                    let mut nearest_attacker_idx: Option<usize> = None;
                    let mut nearest_dist_sq = f32::INFINITY;

                    data.grid.for_each_neighbor(pos_x, pos_y, |idx| {
                        if idx == i {
                            return;
                        }
                        let other = &data.entities()[idx];

                        if matches!(other.state, AiState::Active) {
                            let dx = pos_x - other.position_x;
                            let dy = pos_y - other.position_y;
                            let dist_sq = dx * dx + dy * dy;

                            if dist_sq < nearest_dist_sq {
                                nearest_dist_sq = dist_sq;
                                nearest_attacker_idx = Some(idx);
                            }
                        }
                    });

                    if let Some(attacker_idx) = nearest_attacker_idx {
                        data.resource_transfers
                            .push((attacker_idx, military_strength, money));
                    }
                }
            }
        }

        for &(attacker_idx, military_strength, money) in &data.resource_transfers {
            if let Some(attacker) = data.entities.get_mut(attacker_idx) {
                attacker.military_strength += military_strength;
                attacker.money += money;
            }
        }

        for &dead_idx in &data.dead_indices {
            if let Some(dead_entity) = data.entities.get_mut(dead_idx) {
                dead_entity.state = AiState::Dead;
                dead_entity.health = 0.0;
                dead_entity.military_strength = 0.0;
                dead_entity.money = 0.0;
                dead_entity.territory = 0.0;
            }
        }

        data.mark_snapshots_dirty();

        let end = performance_now();
        if start > 0.0 && end >= start {
            data.last_tick_duration_ms = end - start;
        }
    }
}

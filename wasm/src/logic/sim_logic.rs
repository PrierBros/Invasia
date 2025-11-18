use crate::constants::{ATTACK_COST, DEFENSE_ACCUMULATION, DEFENSE_BONUS_MULTIPLIER, MAX_DEFENSE_STRENGTH};
use crate::data::{
    AiNeighborBuilder, AiStateUpdater, BenchmarkMetricBuilder, GridUpdateBuilder, SimulationData,
};
use crate::types::{AiState, SimulationSnapshot};
use std::mem;
use std::time::Instant;

pub struct SimulationLogic {
    data: SimulationData,
    neighbor_builder: AiNeighborBuilder,
    state_updater: AiStateUpdater,
    grid_builder: GridUpdateBuilder,
    benchmark_builder: BenchmarkMetricBuilder,
    start_time: Instant,
}

impl SimulationLogic {
    pub fn new(entity_count: usize) -> Self {
        Self {
            data: SimulationData::new(entity_count),
            neighbor_builder: AiNeighborBuilder::new(),
            state_updater: AiStateUpdater::new(),
            grid_builder: GridUpdateBuilder::new(5.0, 10.0),
            benchmark_builder: BenchmarkMetricBuilder::new(),
            start_time: Instant::now(),
        }
    }

    pub fn step(&mut self) {
        self.data.increment_tick();
        let current_tick = self.data.tick();
        
        // Use actual wall clock time for time-based resource generation
        let elapsed = self.start_time.elapsed();
        let current_time_ms = elapsed.as_millis() as f64;
        self.state_updater.update_time(current_time_ms);
        
        let (_, duration) = self.benchmark_builder.measure_tick(|| {
            self.neighbor_builder.rebuild_snapshots(&mut self.data);
            let snapshots = self.data.snapshots().to_vec();
            self.grid_builder.rebuild(&snapshots);

            let entity_count = self.data.entity_len();
            for i in 0..entity_count {
                if let Some(entity) = self.data.entity_mut(i) {
                    let snapshot = snapshots[i];
                    self.state_updater.update_entity(
                        entity,
                        current_tick,
                        i,
                        snapshot,
                        &snapshots,
                        &self.grid_builder,
                    );
                }
            }
        });

        // Process conquests - attackers try to conquer adjacent grid spaces
        self.process_conquests();

        self.data.reset_tick_buffers();

        // Check for AIs that lost all territory (death condition)
        let entity_count = self.data.entity_len();
        for i in 0..entity_count {
            let (state, territory, military_strength, money) = {
                let entity = self.data.entity(i).expect("entity must exist");
                (
                    entity.state,
                    entity.territory,
                    entity.military_strength,
                    entity.money,
                )
            };

            // AI dies when it loses all its territory
            if territory == 0 && state != AiState::Dead {
                self.data.dead_indices_mut().push(i);

                // Transfer remaining resources to nearest attacker
                if military_strength > 0.0 || money > 0.0 {
                    let (pos_x, pos_y) = {
                        let entity = self.data.entity(i).expect("entity must exist");
                        (entity.position_x, entity.position_y)
                    };
                    
                    let mut nearest_attacker_idx: Option<usize> = None;
                    let mut nearest_dist_sq = f32::INFINITY;

                    self.grid_builder.for_each_neighbor(pos_x, pos_y, |idx| {
                        if idx == i {
                            return;
                        }

                        if let Some(other) = self.data.entity(idx) {
                            if matches!(other.state, AiState::Attacking) {
                                let dx = pos_x - other.position_x;
                                let dy = pos_y - other.position_y;
                                let dist_sq = dx * dx + dy * dy;

                                if dist_sq < nearest_dist_sq {
                                    nearest_dist_sq = dist_sq;
                                    nearest_attacker_idx = Some(idx);
                                }
                            }
                        }
                    });

                    if let Some(attacker_idx) = nearest_attacker_idx {
                        self.data.resource_transfers_mut().push((
                            attacker_idx,
                            military_strength,
                            money,
                        ));
                    }
                }
            }
        }

        let mut transfers = mem::take(self.data.resource_transfers_mut());
        for &(attacker_idx, military_strength, money) in &transfers {
            if let Some(attacker) = self.data.entity_mut(attacker_idx) {
                attacker.military_strength += military_strength;
                attacker.money += money;
            }
        }
        transfers.clear();
        *self.data.resource_transfers_mut() = transfers;

        let mut dead_indices = mem::take(self.data.dead_indices_mut());
        for &dead_idx in &dead_indices {
            if let Some(dead_entity) = self.data.entity_mut(dead_idx) {
                dead_entity.state = AiState::Dead;
                dead_entity.military_strength = 0.0;
                dead_entity.money = 0.0;
                dead_entity.territory = 0;
            }
        }
        dead_indices.clear();
        *self.data.dead_indices_mut() = dead_indices;

        // Update territory counts based on owned grid spaces
        self.data.update_territories();

        self.data.mark_snapshots_dirty();

        if duration > 0.0 {
            self.data.metrics_mut().update_tick(duration);
        }

        // Check if simulation should end (only one AI alive)
        if self.is_complete() {
            self.data.set_running(false);
        }
    }

    pub fn update(&mut self) {
        if self.data.running() {
            self.step();
        }
    }

    pub fn is_complete(&self) -> bool {
        let alive_count = self.count_alive();
        alive_count <= 1
    }

    pub fn count_alive(&self) -> usize {
        self.data
            .entities()
            .iter()
            .filter(|e| e.state != AiState::Dead)
            .count()
    }

    pub fn start(&mut self) {
        self.data.set_running(true);
    }

    pub fn pause(&mut self) {
        self.data.set_running(false);
    }

    pub fn resume(&mut self) {
        self.start();
    }

    pub fn reset(&mut self) {
        self.data.set_running(false);
        self.data.reset_entities();
    }

    pub fn running(&self) -> bool {
        self.data.running()
    }

    pub fn tick(&self) -> u64 {
        self.data.tick()
    }

    pub fn tick_rate(&self) -> u32 {
        self.data.tick_rate()
    }

    pub fn set_tick_rate(&mut self, tick_rate: u32) {
        self.data.set_tick_rate(tick_rate);
    }

    pub fn entity_count(&self) -> usize {
        self.data.entity_len()
    }

    pub fn set_entity_count(&mut self, entity_count: usize) {
        self.data.set_entity_count(entity_count);
    }

    pub fn grid_size(&self) -> usize {
        self.data.grid_size()
    }

    pub fn set_grid_size(&mut self, grid_size: usize) {
        self.data.set_grid_size(grid_size);
    }

    /// Process conquest attempts by attacking AIs
    fn process_conquests(&mut self) {
        let grid_size = self.data.grid_size();
        let entity_count = self.data.entity_len();
        
        // First, defenders add to defense strength of their grid spaces
        let mut defense_updates = Vec::new();
        for i in 0..entity_count {
            if let Some(entity) = self.data.entity(i) {
                if entity.state == AiState::Defending {
                    if let Some(grid_idx) = self.data.position_to_grid_index(entity.position_x, entity.position_y) {
                        defense_updates.push((grid_idx, entity.id));
                    }
                }
            }
        }
        
        // Apply defense updates
        for (grid_idx, entity_id) in defense_updates {
            if let Some(space) = self.data.grid_space_mut(grid_idx) {
                if space.owner_id == Some(entity_id) {
                    space.defense_strength += DEFENSE_ACCUMULATION;
                    // Cap defense strength
                    space.defense_strength = space.defense_strength.min(MAX_DEFENSE_STRENGTH);
                }
            }
        }
        
        // Collect all attacking entities
        let mut attackers = Vec::new();
        for i in 0..entity_count {
            if let Some(entity) = self.data.entity(i) {
                if entity.state == AiState::Attacking && entity.military_strength >= ATTACK_COST {
                    attackers.push((i, entity.id, entity.military_strength));
                }
            }
        }
        
        // Build a list of (grid_idx, owner_id, defense_strength) to avoid borrowing issues
        let grid_data: Vec<(Option<u32>, f32)> = self.data.grid_spaces()
            .iter()
            .map(|space| (space.owner_id, space.defense_strength))
            .collect();
        
        // For each attacker, try to conquer an adjacent grid space
        // Check adjacency to ALL owned spaces, not just the spawn position
        for (attacker_idx, attacker_id, military_strength) in attackers {
            let mut conquered = false;
            
            // Find all grid spaces owned by this attacker
            for grid_idx in 0..grid_data.len() {
                if conquered {
                    break;
                }
                
                let (owner_id, _) = grid_data[grid_idx];
                if owner_id != Some(attacker_id) {
                    continue; // Not owned by this attacker
                }
                
                // Try to conquer adjacent spaces
                let row = grid_idx / grid_size;
                let col = grid_idx % grid_size;
                
                // Check adjacent cells (4-directional)
                let adjacent_offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                
                for (dr, dc) in adjacent_offsets {
                    if conquered {
                        break;
                    }
                    
                    let new_row = row as i32 + dr;
                    let new_col = col as i32 + dc;
                    
                    if new_row < 0 || new_row >= grid_size as i32 || new_col < 0 || new_col >= grid_size as i32 {
                        continue;
                    }
                    
                    let target_grid_idx = (new_row as usize) * grid_size + (new_col as usize);
                    
                    // Check if this space is owned by a different AI or unowned
                    let (target_owner_id, target_defense_strength) = grid_data[target_grid_idx];
                    let (can_attack, total_defense) = if let Some(defender_id) = target_owner_id {
                        if defender_id != attacker_id {
                            let defense = ATTACK_COST + target_defense_strength * DEFENSE_BONUS_MULTIPLIER;
                            (military_strength >= defense, defense)
                        } else {
                            (false, 0.0) // Own space
                        }
                    } else {
                        // Unowned space
                        (military_strength >= ATTACK_COST, ATTACK_COST)
                    };
                    
                    if can_attack {
                        // Conquest successful! Transfer ownership
                        if let Some(target_space) = self.data.grid_space_mut(target_grid_idx) {
                            target_space.owner_id = Some(attacker_id);
                            target_space.defense_strength = 5.0;
                        }
                        
                        // Deduct cost from attacker
                        if let Some(attacker) = self.data.entity_mut(attacker_idx) {
                            attacker.military_strength -= total_defense;
                        }
                        
                        conquered = true;
                    }
                }
            }
        }
    }

    pub fn request_snapshot(&mut self) -> Option<SimulationSnapshot> {
        if !self.data.snapshot_dirty() {
            return None;
        }

        let (snapshot, duration) = self
            .benchmark_builder
            .measure_snapshot(|| self.data.build_public_snapshot());
        if duration > 0.0 {
            self.data.metrics_mut().update_snapshot(duration);
        }
        Some(snapshot)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn request_flat_snapshot(&mut self) -> Option<&[f32]> {
        if !self.data.flat_snapshot_dirty() {
            return Some(self.data.flat_snapshot_slice());
        }

        let (_, duration) = self.benchmark_builder.measure_snapshot(|| {
            self.data.ensure_flat_snapshot_ready();
        });
        if duration > 0.0 {
            self.data.metrics_mut().update_snapshot(duration);
        }
        Some(self.data.flat_snapshot_slice())
    }

    pub fn last_tick_duration(&self) -> f64 {
        self.data.metrics().last_tick_duration_ms
    }

    pub fn last_snapshot_duration(&self) -> f64 {
        self.data.metrics().last_snapshot_duration_ms
    }

    pub fn destroy(&mut self) {
        self.data.destroy();
    }

    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut SimulationData {
        &mut self.data
    }
}

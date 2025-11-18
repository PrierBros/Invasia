use crate::data::{
    AiNeighborBuilder, AiStateUpdater, BenchmarkMetricBuilder, GridUpdateBuilder, SimulationData,
};
use crate::types::{AiState, SimulationSnapshot};
use std::mem;

pub struct SimulationLogic {
    data: SimulationData,
    neighbor_builder: AiNeighborBuilder,
    state_updater: AiStateUpdater,
    grid_builder: GridUpdateBuilder,
    benchmark_builder: BenchmarkMetricBuilder,
}

impl SimulationLogic {
    pub fn new(entity_count: usize) -> Self {
        Self {
            data: SimulationData::new(entity_count),
            neighbor_builder: AiNeighborBuilder::new(),
            state_updater: AiStateUpdater::new(),
            grid_builder: GridUpdateBuilder::new(5.0, 10.0),
            benchmark_builder: BenchmarkMetricBuilder::new(),
        }
    }

    pub fn step(&mut self) {
        let (_, duration) = self.benchmark_builder.measure_tick(|| {
            self.data.increment_tick();
            let current_tick = self.data.tick();
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

            self.data.reset_tick_buffers();

            for i in 0..entity_count {
                let (state, health, pos_x, pos_y, military_strength, money) = {
                    let entity = self.data.entity(i).expect("entity must exist");
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
                    self.data.dead_indices_mut().push(i);

                    if military_strength > 0.0 || money > 0.0 {
                        let mut nearest_attacker_idx: Option<usize> = None;
                        let mut nearest_dist_sq = f32::INFINITY;

                        self.grid_builder.for_each_neighbor(pos_x, pos_y, |idx| {
                            if idx == i {
                                return;
                            }

                            if let Some(other) = self.data.entity(idx) {
                                if matches!(other.state, AiState::Active) {
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
                    dead_entity.health = 0.0;
                    dead_entity.military_strength = 0.0;
                    dead_entity.money = 0.0;
                    dead_entity.territory = 0.0;
                }
            }
            dead_indices.clear();
            *self.data.dead_indices_mut() = dead_indices;

            self.data.mark_snapshots_dirty();
        });

        if duration > 0.0 {
            self.data.metrics_mut().update_tick(duration);
        }
    }

    pub fn update(&mut self) {
        if self.data.running() {
            self.step();
        }
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

        let (slice, duration) = self.benchmark_builder.measure_snapshot(|| {
            self.data.ensure_flat_snapshot_ready();
            self.data.flat_snapshot_slice()
        });
        if duration > 0.0 {
            self.data.metrics_mut().update_snapshot(duration);
        }
        Some(slice)
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
}

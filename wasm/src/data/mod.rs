mod entities;

pub use entities::{AiEntity, AiState, EntitySnapshot};

use crate::dependency::{EntityStore, FlatSnapshotCache, SnapshotBuffer, SpatialGrid};

pub const SNAPSHOT_FIELD_COUNT: usize = 8;

pub struct SimulationData {
    pub tick: u64,
    pub running: bool,
    pub entity_count: usize,
    pub tick_rate: u32,
    pub entities: EntityStore,
    pub grid: SpatialGrid,
    pub snapshot_buffer: SnapshotBuffer,
    pub resource_transfers: Vec<(usize, f32, f32)>,
    pub dead_indices: Vec<usize>,
    pub last_tick_duration_ms: f64,
    pub last_snapshot_duration_ms: f64,
    pub snapshot_dirty: bool,
    pub flat_snapshot_dirty: bool,
    pub flat_snapshot: FlatSnapshotCache,
}

impl SimulationData {
    pub fn new(entity_count: usize) -> Self {
        Self {
            tick: 0,
            running: false,
            entity_count,
            tick_rate: 60,
            entities: EntityStore::new(entity_count),
            grid: SpatialGrid::new(5.0, 10.0),
            snapshot_buffer: SnapshotBuffer::with_capacity(entity_count),
            resource_transfers: Vec::with_capacity(128),
            dead_indices: Vec::with_capacity(128),
            last_tick_duration_ms: 0.0,
            last_snapshot_duration_ms: 0.0,
            snapshot_dirty: true,
            flat_snapshot_dirty: true,
            flat_snapshot: FlatSnapshotCache::with_capacity(entity_count * SNAPSHOT_FIELD_COUNT),
        }
    }

    pub fn reset_entities(&mut self) {
        self.tick = 0;
        self.running = false;
        self.entities.rebuild(self.entity_count);
        self.grid.clear();
        self.snapshot_buffer = SnapshotBuffer::with_capacity(self.entity_count);
        self.resource_transfers.clear();
        self.dead_indices.clear();
        self.flat_snapshot.clear();
        self.snapshot_dirty = true;
        self.flat_snapshot_dirty = true;
    }

    pub fn destroy(&mut self) {
        self.running = false;
        self.entities.clear();
        self.tick = 0;
        self.grid.clear();
        self.snapshot_buffer.clear();
        self.resource_transfers.clear();
        self.dead_indices.clear();
        self.flat_snapshot.clear();
        self.flat_snapshot_dirty = true;
        self.snapshot_dirty = true;
    }

    pub fn rebuild_snapshot_buffer(&mut self) {
        self.snapshot_buffer.rebuild(self.entities.as_slice());
    }

    pub fn snapshots(&self) -> &[EntitySnapshot] {
        self.snapshot_buffer.as_slice()
    }

    pub fn entity_len(&self) -> usize {
        self.entities.len()
    }

    pub fn entities(&self) -> &[AiEntity] {
        self.entities.as_slice()
    }

    pub fn mark_snapshots_dirty(&mut self) {
        self.snapshot_dirty = true;
        self.flat_snapshot_dirty = true;
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn ensure_flat_snapshot_capacity(&mut self) {
        let required_len = self.entity_len() * SNAPSHOT_FIELD_COUNT;
        self.flat_snapshot.resize(required_len);
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn rebuild_flat_snapshot(&mut self) {
        self.ensure_flat_snapshot_capacity();
        let data = self.flat_snapshot.data_mut();
        for (i, entity) in self.entities.as_slice().iter().enumerate() {
            let base = i * SNAPSHOT_FIELD_COUNT;
            data[base] = entity.id as f32;
            data[base + 1] = entity.health;
            data[base + 2] = entity.military_strength;
            data[base + 3] = entity.money;
            data[base + 4] = entity.territory;
            let state_value: u32 = entity.state.into();
            data[base + 5] = state_value as f32;
            data[base + 6] = entity.position_x;
            data[base + 7] = entity.position_y;
        }
        self.flat_snapshot_dirty = false;
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn ensure_flat_snapshot_ready(&mut self) {
        if self.flat_snapshot_dirty {
            self.rebuild_flat_snapshot();
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn flat_snapshot_slice(&self) -> &[f32] {
        self.flat_snapshot.data()
    }
}

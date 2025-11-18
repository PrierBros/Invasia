mod ai_neighbor_builder;
mod ai_state_updater;
mod benchmark_metric_builder;
mod grid_update_builder;

pub use ai_neighbor_builder::AiNeighborBuilder;
pub use ai_state_updater::AiStateUpdater;
pub use benchmark_metric_builder::BenchmarkMetricBuilder;
pub use grid_update_builder::GridUpdateBuilder;

use crate::types::{
    AiEntity, BenchmarkMetrics, EntitySnapshot, GridSpace, PublicEntitySnapshot,
    SimulationSnapshot, SNAPSHOT_FIELD_COUNT,
};

pub struct SimulationData {
    tick: u64,
    running: bool,
    tick_rate: u32,
    entity_count: usize,
    grid_size: usize,  // Width/height of the grid
    entities: Vec<AiEntity>,
    grid_spaces: Vec<GridSpace>, // Flattened 2D grid
    snapshot_buffer: Vec<EntitySnapshot>,
    flat_snapshot: Vec<f32>,
    snapshot_dirty: bool,
    flat_snapshot_dirty: bool,
    resource_transfers: Vec<(usize, f32, f32)>,
    dead_indices: Vec<usize>,
    metrics: BenchmarkMetrics,
}

impl SimulationData {
    pub fn new(entity_count: usize) -> Self {
        Self::with_grid_size(entity_count, 50) // Default grid size 50x50
    }

    pub fn with_grid_size(entity_count: usize, grid_size: usize) -> Self {
        let total_grid_spaces = grid_size * grid_size;
        let mut data = Self {
            tick: 0,
            running: false,
            tick_rate: 60,
            entity_count,
            grid_size,
            entities: Vec::with_capacity(entity_count),
            grid_spaces: vec![GridSpace::new(); total_grid_spaces],
            snapshot_buffer: Vec::with_capacity(entity_count),
            flat_snapshot: Vec::with_capacity(entity_count * SNAPSHOT_FIELD_COUNT),
            snapshot_dirty: true,
            flat_snapshot_dirty: true,
            resource_transfers: Vec::with_capacity(128),
            dead_indices: Vec::with_capacity(128),
            metrics: BenchmarkMetrics::default(),
        };
        data.rebuild_entities(entity_count);
        data
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn increment_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn tick_rate(&self) -> u32 {
        self.tick_rate
    }

    pub fn set_tick_rate(&mut self, tick_rate: u32) {
        self.tick_rate = tick_rate;
    }

    pub fn entity_len(&self) -> usize {
        self.entities.len()
    }

    pub fn reset_entities(&mut self) {
        let count = self.entity_count;
        self.rebuild_entities(count);
    }

    pub fn rebuild_entities(&mut self, entity_count: usize) {
        self.entities.clear();
        
        // Reset grid spaces
        for space in &mut self.grid_spaces {
            *space = GridSpace::new();
        }
        
        // Assign each AI a deterministic starting grid space based on even distribution
        for i in 0..entity_count {
            let mut entity = AiEntity::new(i as u32);
            
            // Find an unoccupied grid space for this AI
            // Use deterministic placement based on entity ID
            let grid_area = self.grid_size * self.grid_size;
            let spacing = (grid_area as f32 / entity_count as f32).sqrt().floor() as usize;
            let spacing = spacing.max(1);
            
            let divisor = (self.grid_size / spacing).max(1);
            let row = (i / divisor) * spacing;
            let col = (i % divisor) * spacing;
            let grid_index = (row.min(self.grid_size - 1)) * self.grid_size + col.min(self.grid_size - 1);
            
            // Assign this grid space to the AI, ensuring uniqueness
            let mut assigned_index = grid_index;
            let mut found = false;
            for offset in 0..self.grid_spaces.len() {
                let idx = (grid_index + offset) % self.grid_spaces.len();
                if self.grid_spaces[idx].owner.is_none() {
                    assigned_index = idx;
                    found = true;
                    break;
                }
            }
            if found {
                self.grid_spaces[assigned_index] = GridSpace::with_owner(entity.id, 5.0);
                
                // Update entity position to be centered in their grid space
                let assigned_row = assigned_index / self.grid_size;
                let assigned_col = assigned_index % self.grid_size;
                let grid_x = (assigned_col as f32 + 0.5) * (2400.0 / self.grid_size as f32) - 1200.0;
                let grid_y = (assigned_row as f32 + 0.5) * (2400.0 / self.grid_size as f32) - 1200.0;
                entity.position_x = grid_x;
                entity.position_y = grid_y;
            }
            
            self.entities.push(entity);
        }
        
        self.entity_count = entity_count;
        self.snapshot_buffer = Vec::with_capacity(entity_count);
        self.flat_snapshot = Vec::with_capacity(entity_count * SNAPSHOT_FIELD_COUNT);
        self.snapshot_dirty = true;
        self.flat_snapshot_dirty = true;
        self.tick = 0;
    }

    pub fn entity_mut(&mut self, index: usize) -> Option<&mut AiEntity> {
        self.entities.get_mut(index)
    }

    pub fn entity(&self, index: usize) -> Option<&AiEntity> {
        self.entities.get(index)
    }

    pub fn entities(&self) -> &[AiEntity] {
        &self.entities
    }

    pub fn resource_transfers_mut(&mut self) -> &mut Vec<(usize, f32, f32)> {
        &mut self.resource_transfers
    }

    pub fn dead_indices_mut(&mut self) -> &mut Vec<usize> {
        &mut self.dead_indices
    }

    pub fn mark_snapshots_dirty(&mut self) {
        self.snapshot_dirty = true;
        self.flat_snapshot_dirty = true;
    }

    pub fn reset_tick_buffers(&mut self) {
        self.resource_transfers.clear();
        self.dead_indices.clear();
    }

    pub fn snapshot_dirty(&self) -> bool {
        self.snapshot_dirty
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn flat_snapshot_dirty(&self) -> bool {
        self.flat_snapshot_dirty
    }

    pub fn rebuild_snapshot_buffer(&mut self) {
        self.snapshot_buffer.clear();
        for entity in &self.entities {
            self.snapshot_buffer.push(EntitySnapshot::from(entity));
        }
    }

    pub fn snapshots(&self) -> &[EntitySnapshot] {
        &self.snapshot_buffer
    }

    pub fn metrics(&self) -> &BenchmarkMetrics {
        &self.metrics
    }

    pub fn metrics_mut(&mut self) -> &mut BenchmarkMetrics {
        &mut self.metrics
    }

    pub fn build_public_snapshot(&mut self) -> SimulationSnapshot {
        self.snapshot_dirty = false;
        self.entities
            .iter()
            .map(PublicEntitySnapshot::from)
            .collect()
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn ensure_flat_snapshot_ready(&mut self) {
        if self.flat_snapshot_dirty {
            self.rebuild_flat_snapshot();
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn flat_snapshot_slice(&self) -> &[f32] {
        &self.flat_snapshot
    }

    pub fn destroy(&mut self) {
        self.running = false;
        self.entities.clear();
        self.snapshot_buffer.clear();
        self.flat_snapshot.clear();
        self.resource_transfers.clear();
        self.dead_indices.clear();
        self.snapshot_dirty = true;
        self.flat_snapshot_dirty = true;
        self.tick = 0;
    }

    pub fn set_entity_count(&mut self, entity_count: usize) {
        self.rebuild_entities(entity_count);
    }

    pub fn grid_size(&self) -> usize {
        self.grid_size
    }

    pub fn set_grid_size(&mut self, grid_size: usize) {
        self.grid_size = grid_size;
        let total_grid_spaces = grid_size * grid_size;
        self.grid_spaces.resize(total_grid_spaces, GridSpace::new());
        self.rebuild_entities(self.entity_count);
    }

    pub fn grid_spaces(&self) -> &[GridSpace] {
        &self.grid_spaces
    }

    pub fn grid_space_mut(&mut self, index: usize) -> Option<&mut GridSpace> {
        self.grid_spaces.get_mut(index)
    }

    pub fn position_to_grid_index(&self, x: f32, y: f32) -> Option<usize> {
        // Convert world coordinates (-1200 to 1200) to grid coordinates
        let grid_x = ((x + 1200.0) / 2400.0 * self.grid_size as f32).floor() as i32;
        let grid_y = ((y + 1200.0) / 2400.0 * self.grid_size as f32).floor() as i32;
        
        if grid_x < 0 || grid_x >= self.grid_size as i32 || grid_y < 0 || grid_y >= self.grid_size as i32 {
            return None;
        }
        
        Some((grid_y as usize) * self.grid_size + (grid_x as usize))
    }

    /// Update all entities' territory counts based on owned grid spaces
    pub fn update_territories(&mut self) {
        // Reset all territory counts
        for entity in &mut self.entities {
            entity.territory = 0;
        }
        
        // Count owned grid spaces for each entity
        for space in &self.grid_spaces {
            if let Some(owner_id) = space.owner_id {
                // Find the entity with this ID
                if let Some(entity) = self.entities.iter_mut().find(|e| e.id == owner_id) {
                    entity.territory += 1;
                }
            }
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn rebuild_flat_snapshot(&mut self) {
        let required_len = self.entity_len() * SNAPSHOT_FIELD_COUNT;
        if self.flat_snapshot.len() != required_len {
            self.flat_snapshot.resize(required_len, 0.0);
        }
        for (i, entity) in self.entities.iter().enumerate() {
            let base = i * SNAPSHOT_FIELD_COUNT;
            self.flat_snapshot[base] = entity.id as f32;
            self.flat_snapshot[base + 1] = entity.military_strength;
            self.flat_snapshot[base + 2] = entity.money;
            self.flat_snapshot[base + 3] = entity.territory as f32;
            let state_value: u32 = entity.state.into();
            self.flat_snapshot[base + 4] = state_value as f32;
            self.flat_snapshot[base + 5] = entity.position_x;
            self.flat_snapshot[base + 6] = entity.position_y;
        }
        self.flat_snapshot_dirty = false;
    }
}

impl Default for SimulationData {
    fn default() -> Self {
        Self::new(0)
    }
}

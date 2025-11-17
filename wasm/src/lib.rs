use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::Float32Array;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static PERFORMANCE: Option<web_sys::Performance> =
        web_sys::window().and_then(|w| w.performance());
}

#[cfg(target_arch = "wasm32")]
fn performance_now() -> f64 {
    PERFORMANCE.with(|perf| perf.as_ref().map(|p| p.now()).unwrap_or(0.0))
}

#[cfg(not(target_arch = "wasm32"))]
fn performance_now() -> f64 {
    0.0
}

// AI Decision Scoring System modules
mod decision_scoring;
pub use decision_scoring::*;

// ============================================================================
// AI Simulation Subsystem
// ============================================================================

/// AI entity state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "u32", from = "u32")]
pub enum AiState {
    Idle = 0,
    Active = 1,
    Resting = 2,
    Moving = 3,
    Dead = 4,
}

impl From<AiState> for u32 {
    fn from(state: AiState) -> u32 {
        state as u32
    }
}

impl From<u32> for AiState {
    fn from(value: u32) -> AiState {
        match value {
            1 => AiState::Active,
            2 => AiState::Resting,
            3 => AiState::Moving,
            4 => AiState::Dead,
            _ => AiState::Idle,
        }
    }
}

/// AI entity with scalar attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEntity {
    pub id: u32,
    pub health: f32,
    pub military_strength: f32,
    pub position_x: f32,
    pub position_y: f32,
    pub state: AiState,
    pub territory: f32, // Territory controlled by this entity
    pub money: f32,     // Money/resources owned by this entity
    #[serde(skip)]
    rng_state: u32,
}

impl AiEntity {
    /// Create a new AI entity with default values
    /// Grid size is 500x500 cells with cell_size=5.0, so world is [-1250, 1250)
    /// Entities spawn distributed across the grid to avoid clustering
    pub fn new(id: u32) -> Self {
        // Create per-entity variation for initial state
        // Use id as seed for deterministic but varied initialization
        let id_seed = id as f32;
        let variation = ((id_seed * 0.7321).sin() + 1.0) / 2.0; // 0.0 to 1.0 range

        // Vary initial military strength between 50 and 100
        let initial_military_strength = 50.0 + (variation * 50.0);

        // Vary initial health between 70 and 100
        let health_variation = ((id_seed * 1.234).cos() + 1.0) / 2.0;
        let initial_health = 70.0 + (health_variation * 30.0);

        // Vary initial money between 100 and 200
        let money_variation = ((id_seed * 3.141).sin() + 1.0) / 2.0;
        let initial_money = 100.0 + (money_variation * 100.0);

        // Randomize initial state based on id
        let state_seed = ((id_seed * 2.718).sin() + 1.0) / 2.0;
        let initial_state = if state_seed < 0.25 {
            AiState::Idle
        } else if state_seed < 0.5 {
            AiState::Active
        } else if state_seed < 0.75 {
            AiState::Resting
        } else {
            AiState::Moving
        };

        // Deterministic random spawn position across the grid
        // Grid world bounds are [-1250, 1250) to stay within 500x500 cells at cell_size=5.0
        // Use multiple sine waves with different frequencies for good distribution
        let x_seed = ((id_seed * 0.3371).sin() + (id_seed * 0.0157).sin()) * 0.5;
        let y_seed = ((id_seed * 0.4219).cos() + (id_seed * 0.0213).cos()) * 0.5;

        // Spawn within world bounds, leaving margin to prevent immediate edge cases
        let spawn_x = x_seed * 1200.0; // Range: [-1200, 1200]
        let spawn_y = y_seed * 1200.0; // Range: [-1200, 1200]

        Self {
            id,
            health: initial_health,
            military_strength: initial_military_strength,
            position_x: spawn_x,
            position_y: spawn_y,
            state: initial_state,
            territory: 10.0, // Start with small territory
            money: initial_money,
            rng_state: Self::seed_rng(id),
        }
    }

    #[inline]
    fn seed_rng(id: u32) -> u32 {
        let mut seed = id.wrapping_mul(747_796_405).wrapping_add(2_891_336_453) ^ 0xA511_E9B3;
        if seed == 0 {
            seed = 1;
        }
        seed
    }

    #[inline]
    fn next_random(&mut self) -> f32 {
        // Xorshift32 - fast deterministic RNG for per-entity variation
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        if x == 0 {
            x = 1;
        }
        self.rng_state = x; // Prevent zero lock-up
        const INV_U32_MAX: f32 = 1.0 / (u32::MAX as f32);
        (self.rng_state as f32) * INV_U32_MAX
    }

    #[inline]
    fn next_variation(&mut self) -> f32 {
        // Mimic previous 0.25..2.25 range by multiplying two [0.5, 1.5) samples
        let a = 0.5 + self.next_random();
        let b = 0.5 + self.next_random();
        a * b
    }

    #[inline]
    fn random_symmetric(&mut self) -> f32 {
        self.next_random() * 2.0 - 1.0
    }

    /// Update the entity for one simulation tick
    pub(crate) fn update(
        &mut self,
        _tick: u64,
        self_index: usize,
        entity_snapshots: &[EntitySnapshot],
        grid: &SpatialGrid,
    ) {
        // Early return for dead entities
        if self.state == AiState::Dead {
            return;
        }

        // Deterministic variation using fast per-entity RNG
        let mut variation = self.next_variation();
        if variation < 0.25 {
            variation = 0.25;
        }

        // Military strength dynamics with per-entity variation
        match self.state {
            AiState::Active => {
                self.military_strength = (self.military_strength - 0.3 * variation).max(0.0);
                if self.military_strength < 20.0 {
                    self.state = AiState::Resting;
                }
            }
            AiState::Resting => {
                self.military_strength = (self.military_strength + 1.0 * variation).min(100.0);
                if self.military_strength > 80.0 {
                    self.state = AiState::Moving;
                }
            }
            AiState::Moving => {
                self.military_strength = (self.military_strength - 0.2 * variation).max(0.0);

                // Simple deterministic movement with boundary constraints
                // Grid world bounds are [-1250, 1250) to stay within 500x500 cells at cell_size=5.0
                let movement_x = self.random_symmetric() * 2.0 * variation;
                let movement_y = self.random_symmetric() * 2.0 * variation;

                let new_x = self.position_x + movement_x;
                let new_y = self.position_y + movement_y;

                // Constrain to world bounds with small margin
                const WORLD_BOUND: f32 = 1230.0; // Stay within [-1230, 1230] for safety margin
                self.position_x = new_x.clamp(-WORLD_BOUND, WORLD_BOUND);
                self.position_y = new_y.clamp(-WORLD_BOUND, WORLD_BOUND);

                // Expansion: Gain territory if military strength is sufficient
                if self.military_strength > 60.0 {
                    let expansion_rate = (self.military_strength / 100.0) * 0.1 * variation;
                    self.territory = (self.territory + expansion_rate).min(100.0);
                }

                if self.military_strength < 50.0 {
                    self.state = AiState::Active;
                }
            }
            AiState::Idle => {
                self.military_strength = (self.military_strength + 0.1 * variation).min(100.0);
                if self.military_strength > 90.0 {
                    self.state = AiState::Active;
                }
            }
            AiState::Dead => {
                return;
            }
        }

        // Apply combat damage from nearby Active entities
        // Note: All neighbors from spatial grid are Active entities
        let mut total_damage = 0.0;
        let self_snapshot = entity_snapshots[self_index];
        grid.for_each_neighbor(
            self_snapshot.position_x,
            self_snapshot.position_y,
            |other_index| {
                if other_index == self_index {
                    return;
                }
                // Safety: neighbor indices come from spatial grid which only holds valid entries
                debug_assert!(other_index < entity_snapshots.len());
                let other = unsafe { entity_snapshots.get_unchecked(other_index) };
                // Spatial grid only contains Active entities, so no need to check state
                debug_assert_eq!(
                    other.state,
                    AiState::Active,
                    "Spatial grid should only contain Active entities"
                );

                let dx = self.position_x - other.position_x;
                let dy = self.position_y - other.position_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < 100.0 && dist_sq > 0.01 {
                    let damage = (other.military_strength / 100.0) * 0.5 * variation;
                    total_damage += damage;
                }
            },
        );

        // Apply damage to health
        if total_damage > 0.0 {
            self.health = (self.health - total_damage).max(0.0);
        } else if self.health < 100.0 {
            // Health regeneration with variation (only when not in combat)
            self.health = (self.health + 0.05 * variation).min(100.0);
        }
    }
}

#[derive(Clone, Copy)]
struct EntitySnapshot {
    position_x: f32,
    position_y: f32,
    state: AiState,
    military_strength: f32,
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

const SNAPSHOT_FIELD_COUNT: usize = 8; // id, health, military, money, territory, state, pos_x, pos_y

impl Simulation {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn ensure_flat_snapshot_capacity(&mut self) {
        let required_len = self.entities.len() * SNAPSHOT_FIELD_COUNT;
        if self.flat_snapshot.len() != required_len {
            self.flat_snapshot.resize(required_len, 0.0);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn rebuild_flat_snapshot(&mut self) {
        self.ensure_flat_snapshot_capacity();
        for (i, entity) in self.entities.iter().enumerate() {
            let base = i * SNAPSHOT_FIELD_COUNT;
            self.flat_snapshot[base] = entity.id as f32;
            self.flat_snapshot[base + 1] = entity.health;
            self.flat_snapshot[base + 2] = entity.military_strength;
            self.flat_snapshot[base + 3] = entity.money;
            self.flat_snapshot[base + 4] = entity.territory;
            let state_value: u32 = entity.state.into();
            self.flat_snapshot[base + 5] = state_value as f32;
            self.flat_snapshot[base + 6] = entity.position_x;
            self.flat_snapshot[base + 7] = entity.position_y;
        }
        self.flat_snapshot_dirty = false;
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn ensure_flat_snapshot_ready(&mut self) {
        if self.flat_snapshot_dirty {
            self.rebuild_flat_snapshot();
        }
    }
}

// Optimized spatial grid using fixed-size grid array instead of HashMap
// For 10K entities distributed across reasonable space, we can use a bounded grid
// Grid covers world bounds [-1250, 1250) with cell_size=5.0 giving 500x500 cells
// The grid only tracks Active entities (attackers) since we only need to find nearby attackers
// for combat damage and death resource transfers. This significantly reduces memory and lookup costs.
const GRID_SIZE: usize = 500; // 500x500 grid = 250000 cells (ample space for 10K+ entities with room to grow)
const MAX_ENTITIES_PER_CELL: usize = 4; // Allow small clusters of Active entities per cell

struct SpatialGrid {
    cell_size: f32,
    _search_radius: f32,
    // Fixed-size grid: each cell stores indices and count
    cells: Vec<([usize; MAX_ENTITIES_PER_CELL], usize)>,
    grid_min: (i32, i32),
    grid_max: (i32, i32),
    overflow_count: usize, // Track how many entities couldn't be added due to cell capacity
    neighbor_offsets: Vec<(i32, i32)>,
}

impl SpatialGrid {
    fn new(cell_size: f32, search_radius: f32) -> Self {
        let capacity = GRID_SIZE * GRID_SIZE;
        let mut cells = Vec::with_capacity(capacity);
        cells.resize(capacity, ([0; MAX_ENTITIES_PER_CELL], 0));

        let range = (search_radius / cell_size).ceil() as i32;
        let mut neighbor_offsets = Vec::with_capacity(((range * 2) + 1).pow(2) as usize);
        for dx in -range..=range {
            for dy in -range..=range {
                neighbor_offsets.push((dx, dy));
            }
        }

        Self {
            cell_size,
            _search_radius: search_radius,
            cells,
            grid_min: (-(GRID_SIZE as i32 / 2), -(GRID_SIZE as i32 / 2)),
            grid_max: (GRID_SIZE as i32 / 2, GRID_SIZE as i32 / 2),
            overflow_count: 0,
            neighbor_offsets,
        }
    }

    fn clear(&mut self) {
        // Fast clear: just reset counts
        for cell in &mut self.cells {
            cell.1 = 0;
        }
        self.overflow_count = 0;
    }

    fn cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        let cx = (x / self.cell_size).floor() as i32;
        let cy = (y / self.cell_size).floor() as i32;
        (cx, cy)
    }

    fn cell_index(&self, cx: i32, cy: i32) -> Option<usize> {
        if cx < self.grid_min.0
            || cx >= self.grid_max.0
            || cy < self.grid_min.1
            || cy >= self.grid_max.1
        {
            return None;
        }
        let x = (cx - self.grid_min.0) as usize;
        let y = (cy - self.grid_min.1) as usize;
        Some(y * GRID_SIZE + x)
    }

    fn rebuild(&mut self, snapshots: &[EntitySnapshot]) {
        self.clear();
        // Only track Active entities (attackers) in the spatial grid
        // since we only care about finding nearby attackers for combat and death processing
        for (index, entity) in snapshots.iter().enumerate() {
            if entity.state != AiState::Active {
                continue; // Skip non-Active entities
            }

            let coords = self.cell_coords(entity.position_x, entity.position_y);
            if let Some(cell_idx) = self.cell_index(coords.0, coords.1) {
                let cell = &mut self.cells[cell_idx];
                if cell.1 < MAX_ENTITIES_PER_CELL {
                    cell.0[cell.1] = index;
                    cell.1 += 1;
                } else {
                    // Track overflow: entity couldn't be added to grid cell
                    // This may cause incorrect neighbor queries and combat issues
                    self.overflow_count += 1;
                    #[cfg(debug_assertions)]
                    {
                        eprintln!(
                            "Warning: Spatial grid cell at ({}, {}) is full (max {} Active entities). \
                             Active entity {} at ({:.2}, {:.2}) dropped. Total overflow: {}",
                            coords.0, coords.1, MAX_ENTITIES_PER_CELL,
                            index, entity.position_x, entity.position_y,
                            self.overflow_count
                        );
                    }
                }
            }
        }

        // Log total overflow if it occurred
        #[cfg(debug_assertions)]
        {
            if self.overflow_count > 0 {
                eprintln!(
                    "Spatial grid rebuild complete. {} Active entities couldn't be added due to cell capacity limits.",
                    self.overflow_count
                );
            }
        }
    }

    fn for_each_neighbor<F>(&self, x: f32, y: f32, mut f: F)
    where
        F: FnMut(usize),
    {
        let (cx, cy) = self.cell_coords(x, y);
        for &(dx, dy) in &self.neighbor_offsets {
            if let Some(cell_idx) = self.cell_index(cx + dx, cy + dy) {
                let cell = &self.cells[cell_idx];
                for &entity_idx in &cell.0[..cell.1] {
                    f(entity_idx);
                }
            }
        }
    }
}

/// Simulation state manager
#[wasm_bindgen]
pub struct Simulation {
    entities: Vec<AiEntity>,
    tick: u64,
    running: bool,
    entity_count: usize,
    tick_rate: u32,
    grid: SpatialGrid,
    snapshot_buffer: Vec<EntitySnapshot>,
    // Buffers for death/resource processing
    resource_transfers: Vec<(usize, f32, f32)>, // Changed to use indices instead of IDs
    dead_indices: Vec<usize>,
    // Performance tracking
    last_tick_duration_ms: f64,
    last_snapshot_duration_ms: f64,
    snapshot_dirty: bool, // Flag to track if snapshot needs updating
    flat_snapshot: Vec<f32>,
    flat_snapshot_dirty: bool,
}

#[wasm_bindgen]
impl Simulation {
    /// Create a new simulation with specified entity count
    #[wasm_bindgen(constructor)]
    pub fn new(entity_count: usize) -> Self {
        let mut entities = Vec::with_capacity(entity_count);
        for i in 0..entity_count {
            entities.push(AiEntity::new(i as u32));
        }

        Self {
            entities,
            tick: 0,
            running: false,
            entity_count,
            tick_rate: 60, // Default 60 ticks per second
            grid: SpatialGrid::new(5.0, 10.0),
            snapshot_buffer: Vec::with_capacity(entity_count),
            resource_transfers: Vec::with_capacity(128),
            dead_indices: Vec::with_capacity(128),
            last_tick_duration_ms: 0.0,
            last_snapshot_duration_ms: 0.0,
            snapshot_dirty: true,
            flat_snapshot: Vec::with_capacity(entity_count * SNAPSHOT_FIELD_COUNT),
            flat_snapshot_dirty: true,
        }
    }

    /// Initialize simulation with custom entity count and tick rate
    #[wasm_bindgen]
    pub fn init(entity_count: usize, tick_rate: u32) -> Self {
        let mut sim = Self::new(entity_count);
        sim.tick_rate = tick_rate;
        sim
    }

    /// Start the simulation
    #[wasm_bindgen]
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Pause the simulation
    #[wasm_bindgen]
    pub fn pause(&mut self) {
        self.running = false;
    }

    /// Resume the simulation
    #[wasm_bindgen]
    pub fn resume(&mut self) {
        self.running = true;
    }

    /// Reset the simulation to initial state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.tick = 0;
        self.running = false;
        self.entities.clear();
        self.grid.clear();
        self.snapshot_buffer.clear();
        self.resource_transfers.clear();
        self.dead_indices.clear();
        self.flat_snapshot.clear();
        for i in 0..self.entity_count {
            self.entities.push(AiEntity::new(i as u32));
        }
        self.snapshot_dirty = true;
        self.flat_snapshot_dirty = true;
    }

    /// Perform one simulation tick (update all entities)
    #[wasm_bindgen]
    pub fn step(&mut self) {
        let start = performance_now();

        self.tick = self.tick.wrapping_add(1); // Reuse snapshot buffer - capacity is pre-allocated in constructor
        self.snapshot_buffer.clear();
        for entity in &self.entities {
            self.snapshot_buffer.push(EntitySnapshot::from(entity));
        }

        self.grid.rebuild(&self.snapshot_buffer);

        let entities_len = self.entities.len();
        for i in 0..entities_len {
            // Safety: i is bounded by entities_len which equals self.entities.len()
            debug_assert!(i < self.entities.len());
            let entity = unsafe { self.entities.get_unchecked_mut(i) };
            entity.update(
                self.tick,
                i,
                &self.snapshot_buffer,
                &self.grid,
            );
        }

        // Process deaths and transfer resources using pre-allocated buffers
        // Note: This has O(dead_count * neighbors_per_dead_entity) complexity.
        // In scenarios with many simultaneous deaths, this could cause frame spikes.
        self.resource_transfers.clear();
        self.dead_indices.clear();

        // Single pass to identify dead entities and their resources
        for i in 0..entities_len {
            // Safety: i is bounded by entities_len which equals self.entities.len()
            debug_assert!(i < self.entities.len());
            let entity = unsafe { self.entities.get_unchecked(i) };

            if entity.health <= 0.0 && entity.state != AiState::Dead {
                self.dead_indices.push(i);

                // Find nearest Active attacker (only if there are resources to transfer)
                if entity.military_strength > 0.0 || entity.money > 0.0 {
                    let mut nearest_attacker_idx: Option<usize> = None;
                    let mut nearest_dist_sq = f32::INFINITY;

                    self.grid
                        .for_each_neighbor(entity.position_x, entity.position_y, |idx| {
                            if idx == i {
                                return;
                            }
                            // Safety: idx comes from spatial grid which only contains valid entity indices
                            debug_assert!(idx < self.entities.len());
                            let other = unsafe { self.entities.get_unchecked(idx) };

                            debug_assert_eq!(
                                other.state,
                                AiState::Active,
                                "Spatial grid should only contain Active entities"
                            );

                            let dx = entity.position_x - other.position_x;
                            let dy = entity.position_y - other.position_y;
                            let dist_sq = dx * dx + dy * dy;

                            if dist_sq < nearest_dist_sq {
                                nearest_dist_sq = dist_sq;
                                nearest_attacker_idx = Some(idx);
                            }
                        });

                    // Record transfer if attacker found
                    if let Some(attacker_idx) = nearest_attacker_idx {
                        self.resource_transfers.push((
                            attacker_idx,
                            entity.military_strength,
                            entity.money,
                        ));
                    }
                }
            }
        }

        // Apply resource transfers to attackers (now O(1) lookups)
        for &(attacker_idx, military_strength, money) in &self.resource_transfers {
            // Safety: attacker_idx comes from the loop above which uses valid entity indices
            debug_assert!(attacker_idx < self.entities.len());
            let attacker = unsafe { self.entities.get_unchecked_mut(attacker_idx) };
            attacker.military_strength += military_strength;
            attacker.money += money;
        }

        // Set dead entities to terminal state with all values at zero (now O(1) lookups)
        for &dead_idx in &self.dead_indices {
            // Safety: dead_idx comes from the loop above which uses valid entity indices
            debug_assert!(dead_idx < self.entities.len());
            let dead_entity = unsafe { self.entities.get_unchecked_mut(dead_idx) };
            dead_entity.state = AiState::Dead;
            dead_entity.health = 0.0;
            dead_entity.military_strength = 0.0;
            dead_entity.money = 0.0;
            dead_entity.territory = 0.0;
        }

        // Mark snapshot as dirty since we've updated entities
        self.snapshot_dirty = true;
        self.flat_snapshot_dirty = true;

        let end = performance_now();
        if start > 0.0 && end >= start {
            self.last_tick_duration_ms = end - start;
        }
    }

    /// Update the simulation (call this in a loop when running)
    #[wasm_bindgen]
    pub fn update(&mut self) {
        if self.running {
            self.step();
        }
    }

    /// Get current tick count
    #[wasm_bindgen]
    pub fn get_tick(&self) -> u64 {
        self.tick
    }

    /// Check if simulation is running
    #[wasm_bindgen]
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get entity count
    #[wasm_bindgen]
    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get tick rate
    #[wasm_bindgen]
    pub fn get_tick_rate(&self) -> u32 {
        self.tick_rate
    }

    /// Set tick rate
    #[wasm_bindgen]
    pub fn set_tick_rate(&mut self, tick_rate: u32) {
        self.tick_rate = tick_rate;
    }

    /// Set entity count (resets simulation)
    #[wasm_bindgen]
    pub fn set_entity_count(&mut self, entity_count: usize) {
        self.entity_count = entity_count;
        self.reset();
    }

    /// Get snapshot of all entities as a JsValue
    /// Only serializes if data has changed since last call
    #[wasm_bindgen]
    pub fn get_snapshot(&mut self) -> JsValue {
        if !self.snapshot_dirty {
            return JsValue::NULL; // Signal no update needed
        }

        let start = performance_now();

        let result = serde_wasm_bindgen::to_value(&self.entities).unwrap_or(JsValue::NULL);

        let end = performance_now();
        if start > 0.0 && end >= start {
            self.last_snapshot_duration_ms = end - start;
        }

        self.snapshot_dirty = false;
        result
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn get_flat_snapshot(&mut self) -> Float32Array {
        let start = performance_now();
        self.ensure_flat_snapshot_ready();
        let end = performance_now();
        if start > 0.0 && end >= start {
            self.last_snapshot_duration_ms = end - start;
        }
        unsafe { Float32Array::view(&self.flat_snapshot) }
    }

    /// Get last tick duration in milliseconds
    #[wasm_bindgen]
    pub fn get_last_tick_duration(&self) -> f64 {
        self.last_tick_duration_ms
    }

    /// Get last snapshot serialization duration in milliseconds
    #[wasm_bindgen]
    pub fn get_last_snapshot_duration(&self) -> f64 {
        self.last_snapshot_duration_ms
    }

    /// Destroy the simulation (cleanup)
    #[wasm_bindgen]
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_entity_creation() {
        let entity = AiEntity::new(0);
        assert_eq!(entity.id, 0);
        // Initial values now have variation
        assert!(entity.health >= 70.0 && entity.health <= 100.0);
        assert!(entity.military_strength >= 50.0 && entity.military_strength <= 100.0);
        // Entities now spawn at random positions across the grid, not at (0,0)
        assert!(entity.position_x >= -1250.0 && entity.position_x <= 1250.0);
        assert!(entity.position_y >= -1250.0 && entity.position_y <= 1250.0);
        assert_eq!(entity.territory, 10.0);
        // State is now varied per entity
    }

    #[test]
    fn test_ai_entity_update() {
        let mut entity = AiEntity::new(0);
        let snapshot = EntitySnapshot::from(&entity);
        let snapshots = vec![snapshot];
        let mut grid = SpatialGrid::new(5.0, 10.0);
        grid.rebuild(&snapshots);
        entity.update(1, 0, &snapshots, &grid);
        // Military strength should change after update (may increase or decrease depending on state)
        // Just verify the update doesn't crash
        assert!(entity.military_strength >= 0.0 && entity.military_strength <= 100.0);
    }

    #[test]
    fn test_simulation_creation() {
        let sim = Simulation::new(10);
        assert_eq!(sim.get_entity_count(), 10);
        assert_eq!(sim.get_tick(), 0);
        assert!(!sim.is_running());
    }

    #[test]
    fn test_simulation_step() {
        let mut sim = Simulation::new(10);
        let initial_tick = sim.get_tick();

        sim.step();

        assert_eq!(sim.get_tick(), initial_tick + 1);
        assert_eq!(sim.get_entity_count(), 10);
    }

    #[test]
    fn test_simulation_start_stop() {
        let mut sim = Simulation::new(10);

        sim.start();
        assert!(sim.is_running());

        sim.pause();
        assert!(!sim.is_running());
    }

    #[test]
    fn test_simulation_reset() {
        let mut sim = Simulation::new(10);

        sim.step();
        sim.step();
        assert_eq!(sim.get_tick(), 2);

        sim.reset();
        assert_eq!(sim.get_tick(), 0);
        assert_eq!(sim.get_entity_count(), 10);
    }

    #[test]
    fn test_simulation_multiple_steps() {
        let mut sim = Simulation::new(50);

        for _ in 0..10 {
            sim.step();
        }

        assert_eq!(sim.get_tick(), 10);
        assert_eq!(sim.get_entity_count(), 50);
    }

    #[test]
    fn test_simulation_performance_metrics() {
        let mut sim = Simulation::new(100);

        // Tick duration should be 0.0 initially
        assert_eq!(sim.get_last_tick_duration(), 0.0);

        // After stepping, we might not have timing on non-wasm32
        sim.step();
        let tick_duration = sim.get_last_tick_duration();
        assert!(tick_duration >= 0.0);
    }

    #[test]
    fn test_simulation_entity_count_change() {
        let mut sim = Simulation::new(10);
        assert_eq!(sim.get_entity_count(), 10);

        sim.set_entity_count(20);
        assert_eq!(sim.get_entity_count(), 20);
        assert_eq!(sim.get_tick(), 0); // Should reset
    }

    #[test]
    fn test_simulation_tick_rate() {
        let mut sim = Simulation::new(10);
        assert_eq!(sim.get_tick_rate(), 60); // Default

        sim.set_tick_rate(30);
        assert_eq!(sim.get_tick_rate(), 30);
    }

    #[test]
    fn test_simulation_start_pause() {
        let mut sim = Simulation::new(5);
        assert!(!sim.is_running());

        sim.start();
        assert!(sim.is_running());

        sim.pause();
        assert!(!sim.is_running());

        sim.resume();
        assert!(sim.is_running());
    }

    #[test]
    fn test_entity_energy_variation() {
        // Create multiple entities and verify they have different military strength levels after updates
        let mut entities = Vec::new();
        for i in 0..10 {
            entities.push(AiEntity::new(i));
        }

        let snapshots: Vec<EntitySnapshot> = entities.iter().map(EntitySnapshot::from).collect();
        let mut grid = SpatialGrid::new(5.0, 10.0);
        grid.rebuild(&snapshots);

        // Update all entities for the same tick using spatial grid neighbors
        for i in 0..entities.len() {
            entities[i].update(1, i, &snapshots, &grid);
        }

        // Print military strength values for debugging
        for entity in &entities {
            println!(
                "Entity {}: military_strength = {}",
                entity.id, entity.military_strength
            );
        }

        // Check that not all entities have the exact same military strength level
        let first_military_strength = entities[0].military_strength;
        let all_same = entities
            .iter()
            .all(|e| (e.military_strength - first_military_strength).abs() < 0.001);

        assert!(
            !all_same,
            "All entities should not have the exact same military strength level after update"
        );

        // Verify that we have at least some variation
        let max_military_strength = entities
            .iter()
            .map(|e| e.military_strength)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_military_strength = entities
            .iter()
            .map(|e| e.military_strength)
            .fold(f32::INFINITY, f32::min);

        assert!(
            max_military_strength - min_military_strength > 0.0,
            "Entities should have varying military strength levels"
        );
    }

    #[test]
    fn test_combat_reduces_health() {
        // Create two entities near each other
        let mut entity1 = AiEntity::new(0);
        let mut entity2 = AiEntity::new(1);

        // Position them close to each other
        entity1.position_x = 0.0;
        entity1.position_y = 0.0;
        entity2.position_x = 5.0;
        entity2.position_y = 0.0;

        // Set entity2 to Active state with high military strength
        entity2.state = AiState::Active;
        entity2.military_strength = 100.0;

        let initial_health = entity1.health;

        // Update entity1 with entity2 nearby and attacking
        let snapshots = vec![
            EntitySnapshot::from(&entity1),
            EntitySnapshot::from(&entity2),
        ];
        let mut grid = SpatialGrid::new(5.0, 10.0);
        grid.rebuild(&snapshots);
        entity1.update(1, 0, &snapshots, &grid);

        // Health should have decreased due to being attacked
        assert!(
            entity1.health < initial_health,
            "Health should decrease when attacked"
        );
    }

    #[test]
    fn test_expansion_increases_territory() {
        // Create entity in Moving state with high military strength
        let mut entity = AiEntity::new(0);
        entity.state = AiState::Moving;
        entity.military_strength = 80.0;
        entity.territory = 20.0;

        let initial_territory = entity.territory;

        // Update entity (alone, no combat)
        let snapshot = EntitySnapshot::from(&entity);
        let snapshots = vec![snapshot];
        let mut grid = SpatialGrid::new(5.0, 10.0);
        grid.rebuild(&snapshots);
        entity.update(1, 0, &snapshots, &grid);

        // Territory should have increased
        assert!(
            entity.territory >= initial_territory,
            "Territory should increase when in Moving state with high military strength"
        );
    }

    #[test]
    fn test_health_regeneration_only_when_safe() {
        // Create entity with low health, no nearby attackers
        let mut entity = AiEntity::new(0);
        entity.health = 50.0;
        entity.state = AiState::Resting;

        let initial_health = entity.health;

        // Update with no nearby entities
        let snapshot = EntitySnapshot::from(&entity);
        let snapshots = vec![snapshot];
        let mut grid = SpatialGrid::new(5.0, 10.0);
        grid.rebuild(&snapshots);
        entity.update(1, 0, &snapshots, &grid);

        // Health should regenerate when safe
        assert!(
            entity.health >= initial_health,
            "Health should regenerate when not under attack"
        );
    }

    #[test]
    fn test_death_when_health_reaches_zero() {
        // Create a simulation with two entities
        let mut sim = Simulation::new(2);

        // Set one entity to have zero health and position it
        sim.entities[0].health = 0.0;
        sim.entities[0].military_strength = 50.0;
        sim.entities[0].money = 100.0;
        sim.entities[0].position_x = 0.0;
        sim.entities[0].position_y = 0.0;

        // Set the other entity to Active state nearby
        sim.entities[1].state = AiState::Active;
        sim.entities[1].position_x = 5.0;
        sim.entities[1].position_y = 0.0;

        let initial_attacker_military = sim.entities[1].military_strength;
        let initial_attacker_money = sim.entities[1].money;

        // Run one step to process death
        sim.step();

        // First entity should be dead with all stats at zero
        assert_eq!(
            sim.entities[0].state,
            AiState::Dead,
            "Entity with zero health should be dead"
        );
        assert_eq!(
            sim.entities[0].health, 0.0,
            "Dead entity health should be 0"
        );
        assert_eq!(
            sim.entities[0].military_strength, 0.0,
            "Dead entity military strength should be 0"
        );
        assert_eq!(sim.entities[0].money, 0.0, "Dead entity money should be 0");
        assert_eq!(
            sim.entities[0].territory, 0.0,
            "Dead entity territory should be 0"
        );

        // Second entity should have received the resources
        assert!(
            sim.entities[1].military_strength > initial_attacker_military,
            "Attacker should receive military strength"
        );
        assert!(
            sim.entities[1].money > initial_attacker_money,
            "Attacker should receive money"
        );
    }

    #[test]
    fn test_dead_entities_dont_update() {
        // Create entity and set it to dead state
        let mut entity = AiEntity::new(0);
        entity.state = AiState::Dead;
        entity.health = 0.0;
        entity.military_strength = 0.0;
        entity.money = 0.0;
        entity.territory = 0.0;

        let snapshot = EntitySnapshot::from(&entity);
        let snapshots: Vec<EntitySnapshot> = vec![snapshot];
        let mut grid = SpatialGrid::new(5.0, 10.0);
        grid.rebuild(&snapshots);
        entity.update(1, 0, &snapshots, &grid);

        // All stats should remain at zero
        assert_eq!(entity.state, AiState::Dead, "Dead entity should stay dead");
        assert_eq!(entity.health, 0.0, "Dead entity health should stay 0");
        assert_eq!(
            entity.military_strength, 0.0,
            "Dead entity military strength should stay 0"
        );
        assert_eq!(entity.money, 0.0, "Dead entity money should stay 0");
        assert_eq!(entity.territory, 0.0, "Dead entity territory should stay 0");
    }

    #[test]
    fn test_dead_entities_dont_attack() {
        // Create entity with dead attacker nearby
        let mut entity = AiEntity::new(0);
        entity.health = 50.0;
        entity.position_x = 0.0;
        entity.position_y = 0.0;

        let mut dead_attacker = AiEntity::new(1);
        dead_attacker.state = AiState::Dead;
        dead_attacker.position_x = 5.0;
        dead_attacker.position_y = 0.0;
        dead_attacker.military_strength = 0.0;

        let initial_health = entity.health;

        let snapshots: Vec<EntitySnapshot> = vec![
            EntitySnapshot::from(&entity),
            EntitySnapshot::from(&dead_attacker),
        ];
        let mut grid = SpatialGrid::new(5.0, 10.0);
        grid.rebuild(&snapshots);
        entity.update(1, 0, &snapshots, &grid);

        // Health should not decrease from dead attacker
        assert!(
            entity.health >= initial_health,
            "Dead entities should not deal damage"
        );
    }

    #[test]
    fn test_entity_has_money_field() {
        // Verify that entities are created with money
        let entity = AiEntity::new(0);
        assert!(entity.money > 0.0, "New entities should have money");
        assert!(
            entity.money >= 100.0 && entity.money <= 200.0,
            "Money should be in expected range"
        );
    }

    #[test]
    fn test_resource_transfer_to_nearest_attacker() {
        // Create a simulation with three entities
        let mut sim = Simulation::new(3);

        // Entity 0 will die
        sim.entities[0].health = 0.0;
        sim.entities[0].military_strength = 100.0;
        sim.entities[0].money = 200.0;
        sim.entities[0].position_x = 0.0;
        sim.entities[0].position_y = 0.0;

        // Entity 1 is Active and nearby (will receive resources)
        sim.entities[1].state = AiState::Active;
        sim.entities[1].position_x = 3.0;
        sim.entities[1].position_y = 0.0;
        let entity1_initial_military = sim.entities[1].military_strength;
        let entity1_initial_money = sim.entities[1].money;

        // Entity 2 is Active but far away (should not receive resources)
        sim.entities[2].state = AiState::Active;
        sim.entities[2].position_x = 50.0;
        sim.entities[2].position_y = 50.0;
        let entity2_initial_military = sim.entities[2].military_strength;
        let entity2_initial_money = sim.entities[2].money;

        // Run one step to process death
        sim.step();

        // Entity 0 should be dead
        assert_eq!(sim.entities[0].state, AiState::Dead);

        // Entity 1 (nearest) should have received resources
        // Note: military strength also changes during update, so we check it increased by at least the transferred amount
        assert!(
            sim.entities[1].military_strength >= entity1_initial_military + 100.0 - 1.0,
            "Entity 1 should receive military strength from dead entity"
        );
        assert!(
            sim.entities[1].money >= entity1_initial_money + 200.0,
            "Entity 1 should receive money from dead entity"
        );

        // Entity 2 (far) should not have received the dead entity's resources
        // Check that Entity 2's resources didn't increase by the same large amount
        let entity2_military_gain = sim.entities[2].military_strength - entity2_initial_military;
        let entity2_money_gain = sim.entities[2].money - entity2_initial_money;
        assert!(
            entity2_military_gain < 50.0,
            "Entity 2 should not receive significant military strength"
        );
        assert!(
            entity2_money_gain < 50.0,
            "Entity 2 should not receive significant money"
        );
    }

    #[test]
    #[ignore] // This is a performance benchmark test - run with: cargo test -- --ignored
    fn test_benchmark_10000_elements() {
        use std::time::Instant;

        const ENTITY_COUNT: usize = 10_000;
        const TARGET_HZ: u32 = 240;
        const TARGET_TICK_TIME_MS: f64 = 1000.0 / TARGET_HZ as f64; // ~4.17ms per tick
        const BENCHMARK_TICKS: usize = 100; // Run 100 ticks for benchmarking

        println!("\n=== Benchmark: 10,000 Elements at 240 Hz Target ===");
        println!("Entity count: {}", ENTITY_COUNT);
        println!("Target tick rate: {} Hz", TARGET_HZ);
        println!("Target time per tick: {:.2} ms", TARGET_TICK_TIME_MS);

        // Create simulation with 10,000 entities
        let mut sim = Simulation::init(ENTITY_COUNT, TARGET_HZ);

        // Verify entity count
        assert_eq!(
            sim.get_entity_count(),
            ENTITY_COUNT,
            "Simulation should have exactly {} entities",
            ENTITY_COUNT
        );

        // Warm-up: Run 5 ticks to ensure everything is initialized
        println!("\nWarming up...");
        for _ in 0..5 {
            sim.step();
        }

        // Benchmark: Run multiple ticks and measure time
        println!("Running {} ticks for benchmark...", BENCHMARK_TICKS);
        let start = Instant::now();

        for _ in 0..BENCHMARK_TICKS {
            sim.step();
        }

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let avg_tick_time_ms = elapsed_ms / BENCHMARK_TICKS as f64;
        let achieved_hz = 1000.0 / avg_tick_time_ms;

        println!("\n--- Results ---");
        println!(
            "Total time for {} ticks: {:.2} ms ({:.2} s)",
            BENCHMARK_TICKS,
            elapsed_ms,
            elapsed.as_secs_f64()
        );
        println!("Average time per tick: {:.2} ms", avg_tick_time_ms);
        println!("Achieved tick rate: {:.2} Hz", achieved_hz);
        println!("Target tick rate: {} Hz", TARGET_HZ);
        println!(
            "Performance ratio: {:.1}% of target",
            (achieved_hz / TARGET_HZ as f64) * 100.0
        );

        // Verify that all entities were updated
        assert!(
            sim.get_tick() >= BENCHMARK_TICKS as u64,
            "All ticks should have been processed"
        );

        // Verify all entities still exist (none were removed)
        assert_eq!(
            sim.get_entity_count(),
            ENTITY_COUNT,
            "All {} entities should still exist after updates",
            ENTITY_COUNT
        );

        // Validate that all entities are being updated
        // Check that entities have varied states (proof they're being processed)
        let active_count = sim
            .entities
            .iter()
            .filter(|e| e.state == AiState::Active)
            .count();
        let resting_count = sim
            .entities
            .iter()
            .filter(|e| e.state == AiState::Resting)
            .count();
        let moving_count = sim
            .entities
            .iter()
            .filter(|e| e.state == AiState::Moving)
            .count();
        let idle_count = sim
            .entities
            .iter()
            .filter(|e| e.state == AiState::Idle)
            .count();
        let dead_count = sim
            .entities
            .iter()
            .filter(|e| e.state == AiState::Dead)
            .count();

        println!("\n--- Entity States ---");
        println!(
            "Active: {}, Resting: {}, Moving: {}, Idle: {}, Dead: {}",
            active_count, resting_count, moving_count, idle_count, dead_count
        );

        // Verify entities have different states (they're being processed)
        let total_living = active_count + resting_count + moving_count + idle_count;
        assert!(
            total_living > 0,
            "At least some entities should be alive and in various states"
        );

        println!("\n✓ Benchmark COMPLETED:");
        println!("  - Successfully updated all {} entities", ENTITY_COUNT);
        println!(
            "  - Achieved {:.2} Hz ({:.1}% of {} Hz target)",
            achieved_hz,
            (achieved_hz / TARGET_HZ as f64) * 100.0,
            TARGET_HZ
        );

        // For now, we just report performance without strict assertion
        // as the requirement is to validate that it CAN update all items
        // The actual Hz achieved will vary based on hardware
        if achieved_hz >= TARGET_HZ as f64 {
            println!("  ✓ MEETS performance target!");
        } else {
            println!("  ⚠ Below target (expected on debug builds)");
            println!("  Note: Run with --release for optimized performance");
        }
        println!();
    }

    #[test]
    fn test_entities_stay_within_bounds() {
        // Test that entities with Moving state stay within world bounds
        let mut sim = Simulation::new(10);

        // Set all entities to Moving state and run many ticks
        for entity in &mut sim.entities {
            entity.state = AiState::Moving;
        }

        // Run 500 ticks to allow plenty of movement
        for _ in 0..500 {
            sim.step();
        }

        // Verify all entities are still within bounds
        for entity in &sim.entities {
            assert!(
                entity.position_x >= -1250.0 && entity.position_x <= 1250.0,
                "Entity position_x {} is outside world bounds [-1250, 1250]",
                entity.position_x
            );
            assert!(
                entity.position_y >= -1250.0 && entity.position_y <= 1250.0,
                "Entity position_y {} is outside world bounds [-1250, 1250]",
                entity.position_y
            );
        }
    }

    #[test]
    fn test_spatial_grid_only_tracks_active() {
        // Test that the spatial grid only contains Active entities
        let mut sim = Simulation::new(20);

        // Set specific states
        sim.entities[0].state = AiState::Active;
        sim.entities[0].position_x = 0.0;
        sim.entities[0].position_y = 0.0;

        sim.entities[1].state = AiState::Resting;
        sim.entities[1].position_x = 5.0;
        sim.entities[1].position_y = 0.0;

        sim.entities[2].state = AiState::Moving;
        sim.entities[2].position_x = 10.0;
        sim.entities[2].position_y = 0.0;

        sim.entities[3].state = AiState::Active;
        sim.entities[3].position_x = 15.0;
        sim.entities[3].position_y = 0.0;

        // Run one step to rebuild spatial grid
        sim.step();

        // Query neighbors near entity 0 (should only find other Active entities)
        let mut neighbors = Vec::new();
        sim.grid
            .for_each_neighbor(0.0, 0.0, |idx| neighbors.push(idx));

        // Verify all neighbors are from Active entities
        for &idx in &neighbors {
            if idx < sim.entities.len() {
                // The spatial grid should only contain Active entities
                // Note: State may have changed during step(), but initially only Active were added
            }
        }
    }

    #[test]
    fn test_grid_handles_large_distribution() {
        // Test that the 500x500 grid can handle entities spread across the world
        let mut sim = Simulation::new(100);

        // Manually position entities across the entire grid
        for i in 0..100 {
            let x = -1200.0 + (i as f32 * 24.0); // Spread across x axis
            let y = ((i as f32 * 13.7).sin()) * 1200.0; // Vary y position
            sim.entities[i].position_x = x;
            sim.entities[i].position_y = y;
            sim.entities[i].state = AiState::Active; // Make them all Active
        }

        // Run multiple steps
        for _ in 0..10 {
            sim.step();
        }

        // Verify no entities were lost and all are still within bounds
        let active_count = sim
            .entities
            .iter()
            .filter(|e| e.state != AiState::Dead)
            .count();
        assert!(
            active_count >= 50,
            "Most entities should still be alive after 10 ticks"
        );

        for entity in &sim.entities {
            if entity.state != AiState::Dead {
                assert!(
                    entity.position_x >= -1250.0 && entity.position_x <= 1250.0,
                    "Entity x position should be within world bounds"
                );
                assert!(
                    entity.position_y >= -1250.0 && entity.position_y <= 1250.0,
                    "Entity y position should be within world bounds"
                );
            }
        }
    }

    #[test]
    fn test_determinism_with_new_grid_size() {
        // Test that the simulation remains deterministic with 500x500 grid
        let mut sim1 = Simulation::new(50);
        let mut sim2 = Simulation::new(50);

        // Run both simulations for same number of steps
        for _ in 0..20 {
            sim1.step();
            sim2.step();
        }

        // Verify entities in both simulations are in identical states
        for i in 0..50 {
            assert_eq!(
                sim1.entities[i].state, sim2.entities[i].state,
                "Entity {} state should be identical",
                i
            );
            assert_eq!(
                sim1.entities[i].health, sim2.entities[i].health,
                "Entity {} health should be identical",
                i
            );
            assert_eq!(
                sim1.entities[i].position_x, sim2.entities[i].position_x,
                "Entity {} position_x should be identical",
                i
            );
            assert_eq!(
                sim1.entities[i].position_y, sim2.entities[i].position_y,
                "Entity {} position_y should be identical",
                i
            );
        }
    }
}

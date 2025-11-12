use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Counter struct to manage counter state and logic in Rust/WASM
#[wasm_bindgen]
pub struct Counter {
    value: i32,
}

#[wasm_bindgen]
impl Counter {
    /// Create a new counter with initial value of 0
    #[wasm_bindgen(constructor)]
    pub fn new() -> Counter {
        Counter { value: 0 }
    }

    /// Create a counter with a specific initial value
    #[wasm_bindgen]
    pub fn with_value(initial: i32) -> Counter {
        Counter { value: initial }
    }

    /// Increment the counter by 1
    #[wasm_bindgen]
    pub fn increment(&mut self) -> i32 {
        self.value = self.value.saturating_add(1);
        self.value
    }

    /// Decrement the counter by 1
    #[wasm_bindgen]
    pub fn decrement(&mut self) -> i32 {
        self.value = self.value.saturating_sub(1);
        self.value
    }

    /// Get the current value
    #[wasm_bindgen]
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// Reset the counter to 0
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.value = 0;
    }

    /// Set the counter to a specific value
    #[wasm_bindgen]
    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }
}

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
            _ => AiState::Idle,
        }
    }
}

/// AI entity with scalar attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEntity {
    pub id: u32,
    pub health: f32,
    pub energy: f32,
    pub position_x: f32,
    pub position_y: f32,
    pub state: AiState,
}

impl AiEntity {
    /// Create a new AI entity with default values
    pub fn new(id: u32) -> Self {
        // Create per-entity variation for initial state
        // Use id as seed for deterministic but varied initialization
        let id_seed = id as f32;
        let variation = ((id_seed * 0.7321).sin() + 1.0) / 2.0; // 0.0 to 1.0 range
        
        // Vary initial energy between 50 and 100
        let initial_energy = 50.0 + (variation * 50.0);
        
        // Vary initial health between 70 and 100  
        let health_variation = ((id_seed * 1.234).cos() + 1.0) / 2.0;
        let initial_health = 70.0 + (health_variation * 30.0);
        
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
        
        Self {
            id,
            health: initial_health,
            energy: initial_energy,
            position_x: 0.0,
            position_y: 0.0,
            state: initial_state,
        }
    }

    /// Update the entity for one simulation tick
    pub fn update(&mut self, tick: u64) {
        // Deterministic update logic based on tick and entity id
        // Use a better pseudo-random variation that's unique per entity
        let seed1 = (tick.wrapping_mul(1000) + self.id as u64) as f32;
        let seed2 = (tick.wrapping_mul(7919) + self.id.wrapping_mul(6547) as u64) as f32;
        
        // Create entity-specific variation factors (0.5 to 1.5 range)
        // Use different multipliers for better spread
        let id_factor = ((self.id as f32 * 0.7321).sin() + 1.0) / 2.0 + 0.5;
        let tick_factor = ((seed2 * 0.00123).cos() + 1.0) / 2.0 + 0.5;
        let variation = id_factor * tick_factor;
        
        // Energy dynamics with per-entity variation
        match self.state {
            AiState::Active => {
                self.energy = (self.energy - 0.5 * variation).max(0.0);
                if self.energy < 20.0 {
                    self.state = AiState::Resting;
                }
            }
            AiState::Resting => {
                self.energy = (self.energy + 1.0 * variation).min(100.0);
                if self.energy > 80.0 {
                    self.state = AiState::Moving;
                }
            }
            AiState::Moving => {
                self.energy = (self.energy - 0.2 * variation).max(0.0);
                // Simple deterministic movement
                self.position_x += (seed1 * 0.1).sin() * 2.0 * variation;
                self.position_y += (seed1 * 0.1).cos() * 2.0 * variation;
                
                if self.energy < 50.0 {
                    self.state = AiState::Active;
                }
            }
            AiState::Idle => {
                self.energy = (self.energy + 0.1 * variation).min(100.0);
                if self.energy > 90.0 {
                    self.state = AiState::Active;
                }
            }
        }

        // Health regeneration with variation
        if self.health < 100.0 {
            self.health = (self.health + 0.1 * variation).min(100.0);
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
        for i in 0..self.entity_count {
            self.entities.push(AiEntity::new(i as u32));
        }
    }

    /// Perform one simulation tick (update all entities)
    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        for entity in &mut self.entities {
            entity.update(self.tick);
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
    #[wasm_bindgen]
    pub fn get_snapshot(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.entities).unwrap_or(JsValue::NULL)
    }

    /// Destroy the simulation (cleanup)
    #[wasm_bindgen]
    pub fn destroy(&mut self) {
        self.running = false;
        self.entities.clear();
        self.tick = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_counter() {
        let counter = Counter::new();
        assert_eq!(counter.get_value(), 0);
    }

    #[test]
    fn test_increment() {
        let mut counter = Counter::new();
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
    }

    #[test]
    fn test_decrement() {
        let mut counter = Counter::new();
        counter.set_value(5);
        assert_eq!(counter.decrement(), 4);
        assert_eq!(counter.decrement(), 3);
    }

    #[test]
    fn test_reset() {
        let mut counter = Counter::with_value(10);
        counter.reset();
        assert_eq!(counter.get_value(), 0);
    }

    #[test]
    fn test_ai_entity_creation() {
        let entity = AiEntity::new(0);
        assert_eq!(entity.id, 0);
        // Initial values now have variation
        assert!(entity.health >= 70.0 && entity.health <= 100.0);
        assert!(entity.energy >= 50.0 && entity.energy <= 100.0);
        assert_eq!(entity.position_x, 0.0);
        assert_eq!(entity.position_y, 0.0);
        // State is now varied per entity
    }

    #[test]
    fn test_ai_entity_update() {
        let mut entity = AiEntity::new(0);
        entity.update(1);
        // Energy should change after update (may increase or decrease depending on state)
        // Just verify the update doesn't crash
        assert!(entity.energy >= 0.0 && entity.energy <= 100.0);
    }

    #[test]
    fn test_simulation_creation() {
        let sim = Simulation::new(10);
        assert_eq!(sim.get_entity_count(), 10);
        assert_eq!(sim.get_tick(), 0);
        assert!(!sim.is_running());
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
    fn test_simulation_step() {
        let mut sim = Simulation::new(5);
        assert_eq!(sim.get_tick(), 0);
        
        sim.step();
        assert_eq!(sim.get_tick(), 1);
        
        sim.step();
        assert_eq!(sim.get_tick(), 2);
    }

    #[test]
    fn test_simulation_reset() {
        let mut sim = Simulation::new(5);
        sim.start();
        sim.step();
        sim.step();
        assert_eq!(sim.get_tick(), 2);
        
        sim.reset();
        assert_eq!(sim.get_tick(), 0);
        assert!(!sim.is_running());
        assert_eq!(sim.get_entity_count(), 5);
    }

    #[test]
    fn test_simulation_tick_rate() {
        let mut sim = Simulation::new(5);
        assert_eq!(sim.get_tick_rate(), 60);
        
        sim.set_tick_rate(30);
        assert_eq!(sim.get_tick_rate(), 30);
    }

    #[test]
    fn test_entity_energy_variation() {
        // Create multiple entities and verify they have different energy levels after updates
        let mut entities = Vec::new();
        for i in 0..10 {
            entities.push(AiEntity::new(i));
        }
        
        // Update all entities for the same tick
        for entity in &mut entities {
            entity.update(1);
        }
        
        // Print energy values for debugging
        for entity in &entities {
            println!("Entity {}: energy = {}", entity.id, entity.energy);
        }
        
        // Check that not all entities have the exact same energy level
        let first_energy = entities[0].energy;
        let all_same = entities.iter().all(|e| (e.energy - first_energy).abs() < 0.001);
        
        assert!(!all_same, "All entities should not have the exact same energy level after update");
        
        // Verify that we have at least some variation
        let max_energy = entities.iter().map(|e| e.energy).fold(f32::NEG_INFINITY, f32::max);
        let min_energy = entities.iter().map(|e| e.energy).fold(f32::INFINITY, f32::min);
        
        assert!(max_energy - min_energy > 0.0, "Entities should have varying energy levels");
    }
}

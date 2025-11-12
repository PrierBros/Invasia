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
pub enum AiState {
    Idle = 0,
    Active = 1,
    Resting = 2,
    Moving = 3,
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
        Self {
            id,
            health: 100.0,
            energy: 100.0,
            position_x: 0.0,
            position_y: 0.0,
            state: AiState::Idle,
        }
    }

    /// Update the entity for one simulation tick
    pub fn update(&mut self, tick: u64) {
        // Deterministic update logic based on tick and entity id
        let seed = (tick.wrapping_mul(1000) + self.id as u64) as f32;
        
        // Energy dynamics
        match self.state {
            AiState::Active => {
                self.energy = (self.energy - 0.5).max(0.0);
                if self.energy < 20.0 {
                    self.state = AiState::Resting;
                }
            }
            AiState::Resting => {
                self.energy = (self.energy + 1.0).min(100.0);
                if self.energy > 80.0 {
                    self.state = AiState::Moving;
                }
            }
            AiState::Moving => {
                self.energy = (self.energy - 0.2).max(0.0);
                // Simple deterministic movement
                self.position_x += (seed * 0.1).sin() * 2.0;
                self.position_y += (seed * 0.1).cos() * 2.0;
                
                if self.energy < 50.0 {
                    self.state = AiState::Active;
                }
            }
            AiState::Idle => {
                self.energy = (self.energy + 0.1).min(100.0);
                if self.energy > 90.0 {
                    self.state = AiState::Active;
                }
            }
        }

        // Health regeneration
        if self.health < 100.0 {
            self.health = (self.health + 0.1).min(100.0);
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
        assert_eq!(entity.health, 100.0);
        assert_eq!(entity.energy, 100.0);
        assert_eq!(entity.position_x, 0.0);
        assert_eq!(entity.position_y, 0.0);
        assert_eq!(entity.state, AiState::Idle);
    }

    #[test]
    fn test_ai_entity_update() {
        let mut entity = AiEntity::new(0);
        entity.update(1);
        // After one tick in Idle state, energy should increase slightly
        assert!(entity.energy >= 100.0);
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
}

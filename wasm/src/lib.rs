use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub territory: f32,  // Territory controlled by this entity
    pub money: f32,      // Money/resources owned by this entity
}

impl AiEntity {
    /// Create a new AI entity with default values
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
        
        Self {
            id,
            health: initial_health,
            military_strength: initial_military_strength,
            position_x: 0.0,
            position_y: 0.0,
            state: initial_state,
            territory: 10.0,  // Start with small territory
            money: initial_money,
        }
    }

    /// Update the entity for one simulation tick
    pub fn update(&mut self, tick: u64, all_entities: &[AiEntity]) {
        // Dead entities don't update
        if self.state == AiState::Dead {
            return;
        }
        
        // Deterministic update logic based on tick and entity id
        // Use a better pseudo-random variation that's unique per entity
        let seed1 = (tick.wrapping_mul(1000) + self.id as u64) as f32;
        let seed2 = (tick.wrapping_mul(7919) + self.id.wrapping_mul(6547) as u64) as f32;
        
        // Create entity-specific variation factors (0.5 to 1.5 range)
        // Use different multipliers for better spread
        let id_factor = ((self.id as f32 * 0.7321).sin() + 1.0) / 2.0 + 0.5;
        let tick_factor = ((seed2 * 0.00123).cos() + 1.0) / 2.0 + 0.5;
        let variation = id_factor * tick_factor;
        
        // Military strength dynamics with per-entity variation
        match self.state {
            AiState::Active => {
                // Active state: Attack nearby entities
                self.military_strength = (self.military_strength - 0.3 * variation).max(0.0);
                
                // Entities in Active state deal damage to nearby entities
                // (damage is calculated and applied in the combat damage section below)
                
                if self.military_strength < 20.0 {
                    self.state = AiState::Resting;
                }
            }
            AiState::Resting => {
                // Resting state: Rebuild military strength
                self.military_strength = (self.military_strength + 1.0 * variation).min(100.0);
                if self.military_strength > 80.0 {
                    self.state = AiState::Moving;
                }
            }
            AiState::Moving => {
                // Moving state: Attempt expansion
                self.military_strength = (self.military_strength - 0.2 * variation).max(0.0);
                
                // Simple deterministic movement
                self.position_x += (seed1 * 0.1).sin() * 2.0 * variation;
                self.position_y += (seed1 * 0.1).cos() * 2.0 * variation;
                
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
                // Already handled above, but include for completeness
                return;
            }
        }

        // Apply combat damage from nearby Active entities
        let mut total_damage = 0.0;
        for other in all_entities {
            if other.id != self.id && other.state == AiState::Active {
                let dx = self.position_x - other.position_x;
                let dy = self.position_y - other.position_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                // Take damage from nearby attackers
                if distance < 10.0 && distance > 0.1 {
                    let damage = (other.military_strength / 100.0) * 0.5 * variation;
                    total_damage += damage;
                }
            }
        }
        
        // Apply damage to health
        if total_damage > 0.0 {
            self.health = (self.health - total_damage).max(0.0);
        }

        // Health regeneration with variation (slower than before, and not during combat)
        if self.health < 100.0 && total_damage == 0.0 {
            self.health = (self.health + 0.05 * variation).min(100.0);
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
        
        // Clone entities for reading during updates
        let entities_snapshot = self.entities.clone();
        
        // Update each entity with access to all entities for combat
        for entity in &mut self.entities {
            entity.update(self.tick, &entities_snapshot);
        }
        
        // Process deaths and transfer resources
        // Collect information about deaths and who should receive resources
        let mut resource_transfers: Vec<(u32, f32, f32)> = Vec::new(); // (attacker_id, military_strength, money)
        let mut dead_ids: Vec<u32> = Vec::new();
        
        for entity in &self.entities {
            if entity.health <= 0.0 && entity.state != AiState::Dead {
                let military_strength = entity.military_strength;
                let money = entity.money;
                
                // Find nearest Active attacker
                let mut nearest_attacker_id: Option<u32> = None;
                let mut nearest_distance = f32::INFINITY;
                
                for other in &self.entities {
                    if other.id != entity.id && other.state == AiState::Active && other.state != AiState::Dead {
                        let dx = entity.position_x - other.position_x;
                        let dy = entity.position_y - other.position_y;
                        let distance = (dx * dx + dy * dy).sqrt();
                        
                        if distance < nearest_distance {
                            nearest_distance = distance;
                            nearest_attacker_id = Some(other.id);
                        }
                    }
                }
                
                // Record transfer if attacker found
                if let Some(attacker_id) = nearest_attacker_id {
                    resource_transfers.push((attacker_id, military_strength, money));
                }
                
                dead_ids.push(entity.id);
            }
        }
        
        // Apply resource transfers to attackers
        for (attacker_id, military_strength, money) in resource_transfers {
            if let Some(attacker) = self.entities.iter_mut().find(|e| e.id == attacker_id) {
                attacker.military_strength += military_strength;
                attacker.money += money;
            }
        }
        
        // Set dead entities to terminal state with all values at zero
        for dead_id in dead_ids {
            if let Some(dead_entity) = self.entities.iter_mut().find(|e| e.id == dead_id) {
                dead_entity.state = AiState::Dead;
                dead_entity.health = 0.0;
                dead_entity.military_strength = 0.0;
                dead_entity.money = 0.0;
                dead_entity.territory = 0.0;
            }
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
    fn test_ai_entity_creation() {
        let entity = AiEntity::new(0);
        assert_eq!(entity.id, 0);
        // Initial values now have variation
        assert!(entity.health >= 70.0 && entity.health <= 100.0);
        assert!(entity.military_strength >= 50.0 && entity.military_strength <= 100.0);
        assert_eq!(entity.position_x, 0.0);
        assert_eq!(entity.position_y, 0.0);
        assert_eq!(entity.territory, 10.0);
        // State is now varied per entity
    }

    #[test]
    fn test_ai_entity_update() {
        let mut entity = AiEntity::new(0);
        let entities = vec![entity.clone()];
        entity.update(1, &entities);
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
        // Create multiple entities and verify they have different military strength levels after updates
        let mut entities = Vec::new();
        for i in 0..10 {
            entities.push(AiEntity::new(i));
        }
        
        // Clone for read access during updates
        let entities_snapshot = entities.clone();
        
        // Update all entities for the same tick
        for entity in &mut entities {
            entity.update(1, &entities_snapshot);
        }
        
        // Print military strength values for debugging
        for entity in &entities {
            println!("Entity {}: military_strength = {}", entity.id, entity.military_strength);
        }
        
        // Check that not all entities have the exact same military strength level
        let first_military_strength = entities[0].military_strength;
        let all_same = entities.iter().all(|e| (e.military_strength - first_military_strength).abs() < 0.001);
        
        assert!(!all_same, "All entities should not have the exact same military strength level after update");
        
        // Verify that we have at least some variation
        let max_military_strength = entities.iter().map(|e| e.military_strength).fold(f32::NEG_INFINITY, f32::max);
        let min_military_strength = entities.iter().map(|e| e.military_strength).fold(f32::INFINITY, f32::min);
        
        assert!(max_military_strength - min_military_strength > 0.0, "Entities should have varying military strength levels");
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
        let entities = vec![entity1.clone(), entity2.clone()];
        entity1.update(1, &entities);
        
        // Health should have decreased due to being attacked
        assert!(entity1.health < initial_health, "Health should decrease when attacked");
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
        let entities = vec![entity.clone()];
        entity.update(1, &entities);
        
        // Territory should have increased
        assert!(entity.territory >= initial_territory, "Territory should increase when in Moving state with high military strength");
    }

    #[test]
    fn test_health_regeneration_only_when_safe() {
        // Create entity with low health, no nearby attackers
        let mut entity = AiEntity::new(0);
        entity.health = 50.0;
        entity.state = AiState::Resting;
        
        let initial_health = entity.health;
        
        // Update with no nearby entities
        let entities = vec![entity.clone()];
        entity.update(1, &entities);
        
        // Health should regenerate when safe
        assert!(entity.health >= initial_health, "Health should regenerate when not under attack");
    }

    #[test]
    fn test_death_when_health_reaches_zero() {
        // Create a simulation with two entities
        let mut sim = Simulation::new(2);
        
        // Set one entity to have zero health
        sim.entities[0].health = 0.0;
        sim.entities[0].military_strength = 50.0;
        sim.entities[0].money = 100.0;
        
        // Set the other entity to Active state nearby
        sim.entities[1].state = AiState::Active;
        sim.entities[1].position_x = 5.0;
        sim.entities[1].position_y = 0.0;
        
        let initial_attacker_military = sim.entities[1].military_strength;
        let initial_attacker_money = sim.entities[1].money;
        
        // Run one step to process death
        sim.step();
        
        // First entity should be dead with all stats at zero
        assert_eq!(sim.entities[0].state, AiState::Dead, "Entity with zero health should be dead");
        assert_eq!(sim.entities[0].health, 0.0, "Dead entity health should be 0");
        assert_eq!(sim.entities[0].military_strength, 0.0, "Dead entity military strength should be 0");
        assert_eq!(sim.entities[0].money, 0.0, "Dead entity money should be 0");
        assert_eq!(sim.entities[0].territory, 0.0, "Dead entity territory should be 0");
        
        // Second entity should have received the resources
        assert!(sim.entities[1].military_strength > initial_attacker_military, "Attacker should receive military strength");
        assert!(sim.entities[1].money > initial_attacker_money, "Attacker should receive money");
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
        
        let entities = vec![entity.clone()];
        entity.update(1, &entities);
        
        // All stats should remain at zero
        assert_eq!(entity.state, AiState::Dead, "Dead entity should stay dead");
        assert_eq!(entity.health, 0.0, "Dead entity health should stay 0");
        assert_eq!(entity.military_strength, 0.0, "Dead entity military strength should stay 0");
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
        
        let entities = vec![entity.clone(), dead_attacker];
        entity.update(1, &entities);
        
        // Health should not decrease from dead attacker
        assert!(entity.health >= initial_health, "Dead entities should not deal damage");
    }

    #[test]
    fn test_entity_has_money_field() {
        // Verify that entities are created with money
        let entity = AiEntity::new(0);
        assert!(entity.money > 0.0, "New entities should have money");
        assert!(entity.money >= 100.0 && entity.money <= 200.0, "Money should be in expected range");
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
        assert!(sim.entities[1].military_strength >= entity1_initial_military + 100.0 - 1.0, 
                "Entity 1 should receive military strength from dead entity");
        assert!(sim.entities[1].money >= entity1_initial_money + 200.0, 
                "Entity 1 should receive money from dead entity");
        
        // Entity 2 (far) should not have received the dead entity's resources
        // Check that Entity 2's resources didn't increase by the same large amount
        let entity2_military_gain = sim.entities[2].military_strength - entity2_initial_military;
        let entity2_money_gain = sim.entities[2].money - entity2_initial_money;
        assert!(entity2_military_gain < 50.0, "Entity 2 should not receive significant military strength");
        assert!(entity2_money_gain < 50.0, "Entity 2 should not receive significant money");
    }
}

/// World state and simulation management
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

use super::actions::*;
use super::country::*;
use super::luts::*;
use super::scoring::*;

/// Alliance relationships between countries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alliance {
    pub country_a: u32,
    pub country_b: u32,
}

/// World state containing all countries and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    countries: HashMap<u32, Country>,
    alliances: HashSet<(u32, u32)>,  // Normalized pairs (min, max)
    tick: u64,
}

impl WorldState {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            countries: HashMap::new(),
            alliances: HashSet::new(),
            tick: 0,
        }
    }
    
    /// Add a country to the world
    pub fn add_country(&mut self, country: Country) {
        self.countries.insert(country.id, country);
    }
    
    /// Get a country by ID
    pub fn get_country(&self, id: u32) -> Option<&Country> {
        self.countries.get(&id)
    }
    
    /// Get a mutable country by ID
    pub fn get_country_mut(&mut self, id: u32) -> Option<&mut Country> {
        self.countries.get_mut(&id)
    }
    
    /// Get all countries
    pub fn countries(&self) -> &HashMap<u32, Country> {
        &self.countries
    }
    
    /// Add an alliance between two countries
    pub fn add_alliance(&mut self, a: u32, b: u32) {
        let pair = if a < b { (a, b) } else { (b, a) };
        self.alliances.insert(pair);
        
        // Update ally counts
        if let Some(country_a) = self.countries.get_mut(&a) {
            country_a.ally_count += 1;
        }
        if let Some(country_b) = self.countries.get_mut(&b) {
            country_b.ally_count += 1;
        }
    }
    
    /// Check if two countries are allies
    pub fn are_allies(&self, a: u32, b: u32) -> bool {
        let pair = if a < b { (a, b) } else { (b, a) };
        self.alliances.contains(&pair)
    }
    
    /// Get current tick
    pub fn get_tick(&self) -> u64 {
        self.tick
    }
    
    /// Update all countries' threat indices incrementally
    pub fn update_threat_indices(&mut self, luts: &LookupTables) {
        let country_ids: Vec<u32> = self.countries.keys().copied().collect();
        
        for &id in &country_ids {
            if let Some(country) = self.countries.get(&id) {
                let ti = compute_threat_index(country, self, luts);
                if let Some(country_mut) = self.countries.get_mut(&id) {
                    country_mut.threat_index = ti;
                }
            }
        }
    }
    
    /// Update all countries' adaptive weights
    pub fn update_weights(&mut self) {
        for country in self.countries.values_mut() {
            let resources = country.resources;
            let threat_index = country.threat_index;
            let growth = country.growth;
            let ally_count = country.ally_count;
            let recent_losses = country.recent_losses;
            let m_eff = country.m_eff;
            let gdp = country.gdp;
            let tech_level = country.tech_level;
            let prestige = country.prestige;
            
            country.weights.update(resources, threat_index, growth, ally_count, recent_losses);
            country.marginal_values.update(m_eff, gdp, tech_level, prestige);
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

/// Decision log entry for telemetry (§9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLog {
    pub tick: u64,
    pub country_id: u32,
    pub chosen_action: String,
    pub score: f32,
    pub components: ScoreComponents,
    pub weights: AdaptiveWeights,
    pub rejected_actions: Vec<(String, f32)>,  // Top 1-2 rejected with scores
}

/// AI Decision System - main coordinator (§6, §10)
#[wasm_bindgen]
pub struct DecisionSystem {
    world: WorldState,
    luts: LookupTables,
    pruning_config: PruningConfig,
    logs: Vec<DecisionLog>,
    rng_seed: u64,
}

#[wasm_bindgen]
impl DecisionSystem {
    /// Create a new decision system
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            world: WorldState::new(),
            luts: LookupTables::new(),
            pruning_config: PruningConfig::new(),
            logs: Vec::new(),
            rng_seed: 12345,
        }
    }
    
    /// Initialize with custom seed for determinism
    #[wasm_bindgen]
    pub fn init(seed: u64) -> Self {
        Self {
            world: WorldState::new(),
            luts: LookupTables::new(),
            pruning_config: PruningConfig::new(),
            logs: Vec::new(),
            rng_seed: seed,
        }
    }
    
    /// Add a country to the world
    #[wasm_bindgen]
    pub fn add_country(&mut self, id: u32) {
        let country = Country::new(id);
        self.world.add_country(country);
    }
    
    /// Add an edge between two countries
    #[wasm_bindgen]
    pub fn add_edge(&mut self, from_id: u32, to_id: u32, distance: usize, hostility: f32) {
        if let Some(country) = self.world.get_country_mut(from_id) {
            let mut edge = CountryEdge::new(to_id);
            edge.distance_bucket = distance;
            edge.hostility = hostility;
            country.add_edge(edge);
        }
    }
    
    /// Execute one tick of the decision system (§6)
    #[wasm_bindgen]
    pub fn tick(&mut self) {
        // 1. Update weights
        self.world.update_weights();
        
        // 2. Update local fields (TI, caches)
        self.world.update_threat_indices(&self.luts);
        
        // 3-5. Build shortlist, score, and choose for each country
        let country_ids: Vec<u32> = self.world.countries().keys().copied().collect();
        let mut decisions: HashMap<u32, (Action, f32, ScoreComponents)> = HashMap::new();
        
        for country_id in country_ids {
            if let Some(country) = self.world.get_country(country_id) {
                // 3. Build shortlist
                let shortlist = generate_shortlist(
                    country_id,
                    country,
                    &self.world,
                    &self.pruning_config,
                );
                
                // 4. Score each action
                let mut best_action = Action::Pass;
                let mut best_score = f32::NEG_INFINITY;
                let mut best_components = ScoreComponents::zero();
                let mut scored_actions = Vec::new();
                
                for action in &shortlist {
                    let components = score_action(country, action, &self.world, &self.luts);
                    let score = components.final_score(&country.weights);
                    
                    scored_actions.push((action.description(), score));
                    
                    if score > best_score {
                        best_score = score;
                        best_action = action.clone();
                        best_components = components.clone();
                    }
                }
                
                // 5. Choose action (argmax)
                decisions.insert(country_id, (best_action.clone(), best_score, best_components.clone()));
                
                // 7. Log telemetry
                let mut rejected = scored_actions
                    .into_iter()
                    .filter(|(desc, _)| desc != &best_action.description())
                    .collect::<Vec<_>>();
                rejected.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                rejected.truncate(2);  // Top 2 rejected
                
                self.logs.push(DecisionLog {
                    tick: self.world.tick,
                    country_id,
                    chosen_action: best_action.description(),
                    score: best_score,
                    components: best_components,
                    weights: country.weights.clone(),
                    rejected_actions: rejected,
                });
            }
        }
        
        // 6. Apply actions and emit deltas
        self.apply_actions(decisions);
        
        // Increment tick
        self.world.tick += 1;
    }
    
    /// Get current tick
    #[wasm_bindgen]
    pub fn get_tick(&self) -> u64 {
        self.world.get_tick()
    }
    
    /// Get decision logs as JSON
    #[wasm_bindgen]
    pub fn get_logs(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.logs).unwrap_or(JsValue::NULL)
    }
    
    /// Get world state snapshot as JSON
    #[wasm_bindgen]
    pub fn get_world_snapshot(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.world).unwrap_or(JsValue::NULL)
    }
    
    /// Clear logs (for memory management)
    #[wasm_bindgen]
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
}

// Non-WASM methods
impl DecisionSystem {
    /// Apply all chosen actions to world state
    fn apply_actions(&mut self, decisions: HashMap<u32, (Action, f32, ScoreComponents)>) {
        for (country_id, (action, _score, components)) in decisions {
            self.apply_action(country_id, &action, &components);
        }
    }
    
    /// Apply a single action
    fn apply_action(&mut self, country_id: u32, action: &Action, components: &ScoreComponents) {
        match action {
            Action::Attack { target_id: _ } => {
                // Simple implementation: apply resource and security changes
                if let Some(country) = self.world.get_country_mut(country_id) {
                    country.resources += components.delta_res * 50.0;  // Denormalize
                    country.resources = country.resources.max(0.0);
                }
            }
            Action::Invest { sector: _ } => {
                if let Some(country) = self.world.get_country_mut(country_id) {
                    // Apply growth
                    country.growth += components.delta_growth * 0.1;
                    country.resources -= components.cost * 20.0;  // Denormalize cost
                    country.resources = country.resources.max(0.0);
                }
            }
            Action::Research { tech: _ } => {
                if let Some(country) = self.world.get_country_mut(country_id) {
                    // Apply tech advancement
                    country.tech_level += 0.1;
                    country.resources -= components.cost * 20.0;
                    country.resources = country.resources.max(0.0);
                }
            }
            Action::Ally { target_id } => {
                // Form alliance
                self.world.add_alliance(country_id, *target_id);
            }
            Action::Pact { .. } | Action::Trade { .. } => {
                // Update relations/resources
                if let Some(country) = self.world.get_country_mut(country_id) {
                    country.resources += components.delta_res * 50.0;
                }
            }
            Action::Fortify { tile_id } => {
                if let Some(country) = self.world.get_country_mut(country_id) {
                    if let Some(tile) = country.border_tiles.iter_mut().find(|t| t.id == *tile_id) {
                        tile.fortification += 0.5;
                    }
                }
            }
            Action::Move { .. } => {
                // Movement logic (simplified)
            }
            Action::Pass => {
                // No action
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_state_creation() {
        let world = WorldState::new();
        assert_eq!(world.countries().len(), 0);
        assert_eq!(world.get_tick(), 0);
    }

    #[test]
    fn test_add_country() {
        let mut world = WorldState::new();
        let country = Country::new(1);
        world.add_country(country);
        
        assert_eq!(world.countries().len(), 1);
        assert!(world.get_country(1).is_some());
    }

    #[test]
    fn test_alliances() {
        let mut world = WorldState::new();
        world.add_country(Country::new(1));
        world.add_country(Country::new(2));
        
        assert!(!world.are_allies(1, 2));
        
        world.add_alliance(1, 2);
        
        assert!(world.are_allies(1, 2));
        assert!(world.are_allies(2, 1));  // Symmetric
    }

    #[test]
    fn test_decision_system_creation() {
        let system = DecisionSystem::new();
        assert_eq!(system.get_tick(), 0);
    }

    #[test]
    fn test_decision_system_determinism() {
        let mut system1 = DecisionSystem::init(42);
        let mut system2 = DecisionSystem::init(42);
        
        // Add identical countries
        system1.add_country(1);
        system1.add_country(2);
        system2.add_country(1);
        system2.add_country(2);
        
        // Run one tick
        system1.tick();
        system2.tick();
        
        // Both should be at tick 1
        assert_eq!(system1.get_tick(), system2.get_tick());
    }

    #[test]
    fn test_tick_execution() {
        let mut system = DecisionSystem::new();
        system.add_country(1);
        system.add_country(2);
        system.add_edge(1, 2, 1, 0.5);
        
        assert_eq!(system.get_tick(), 0);
        
        system.tick();
        
        assert_eq!(system.get_tick(), 1);
        
        // Should have logs for both countries
        let logs = system.logs;
        assert_eq!(logs.len(), 2);
    }
}

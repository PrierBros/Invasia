/// Country state and edge relationship data
use serde::{Deserialize, Serialize};

/// Adaptive weights for decision scoring (§4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveWeights {
    pub alpha: i32,    // Resource gain weight
    pub beta: i32,     // Security weight
    pub gamma: i32,    // Growth weight
    pub delta: i32,    // Positional advantage weight
    pub kappa: i32,    // Cost weight
    pub rho: i32,      // Risk weight
}

impl AdaptiveWeights {
    /// Create new weights with baseline values
    pub fn new() -> Self {
        Self {
            alpha: 8,
            beta: 8,
            gamma: 8,
            delta: 4,
            kappa: 8,
            rho: 4,
        }
    }
    
    /// Update weights based on needs signals
    pub fn update(&mut self, resources: f32, threat_index: f32, growth: f32, ally_count: usize, recent_losses: f32) {
        // Resource weight: α_i = clamp(α0 * (1 + c_R * (R* - R_i)/R*), α_min, α_max)
        let r_target = 1000.0;
        let c_r = 0.5;
        let alpha_base = 8.0;
        let alpha_new = alpha_base * (1.0 + c_r * (r_target - resources) / r_target);
        self.alpha = (alpha_new.round() as i32).clamp(2, 16);
        
        // Security weight: β_i = clamp(β0 * (1 + c_T * TI_i/(1 + TI_i)), β_min, β_max)
        let c_t = 0.8;
        let beta_base = 8.0;
        let ti_norm = threat_index / (1.0 + threat_index);
        let beta_new = beta_base * (1.0 + c_t * ti_norm);
        self.beta = (beta_new.round() as i32).clamp(2, 16);
        
        // Growth weight: γ_i = clamp(γ0 * (1 + c_G * (G* - G_i)/G*), γ_min, γ_max)
        let g_target = 100.0;
        let c_g = 0.5;
        let gamma_base = 8.0;
        let gamma_new = gamma_base * (1.0 + c_g * (g_target - growth) / g_target);
        self.gamma = (gamma_new.round() as i32).clamp(2, 16);
        
        // Position weight: based on diplomatic isolation
        let delta_base = 4.0;
        let isolation_factor = if ally_count > 0 {
            1.0 / (ally_count as f32 + 1.0)
        } else {
            2.0
        };
        let delta_new = delta_base * isolation_factor;
        self.delta = (delta_new.round() as i32).clamp(2, 16);
        
        // Cost weight: relatively stable
        self.kappa = 8;
        
        // Risk weight: increase with recent losses
        let rho_base = 4.0;
        let loss_factor = 1.0 + (recent_losses / 100.0);
        let rho_new = rho_base * loss_factor;
        self.rho = (rho_new.round() as i32).clamp(2, 16);
    }
}

impl Default for AdaptiveWeights {
    fn default() -> Self {
        Self::new()
    }
}

/// Marginal values for research tech stats (§3.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginalValues {
    pub military: f32,
    pub economy: f32,
    pub tech: f32,
    pub diplomacy: f32,
}

impl MarginalValues {
    pub fn new() -> Self {
        Self {
            military: 1.0,
            economy: 1.0,
            tech: 1.0,
            diplomacy: 1.0,
        }
    }
    
    /// Update marginal values based on current country state
    pub fn update(&mut self, m_eff: f32, gdp: f32, tech_level: f32, prestige: f32) {
        // Higher marginal value when stat is lower (diminishing returns)
        self.military = 100.0 / (m_eff + 10.0);
        self.economy = 100.0 / (gdp + 10.0);
        self.tech = 50.0 / (tech_level + 5.0);
        self.diplomacy = 10.0 / (prestige + 1.0);
    }
}

impl Default for MarginalValues {
    fn default() -> Self {
        Self::new()
    }
}

/// Edge relationship between two countries (§8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryEdge {
    pub neighbor_id: u32,
    pub distance_bucket: usize,   // Discrete distance for kernel lookup
    pub terrain_penalty: f32,
    pub fortification: f32,
    pub border_length: f32,
    pub supply_diff: f32,
    pub hostility: f32,           // 0.0 to 1.0
    pub relations: f32,           // -100 to +100
}

impl CountryEdge {
    pub fn new(neighbor_id: u32) -> Self {
        Self {
            neighbor_id,
            distance_bucket: 1,
            terrain_penalty: 0.0,
            fortification: 0.0,
            border_length: 1.0,
            supply_diff: 0.0,
            hostility: 0.0,
            relations: 0.0,
        }
    }
}

/// Country state with cached features (§8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: u32,
    pub m_eff: f32,              // Effective military strength
    pub gdp: f32,                // Current GDP
    pub growth: f32,             // Growth rate
    pub prestige: f32,           // Prestige/influence
    pub morale: f32,             // Military morale
    pub tech_level: f32,         // Technology level
    pub resources: f32,          // Available resources
    pub threat_index: f32,       // Cached TI_i
    pub ally_count: usize,       // Number of allies
    pub recent_losses: f32,      // Recent casualty count
    
    // Adaptive components
    pub weights: AdaptiveWeights,
    pub marginal_values: MarginalValues,
    
    // Neighbors and edges
    pub edges: Vec<CountryEdge>,
    
    // Border tiles for fortify/move actions
    pub border_tiles: Vec<BorderTile>,
}

impl Country {
    /// Create a new country with default values
    pub fn new(id: u32) -> Self {
        Self {
            id,
            m_eff: 100.0,
            gdp: 100.0,
            growth: 5.0,
            prestige: 10.0,
            morale: 1.0,
            tech_level: 1.0,
            resources: 500.0,
            threat_index: 0.0,
            ally_count: 0,
            recent_losses: 0.0,
            weights: AdaptiveWeights::new(),
            marginal_values: MarginalValues::new(),
            edges: Vec::new(),
            border_tiles: Vec::new(),
        }
    }
    
    /// Add an edge to a neighbor
    pub fn add_edge(&mut self, edge: CountryEdge) {
        self.edges.push(edge);
    }
    
    /// Get edge to specific neighbor
    pub fn get_edge(&self, neighbor_id: u32) -> Option<&CountryEdge> {
        self.edges.iter().find(|e| e.neighbor_id == neighbor_id)
    }
    
    /// Get mutable edge to specific neighbor
    pub fn get_edge_mut(&mut self, neighbor_id: u32) -> Option<&mut CountryEdge> {
        self.edges.iter_mut().find(|e| e.neighbor_id == neighbor_id)
    }
}

/// Border tile for fortify/move actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderTile {
    pub id: u32,
    pub position_x: i32,
    pub position_y: i32,
    pub threat_gradient: f32,    // |∇TI| for prioritization
    pub fortification: f32,
    pub garrison_strength: f32,
}

impl BorderTile {
    pub fn new(id: u32, x: i32, y: i32) -> Self {
        Self {
            id,
            position_x: x,
            position_y: y,
            threat_gradient: 0.0,
            fortification: 0.0,
            garrison_strength: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_creation() {
        let country = Country::new(1);
        assert_eq!(country.id, 1);
        assert_eq!(country.m_eff, 100.0);
        assert_eq!(country.edges.len(), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut country = Country::new(1);
        let edge = CountryEdge::new(2);
        country.add_edge(edge);
        assert_eq!(country.edges.len(), 1);
        assert_eq!(country.edges[0].neighbor_id, 2);
    }

    #[test]
    fn test_adaptive_weights_update() {
        let country = Country::new(1);
        let mut weights = AdaptiveWeights::new();
        
        weights.update(200.0, 50.0, 20.0, 0, 0.0);
        
        // Should increase alpha (need resources)
        assert!(weights.alpha >= 8);
        
        // Should increase beta (high threat)
        assert!(weights.beta > 8);
        
        // All weights within bounds
        assert!(weights.alpha >= 2 && weights.alpha <= 16);
        assert!(weights.beta >= 2 && weights.beta <= 16);
        assert!(weights.gamma >= 2 && weights.gamma <= 16);
    }

    #[test]
    fn test_marginal_values_update() {
        let mut marginal_values = MarginalValues::new();
        
        marginal_values.update(10.0, 200.0, 1.0, 10.0);
        
        // Higher marginal value for military (lower stat)
        assert!(marginal_values.military > marginal_values.economy);
    }
}

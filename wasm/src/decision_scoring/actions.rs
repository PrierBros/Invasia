/// Action types and candidate generation
use serde::{Deserialize, Serialize};

/// Action types for countries (§2, §3)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Attack a neighboring country
    Attack { target_id: u32 },
    
    /// Invest in a specific sector
    Invest { sector: InvestSector },
    
    /// Research a technology
    Research { tech: TechType },
    
    /// Form alliance with neighbor
    Ally { target_id: u32 },
    
    /// Sign pact with neighbor
    Pact { target_id: u32 },
    
    /// Trade agreement with neighbor
    Trade { target_id: u32 },
    
    /// Fortify a border tile
    Fortify { tile_id: u32 },
    
    /// Move troops to border tile
    Move { tile_id: u32 },
    
    /// Do nothing (baseline)
    Pass,
}

impl Action {
    /// Get a string description of the action
    pub fn description(&self) -> String {
        match self {
            Action::Attack { target_id } => format!("Attack country {}", target_id),
            Action::Invest { sector } => format!("Invest in {:?}", sector),
            Action::Research { tech } => format!("Research {:?}", tech),
            Action::Ally { target_id } => format!("Ally with country {}", target_id),
            Action::Pact { target_id } => format!("Sign pact with country {}", target_id),
            Action::Trade { target_id } => format!("Trade with country {}", target_id),
            Action::Fortify { tile_id } => format!("Fortify tile {}", tile_id),
            Action::Move { tile_id } => format!("Move to tile {}", tile_id),
            Action::Pass => "Pass".to_string(),
        }
    }
}

/// Investment sectors (§3.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestSector {
    Infrastructure,
    Military,
    Economy,
    Technology,
}

/// Technology types (§3.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechType {
    MilitaryAdvancement,
    EconomicEfficiency,
    DiplomaticInfluence,
    TechnologicalBreakthrough,
}

/// Candidate pruning configuration (§5)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningConfig {
    pub k_attack: usize,      // Top K attacks by upper bound
    pub k_fortify: usize,     // Top K border tiles by threat gradient
    pub k_invest: usize,      // Top K sectors by ROI
    pub k_research: usize,    // Top K techs by marginal value
    pub k_diplomacy: usize,   // Up to K diplomatic actions
}

impl PruningConfig {
    pub fn new() -> Self {
        Self {
            k_attack: 3,
            k_fortify: 3,
            k_invest: 2,
            k_research: 2,
            k_diplomacy: 2,
        }
    }
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Candidate action with priority score for pruning
#[derive(Debug, Clone)]
pub struct ActionCandidate {
    pub action: Action,
    pub priority: f32,  // Upper bound or heuristic for pruning
}

impl ActionCandidate {
    pub fn new(action: Action, priority: f32) -> Self {
        Self { action, priority }
    }
}

/// Generate shortlist of candidate actions (§5)
pub fn generate_shortlist(
    _country_id: u32,
    country: &super::country::Country,
    world: &super::world::WorldState,
    config: &PruningConfig,
) -> Vec<Action> {
    let mut candidates = Vec::new();
    
    // Always include Pass
    candidates.push(Action::Pass);
    
    // Generate attack candidates (top K by upper bound of ΔSec + ΔRes)
    let mut attack_candidates = Vec::new();
    for edge in &country.edges {
        if let Some(neighbor) = world.get_country(edge.neighbor_id) {
            // Upper bound heuristic: resource gain + threat reduction
            let resource_upper = neighbor.resources * 0.5;  // Potential resource gain
            let threat_reduction = edge.hostility * neighbor.m_eff * 0.3;  // Threat reduction estimate
            let priority = resource_upper + threat_reduction;
            
            attack_candidates.push(ActionCandidate::new(
                Action::Attack { target_id: edge.neighbor_id },
                priority,
            ));
        }
    }
    // Sort by priority and take top K
    attack_candidates.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    candidates.extend(
        attack_candidates.iter()
            .take(config.k_attack)
            .map(|c| c.action.clone())
    );
    
    // Generate fortify/move candidates (top K by |∇TI|)
    let mut fortify_candidates = Vec::new();
    for tile in &country.border_tiles {
        fortify_candidates.push(ActionCandidate::new(
            Action::Fortify { tile_id: tile.id },
            tile.threat_gradient.abs(),
        ));
    }
    fortify_candidates.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    candidates.extend(
        fortify_candidates.iter()
            .take(config.k_fortify)
            .map(|c| c.action.clone())
    );
    
    // Generate invest candidates (top K by ROI estimate)
    let invest_sectors = [
        InvestSector::Infrastructure,
        InvestSector::Military,
        InvestSector::Economy,
        InvestSector::Technology,
    ];
    let mut invest_candidates = Vec::new();
    for sector in &invest_sectors {
        // Simple ROI heuristic based on current needs
        let roi = match sector {
            InvestSector::Military => country.marginal_values.military,
            InvestSector::Economy => country.marginal_values.economy,
            InvestSector::Technology => country.marginal_values.tech,
            InvestSector::Infrastructure => country.marginal_values.economy * 0.5,
        };
        invest_candidates.push(ActionCandidate::new(
            Action::Invest { sector: *sector },
            roi,
        ));
    }
    invest_candidates.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    candidates.extend(
        invest_candidates.iter()
            .take(config.k_invest)
            .map(|c| c.action.clone())
    );
    
    // Generate research candidates (top K by Σ m_tq * MV_q)
    let tech_types = [
        TechType::MilitaryAdvancement,
        TechType::EconomicEfficiency,
        TechType::DiplomaticInfluence,
        TechType::TechnologicalBreakthrough,
    ];
    let mut research_candidates = Vec::new();
    for tech in &tech_types {
        // Marginal value weighted by tech impact
        let mv_weighted = match tech {
            TechType::MilitaryAdvancement => country.marginal_values.military * 1.5,
            TechType::EconomicEfficiency => country.marginal_values.economy * 1.5,
            TechType::DiplomaticInfluence => country.marginal_values.diplomacy * 1.5,
            TechType::TechnologicalBreakthrough => country.marginal_values.tech * 2.0,
        };
        research_candidates.push(ActionCandidate::new(
            Action::Research { tech: *tech },
            mv_weighted,
        ));
    }
    research_candidates.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    candidates.extend(
        research_candidates.iter()
            .take(config.k_research)
            .map(|c| c.action.clone())
    );
    
    // Generate diplomacy candidates (up to K with improving stance)
    let mut diplo_candidates = Vec::new();
    for edge in &country.edges {
        // Consider diplomacy if relations are neutral to positive or if strategically valuable
        if edge.relations >= -20.0 {
            let priority = edge.relations + 50.0;  // Favor better relations
            diplo_candidates.push(ActionCandidate::new(
                Action::Ally { target_id: edge.neighbor_id },
                priority,
            ));
        }
    }
    diplo_candidates.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    candidates.extend(
        diplo_candidates.iter()
            .take(config.k_diplomacy)
            .map(|c| c.action.clone())
    );
    
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision_scoring::country::{Country, CountryEdge};
    use crate::decision_scoring::world::WorldState;

    #[test]
    fn test_action_description() {
        let action = Action::Attack { target_id: 2 };
        assert_eq!(action.description(), "Attack country 2");
        
        let action = Action::Pass;
        assert_eq!(action.description(), "Pass");
    }

    #[test]
    fn test_pruning_config() {
        let config = PruningConfig::default();
        assert_eq!(config.k_attack, 3);
        assert_eq!(config.k_fortify, 3);
    }

    #[test]
    fn test_generate_shortlist_includes_pass() {
        let country = Country::new(1);
        let world = WorldState::new();
        let config = PruningConfig::default();
        
        let shortlist = generate_shortlist(1, &country, &world, &config);
        
        // Should always include Pass
        assert!(shortlist.iter().any(|a| matches!(a, Action::Pass)));
    }

    #[test]
    fn test_generate_shortlist_attack_candidates() {
        let mut country = Country::new(1);
        let mut world = WorldState::new();
        
        // Add neighbors
        let mut neighbor1 = Country::new(2);
        neighbor1.resources = 1000.0;
        let mut neighbor2 = Country::new(3);
        neighbor2.resources = 500.0;
        
        world.add_country(neighbor1);
        world.add_country(neighbor2);
        
        let mut edge1 = CountryEdge::new(2);
        edge1.hostility = 0.8;
        let mut edge2 = CountryEdge::new(3);
        edge2.hostility = 0.2;
        
        country.add_edge(edge1);
        country.add_edge(edge2);
        
        let config = PruningConfig::default();
        let shortlist = generate_shortlist(1, &country, &world, &config);
        
        // Should include some attack actions
        let attack_count = shortlist.iter().filter(|a| matches!(a, Action::Attack { .. })).count();
        assert!(attack_count > 0);
        assert!(attack_count <= config.k_attack);
    }
}

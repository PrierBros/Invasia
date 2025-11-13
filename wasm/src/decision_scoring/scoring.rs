/// Decision scoring system (§1, §2, §3)
use serde::{Deserialize, Serialize};
use super::actions::*;
use super::country::*;
use super::luts::*;
use super::world::WorldState;

/// Six-channel score components (§1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreComponents {
    pub delta_res: f32,      // Resource gain
    pub delta_sec: f32,      // Security/threat reduction
    pub delta_growth: f32,   // Growth trajectory delta
    pub delta_pos: f32,      // Positional/diplomatic advantage
    pub cost: f32,           // Immediate costs
    pub risk: f32,           // Outcome uncertainty penalty
}

impl ScoreComponents {
    pub fn zero() -> Self {
        Self {
            delta_res: 0.0,
            delta_sec: 0.0,
            delta_growth: 0.0,
            delta_pos: 0.0,
            cost: 0.0,
            risk: 0.0,
        }
    }
    
    /// Compute final score with adaptive weights (§1)
    pub fn final_score(&self, weights: &AdaptiveWeights) -> f32 {
        let alpha = weights.alpha as f32;
        let beta = weights.beta as f32;
        let gamma = weights.gamma as f32;
        let delta = weights.delta as f32;
        let kappa = weights.kappa as f32;
        let rho = weights.rho as f32;
        
        alpha * self.delta_res +
        beta * self.delta_sec +
        gamma * self.delta_growth +
        delta * self.delta_pos -
        kappa * self.cost -
        rho * self.risk
    }
}

/// Compute threat index for a country (§2)
pub fn compute_threat_index(
    country: &Country,
    world: &WorldState,
    luts: &LookupTables,
) -> f32 {
    let mut threat = 0.0;
    
    for edge in &country.edges {
        if let Some(neighbor) = world.get_country(edge.neighbor_id) {
            let kernel = luts.distance_kernel.get(edge.distance_bucket);
            
            // Check if neighbor is an ally
            let is_ally = world.are_allies(country.id, neighbor.id);
            
            if is_ally {
                // Allies reduce threat
                threat -= kernel * neighbor.m_eff;
            } else {
                // Enemies contribute to threat based on hostility
                threat += kernel * neighbor.m_eff * edge.hostility;
            }
        }
    }
    
    threat
}

/// Score an attack action (§3.1)
pub fn score_attack(
    attacker: &Country,
    defender_id: u32,
    world: &WorldState,
    luts: &LookupTables,
) -> ScoreComponents {
    let mut comp = ScoreComponents::zero();
    
    // Get defender
    let defender = match world.get_country(defender_id) {
        Some(d) => d,
        None => return comp,
    };
    
    // Get edge
    let edge = match attacker.get_edge(defender_id) {
        Some(e) => e,
        None => return comp,
    };
    
    // Compute effective force ratio (§3.1)
    let g_penalty = 1.0 + edge.terrain_penalty;
    let fr = attacker.m_eff / (defender.m_eff * g_penalty);
    
    // Win probability using sigmoid
    let ln_fr = luts.log_ratio.lookup(fr);
    let b_fort = 0.3;
    let b_terr = 0.2;
    let b_dist = 0.1;
    let lambda = 1.5;
    
    let logit = lambda * (
        ln_fr
        - b_fort * edge.fortification
        - b_terr * edge.terrain_penalty
        - b_dist * (edge.distance_bucket as f32)
    );
    
    let p_win = luts.sigmoid.lookup(logit);
    
    // Expected values
    let v_win_res = defender.resources * 0.5;  // Gain half of defender's resources
    let v_win_sec = edge.hostility * defender.m_eff * 0.8;  // Threat reduction
    let v_win_pos = defender.prestige * 0.3;  // Prestige gain
    
    let v_loss_res = -attacker.resources * 0.1;  // Lose some resources
    let v_loss_sec = -defender.m_eff * 0.2;  // Increase in relative threat
    let v_loss_pos = -attacker.prestige * 0.1;  // Prestige loss
    
    comp.delta_res = p_win * v_win_res + (1.0 - p_win) * v_loss_res;
    comp.delta_sec = p_win * v_win_sec + (1.0 - p_win) * v_loss_sec;
    comp.delta_pos = p_win * v_win_pos + (1.0 - p_win) * v_loss_pos;
    
    // Risk: uncertainty penalty (§3.1)
    let s_risk = 8.0;
    comp.risk = s_risk * p_win * (1.0 - p_win);
    
    // Cost: casualties, upkeep, diplomatic penalty (§3.1)
    let c_cas = 0.5;
    let c_upkeep = 0.2;
    let c_dipl = 0.3;
    let e_casualties = attacker.m_eff * 0.1 * (1.0 - p_win + 0.5);
    let delta_upkeep = defender.m_eff * 0.05;  // Occupation costs
    let dipl_penalty = edge.relations.max(0.0) * 0.5;  // Penalty for attacking friends
    
    comp.cost = c_cas * e_casualties + c_upkeep * delta_upkeep + c_dipl * dipl_penalty;
    
    // Normalize to target ranges [-32, +32] for deltas, [0, 16] for cost/risk
    comp.delta_res = (comp.delta_res / 50.0).clamp(-32.0, 32.0);
    comp.delta_sec = (comp.delta_sec / 50.0).clamp(-32.0, 32.0);
    comp.delta_pos = (comp.delta_pos / 20.0).clamp(-32.0, 32.0);
    comp.cost = (comp.cost / 20.0).clamp(0.0, 16.0);
    comp.risk = comp.risk.clamp(0.0, 16.0);
    
    comp
}

/// Score an invest action (§3.2)
pub fn score_invest(
    country: &Country,
    sector: InvestSector,
    luts: &LookupTables,
) -> ScoreComponents {
    let mut comp = ScoreComponents::zero();
    
    // Compute ROI over horizon H
    let h = 8;  // Short horizon
    let mut roi = 0.0;
    
    // Base GDP increase per sector
    let gdp_boost = match sector {
        InvestSector::Economy => 5.0,
        InvestSector::Infrastructure => 3.0,
        InvestSector::Technology => 4.0,
        InvestSector::Military => 2.0,
    };
    
    // Discounted future value
    for horizon in 1..=h {
        let discount = luts.discount.get(horizon);
        let delta_gdp = gdp_boost * (1.0 + country.growth / 100.0).powi(horizon as i32);
        roi += discount * delta_gdp;
    }
    roi /= h as f32;
    
    comp.delta_growth = roi;
    
    // Cost varies by sector
    let base_cost = match sector {
        InvestSector::Economy => 20.0,
        InvestSector::Infrastructure => 30.0,
        InvestSector::Technology => 25.0,
        InvestSector::Military => 15.0,
    };
    comp.cost = base_cost / country.resources.max(10.0);
    
    // Risk is low for investments
    comp.risk = 1.0;
    
    // Normalize
    comp.delta_growth = (comp.delta_growth / 10.0).clamp(-32.0, 32.0);
    comp.cost = (comp.cost * 10.0).clamp(0.0, 16.0);
    
    comp
}

/// Score a research action (§3.3)
pub fn score_research(
    country: &Country,
    tech: TechType,
) -> ScoreComponents {
    let mut comp = ScoreComponents::zero();
    
    // Marginal value weighted by tech multipliers
    let mv = &country.marginal_values;
    let delta_growth = match tech {
        TechType::MilitaryAdvancement => mv.military * 1.5,
        TechType::EconomicEfficiency => mv.economy * 1.8,
        TechType::DiplomaticInfluence => mv.diplomacy * 1.2,
        TechType::TechnologicalBreakthrough => mv.tech * 2.0,
    };
    
    comp.delta_growth = delta_growth;
    
    // Research cost (RP_t)
    let rp_cost = match tech {
        TechType::MilitaryAdvancement => 30.0,
        TechType::EconomicEfficiency => 25.0,
        TechType::DiplomaticInfluence => 20.0,
        TechType::TechnologicalBreakthrough => 40.0,
    };
    comp.cost = rp_cost / country.resources.max(10.0);
    
    // Risk is zero for research
    comp.risk = 0.0;
    
    // Normalize
    comp.delta_growth = (comp.delta_growth / 5.0).clamp(-32.0, 32.0);
    comp.cost = (comp.cost * 10.0).clamp(0.0, 16.0);
    
    comp
}

/// Score a diplomacy action (§3.4)
pub fn score_diplomacy(
    country: &Country,
    target_id: u32,
    action_type: DiplomacyType,
    world: &WorldState,
    luts: &LookupTables,
) -> ScoreComponents {
    let mut comp = ScoreComponents::zero();
    
    let target = match world.get_country(target_id) {
        Some(t) => t,
        None => return comp,
    };
    
    let _edge = match country.get_edge(target_id) {
        Some(e) => e,
        None => return comp,
    };
    
    // Estimate target's score for accepting
    let score_with = estimate_alliance_benefit(target, country);
    let score_without = 0.0;  // Status quo baseline
    
    let theta = 0.5;
    let logit = theta * (score_with - score_without);
    let p_accept = luts.sigmoid.lookup(logit);
    
    // Benefits if accepted
    match action_type {
        DiplomacyType::Ally => {
            comp.delta_sec = target.m_eff * 0.5;  // Ally military strength helps
            comp.delta_pos = 5.0;  // Diplomatic positioning
        }
        DiplomacyType::Pact => {
            comp.delta_sec = target.m_eff * 0.3;
            comp.delta_pos = 3.0;
        }
        DiplomacyType::Trade => {
            comp.delta_res = target.gdp * 0.1;  // Trade benefits
            comp.delta_growth = 2.0;
        }
    }
    
    // Cost: commitment cost
    comp.cost = 5.0;
    comp.risk = 2.0;  // Some diplomatic risk
    
    // Multiply by acceptance probability
    comp.delta_sec *= p_accept;
    comp.delta_pos *= p_accept;
    comp.delta_res *= p_accept;
    comp.delta_growth *= p_accept;
    
    // Normalize
    comp.delta_sec = (comp.delta_sec / 50.0).clamp(-32.0, 32.0);
    comp.delta_pos = (comp.delta_pos / 5.0).clamp(-32.0, 32.0);
    comp.delta_res = (comp.delta_res / 50.0).clamp(-32.0, 32.0);
    comp.delta_growth = (comp.delta_growth / 5.0).clamp(-32.0, 32.0);
    comp.cost = comp.cost.clamp(0.0, 16.0);
    
    comp
}

#[derive(Debug, Clone, Copy)]
pub enum DiplomacyType {
    Ally,
    Pact,
    Trade,
}

fn estimate_alliance_benefit(_target: &Country, proposer: &Country) -> f32 {
    // Simple heuristic: military strength + diplomatic value
    proposer.m_eff * 0.2 + proposer.prestige * 0.1
}

/// Score a fortify action (§3.5)
pub fn score_fortify(
    country: &Country,
    tile_id: u32,
) -> ScoreComponents {
    let mut comp = ScoreComponents::zero();
    
    let tile = match country.border_tiles.iter().find(|t| t.id == tile_id) {
        Some(t) => t,
        None => return comp,
    };
    
    // Security improvement based on threat gradient
    comp.delta_sec = tile.threat_gradient * 0.5;
    
    // Cost of fortification
    comp.cost = 3.0;
    comp.risk = 0.5;
    
    // Normalize
    comp.delta_sec = (comp.delta_sec / 10.0).clamp(-32.0, 32.0);
    
    comp
}

/// Score a move action (§3.5)
pub fn score_move(
    country: &Country,
    tile_id: u32,
) -> ScoreComponents {
    let mut comp = ScoreComponents::zero();
    
    let tile = match country.border_tiles.iter().find(|t| t.id == tile_id) {
        Some(t) => t,
        None => return comp,
    };
    
    // Security and positioning improvement
    comp.delta_sec = tile.threat_gradient * 0.3;
    comp.delta_pos = tile.threat_gradient * 0.2;
    
    // Cost of movement
    comp.cost = 2.0;
    comp.risk = 1.0;
    
    // Normalize
    comp.delta_sec = (comp.delta_sec / 10.0).clamp(-32.0, 32.0);
    comp.delta_pos = (comp.delta_pos / 10.0).clamp(-32.0, 32.0);
    
    comp
}

/// Score any action (dispatch to specific scoring functions)
pub fn score_action(
    country: &Country,
    action: &Action,
    world: &WorldState,
    luts: &LookupTables,
) -> ScoreComponents {
    match action {
        Action::Attack { target_id } => score_attack(country, *target_id, world, luts),
        Action::Invest { sector } => score_invest(country, *sector, luts),
        Action::Research { tech } => score_research(country, *tech),
        Action::Ally { target_id } => score_diplomacy(country, *target_id, DiplomacyType::Ally, world, luts),
        Action::Pact { target_id } => score_diplomacy(country, *target_id, DiplomacyType::Pact, world, luts),
        Action::Trade { target_id } => score_diplomacy(country, *target_id, DiplomacyType::Trade, world, luts),
        Action::Fortify { tile_id } => score_fortify(country, *tile_id),
        Action::Move { tile_id } => score_move(country, *tile_id),
        Action::Pass => ScoreComponents::zero(),  // Pass has zero change
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_components_final_score() {
        let mut comp = ScoreComponents::zero();
        comp.delta_res = 10.0;
        comp.delta_sec = 5.0;
        comp.cost = 3.0;
        
        let weights = AdaptiveWeights::new();
        let score = comp.final_score(&weights);
        
        // Score = 8*10 + 8*5 + 0 + 0 - 8*3 - 0 = 80 + 40 - 24 = 96
        assert!((score - 96.0).abs() < 0.1);
    }

    #[test]
    fn test_score_invest() {
        let country = Country::new(1);
        let luts = LookupTables::new();
        
        let comp = score_invest(&country, InvestSector::Economy, &luts);
        
        // Should have positive growth delta
        assert!(comp.delta_growth > 0.0);
        
        // Should have some cost
        assert!(comp.cost > 0.0);
        
        // Risk should be low
        assert!(comp.risk < 5.0);
    }

    #[test]
    fn test_score_research() {
        let mut country = Country::new(1);
        country.marginal_values.tech = 5.0;
        
        let comp = score_research(&country, TechType::TechnologicalBreakthrough);
        
        // Should have positive growth delta
        assert!(comp.delta_growth > 0.0);
        
        // Risk should be zero for research
        assert_eq!(comp.risk, 0.0);
    }

    #[test]
    fn test_score_pass() {
        let country = Country::new(1);
        let world = WorldState::new();
        let luts = LookupTables::new();
        
        let comp = score_action(&country, &Action::Pass, &world, &luts);
        
        // All components should be zero
        assert_eq!(comp.delta_res, 0.0);
        assert_eq!(comp.delta_sec, 0.0);
        assert_eq!(comp.cost, 0.0);
    }
}

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEntity {
    pub id: u32,
    pub health: f32,
    pub military_strength: f32,
    pub position_x: f32,
    pub position_y: f32,
    pub state: AiState,
    pub territory: f32,
    pub money: f32,
    #[serde(skip)]
    rng_state: u32,
}

impl AiEntity {
    pub fn new(id: u32) -> Self {
        let id_seed = id as f32;
        let variation = ((id_seed * 0.7321).sin() + 1.0) / 2.0;
        let initial_military_strength = 50.0 + (variation * 50.0);

        let health_variation = ((id_seed * 1.234).cos() + 1.0) / 2.0;
        let initial_health = 70.0 + (health_variation * 30.0);

        let money_variation = ((id_seed * 3.141).sin() + 1.0) / 2.0;
        let initial_money = 100.0 + (money_variation * 100.0);

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

        let x_seed = ((id_seed * 0.3371).sin() + (id_seed * 0.0157).sin()) * 0.5;
        let y_seed = ((id_seed * 0.4219).cos() + (id_seed * 0.0213).cos()) * 0.5;

        let spawn_x = x_seed * 1200.0;
        let spawn_y = y_seed * 1200.0;

        Self {
            id,
            health: initial_health,
            military_strength: initial_military_strength,
            position_x: spawn_x,
            position_y: spawn_y,
            state: initial_state,
            territory: 10.0,
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
    pub fn next_random(&mut self) -> f32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        if x == 0 {
            x = 1;
        }
        self.rng_state = x;
        const INV_U32_MAX: f32 = 1.0 / (u32::MAX as f32);
        (self.rng_state as f32) * INV_U32_MAX
    }

    #[inline]
    pub fn next_variation(&mut self) -> f32 {
        let a = 0.5 + self.next_random();
        let b = 0.5 + self.next_random();
        a * b
    }

    #[inline]
    pub fn random_symmetric(&mut self) -> f32 {
        self.next_random() * 2.0 - 1.0
    }
}

#[derive(Clone, Copy)]
pub struct EntitySnapshot {
    pub position_x: f32,
    pub position_y: f32,
    pub state: AiState,
    pub military_strength: f32,
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

impl AiEntity {
    pub(crate) fn update(
        &mut self,
        _tick: u64,
        self_index: usize,
        self_snapshot: EntitySnapshot,
        entity_snapshots: &[EntitySnapshot],
        grid: &crate::dependency::SpatialGrid,
    ) {
        if self.state == AiState::Dead {
            return;
        }

        let mut variation = self.next_variation();
        if variation < 0.25 {
            variation = 0.25;
        }

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

                let movement_x = self.random_symmetric() * 2.0 * variation;
                let movement_y = self.random_symmetric() * 2.0 * variation;

                let new_x = self.position_x + movement_x;
                let new_y = self.position_y + movement_y;

                const WORLD_BOUND: f32 = 1230.0;
                self.position_x = new_x.clamp(-WORLD_BOUND, WORLD_BOUND);
                self.position_y = new_y.clamp(-WORLD_BOUND, WORLD_BOUND);

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

        let mut total_damage = 0.0;
        grid.for_each_neighbor(
            self_snapshot.position_x,
            self_snapshot.position_y,
            |other_index| {
                if other_index == self_index {
                    return;
                }
                debug_assert!(other_index < entity_snapshots.len());
                let other = unsafe { entity_snapshots.get_unchecked(other_index) };
                debug_assert_eq!(
                    other.state,
                    AiState::Active,
                    "Spatial grid should only contain Active entities",
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

        if total_damage > 0.0 {
            self.health = (self.health - total_damage).max(0.0);
        } else if self.health < 100.0 {
            self.health = (self.health + 0.05 * variation).min(100.0);
        }
    }
}

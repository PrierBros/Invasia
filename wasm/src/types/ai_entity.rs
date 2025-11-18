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

    fn seed_rng(id: u32) -> u32 {
        let mut seed = id.wrapping_mul(747_796_405).wrapping_add(2_891_336_453) ^ 0xA511_E9B3;
        if seed == 0 {
            seed = 1;
        }
        seed
    }
}

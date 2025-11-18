use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "u32", from = "u32")]
pub enum AiState {
    Idle = 0,
    Attacking = 1,
    Defending = 2,
    Dead = 3,
}

impl From<AiState> for u32 {
    fn from(state: AiState) -> u32 {
        state as u32
    }
}

impl From<u32> for AiState {
    fn from(value: u32) -> AiState {
        match value {
            1 => AiState::Attacking,
            2 => AiState::Defending,
            3 => AiState::Dead,
            _ => AiState::Idle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEntity {
    pub id: u32,
    pub military_strength: f32,
    pub position_x: f32,
    pub position_y: f32,
    pub state: AiState,
    pub territory: u32, // Number of grid spaces owned
    pub money: f32,
    #[serde(skip)]
    rng_state: u32,
    #[serde(skip)]
    pub last_update_time: f64, // For time-based resource accumulation
}

impl AiEntity {
    pub fn new(id: u32) -> Self {
        let id_seed = id as f32;
        
        // Deterministic position generation based on ID
        let x_seed = ((id_seed * 0.3371).sin() + (id_seed * 0.0157).sin()) * 0.5;
        let y_seed = ((id_seed * 0.4219).cos() + (id_seed * 0.0213).cos()) * 0.5;

        let spawn_x = x_seed * 1200.0;
        let spawn_y = y_seed * 1200.0;

        Self {
            id,
            military_strength: 10.0, // All AIs start with 10 military strength
            position_x: spawn_x,
            position_y: spawn_y,
            state: AiState::Idle,
            territory: 1, // All AIs start with 1 grid space
            money: 0.0,   // All AIs start with 0 money
            rng_state: Self::seed_rng(id),
            last_update_time: 0.0,
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

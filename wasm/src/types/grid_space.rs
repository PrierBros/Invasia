/// Represents a grid space in the world
#[derive(Debug, Clone, Copy)]
pub struct GridSpace {
    /// ID of the AI that owns this space (None if unowned)
    pub owner_id: Option<u32>,
    /// Defense strength accumulated on this space
    pub defense_strength: f32,
}

impl GridSpace {
    pub fn new() -> Self {
        Self {
            owner_id: None,
            defense_strength: 0.0,
        }
    }

    pub fn with_owner(owner_id: u32, defense_strength: f32) -> Self {
        Self {
            owner_id: Some(owner_id),
            defense_strength,
        }
    }
}

impl Default for GridSpace {
    fn default() -> Self {
        Self::new()
    }
}

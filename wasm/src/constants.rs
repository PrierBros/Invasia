// Shared simulation constants

// Resource generation rates per grid space per second
pub const MILITARY_STRENGTH_PER_SPACE_PER_SEC: f32 = 0.5;
pub const MONEY_PER_SPACE_PER_SEC: f32 = 1.0;

// Combat costs and bonuses
pub const ATTACK_COST: f32 = 10.0; // Cost to attempt conquering a grid space
pub const DEFENSE_BONUS_MULTIPLIER: f32 = 1.5; // Defense bonus when defending
pub const DEFENSE_ACCUMULATION: f32 = 1.0; // Defense strength added per defending AI per tick
pub const MAX_DEFENSE_STRENGTH: f32 = 50.0; // Maximum defense strength cap

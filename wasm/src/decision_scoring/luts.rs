/// Lookup Tables (LUTs) for efficient computation
/// All scoring uses fixed, precomputed LUTs and integer/fixed-point arithmetic

use serde::{Deserialize, Serialize};

/// Sigmoid lookup table for logistic function over bounded range [-4, +4]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmoidLUT {
    table: Vec<f32>,
    min_x: f32,
    max_x: f32,
    step: f32,
}

impl SigmoidLUT {
    /// Create a new sigmoid LUT with specified range and resolution
    pub fn new(min_x: f32, max_x: f32, steps: usize) -> Self {
        let step = (max_x - min_x) / (steps - 1) as f32;
        let mut table = Vec::with_capacity(steps);
        
        for i in 0..steps {
            let x = min_x + i as f32 * step;
            // Sigmoid: σ(x) = 1 / (1 + e^(-x))
            let value = 1.0 / (1.0 + (-x).exp());
            table.push(value);
        }
        
        Self {
            table,
            min_x,
            max_x,
            step,
        }
    }
    
    /// Lookup sigmoid value with linear interpolation
    pub fn lookup(&self, x: f32) -> f32 {
        // Clamp to range
        let x_clamped = x.clamp(self.min_x, self.max_x);
        
        // Find position in table
        let pos = (x_clamped - self.min_x) / self.step;
        let idx = pos.floor() as usize;
        
        // Linear interpolation
        if idx >= self.table.len() - 1 {
            self.table[self.table.len() - 1]
        } else {
            let frac = pos - idx as f32;
            self.table[idx] * (1.0 - frac) + self.table[idx + 1] * frac
        }
    }
}

impl Default for SigmoidLUT {
    fn default() -> Self {
        // Default range [-4, +4] with 256 steps
        Self::new(-4.0, 4.0, 256)
    }
}

/// Log-ratio lookup table for force ratios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRatioLUT {
    table: Vec<f32>,
    min_ratio: f32,
    max_ratio: f32,
    step: f32,
}

impl LogRatioLUT {
    /// Create a new log-ratio LUT for force ratios [0.25, 4]
    pub fn new(min_ratio: f32, max_ratio: f32, steps: usize) -> Self {
        let step = (max_ratio - min_ratio) / (steps - 1) as f32;
        let mut table = Vec::with_capacity(steps);
        
        for i in 0..steps {
            let ratio = min_ratio + i as f32 * step;
            // ln(ratio), clamped outside valid range
            let value = if ratio > 0.0 {
                ratio.ln()
            } else {
                f32::NEG_INFINITY
            };
            table.push(value);
        }
        
        Self {
            table,
            min_ratio,
            max_ratio,
            step,
        }
    }
    
    /// Lookup log ratio with linear interpolation
    pub fn lookup(&self, ratio: f32) -> f32 {
        // Clamp to range
        let ratio_clamped = ratio.clamp(self.min_ratio, self.max_ratio);
        
        // Find position in table
        let pos = (ratio_clamped - self.min_ratio) / self.step;
        let idx = pos.floor() as usize;
        
        // Linear interpolation
        if idx >= self.table.len() - 1 {
            self.table[self.table.len() - 1]
        } else {
            let frac = pos - idx as f32;
            self.table[idx] * (1.0 - frac) + self.table[idx + 1] * frac
        }
    }
}

impl Default for LogRatioLUT {
    fn default() -> Self {
        // Default range [0.25, 4.0] with 256 steps
        Self::new(0.25, 4.0, 256)
    }
}

/// Discount factor lookup table for growth projections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountLUT {
    factors: Vec<f32>,
    discount_rate: f32,
}

impl DiscountLUT {
    /// Create a new discount LUT with specified rate and horizon
    pub fn new(discount_rate: f32, horizon: usize) -> Self {
        let mut factors = Vec::with_capacity(horizon);
        
        for h in 1..=horizon {
            // d^h where d is discount factor (e.g., 0.95)
            let factor = discount_rate.powi(h as i32);
            factors.push(factor);
        }
        
        Self {
            factors,
            discount_rate,
        }
    }
    
    /// Get discount factor for given horizon (1-indexed)
    pub fn get(&self, horizon: usize) -> f32 {
        if horizon == 0 || horizon > self.factors.len() {
            0.0
        } else {
            self.factors[horizon - 1]
        }
    }
    
    /// Get all discount factors
    pub fn factors(&self) -> &[f32] {
        &self.factors
    }
}

impl Default for DiscountLUT {
    fn default() -> Self {
        // Default: 95% discount rate, 16 horizon steps
        Self::new(0.95, 16)
    }
}

/// Distance kernel lookup table for threat computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceKernelLUT {
    kernels: Vec<f32>,
    max_distance: usize,
}

impl DistanceKernelLUT {
    /// Create a new distance kernel LUT with exponential decay
    pub fn new(max_distance: usize, decay_rate: f32) -> Self {
        let mut kernels = Vec::with_capacity(max_distance + 1);
        
        for d in 0..=max_distance {
            // K(d) = exp(-decay_rate * d), monotone decreasing
            let kernel = (-decay_rate * d as f32).exp();
            kernels.push(kernel);
        }
        
        Self {
            kernels,
            max_distance,
        }
    }
    
    /// Get kernel value for distance bucket
    pub fn get(&self, distance: usize) -> f32 {
        if distance > self.max_distance {
            0.0
        } else {
            self.kernels[distance]
        }
    }
}

impl Default for DistanceKernelLUT {
    fn default() -> Self {
        // Default: max distance 20, decay rate 0.2
        Self::new(20, 0.2)
    }
}

/// Complete LUT collection for AI decision system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupTables {
    pub sigmoid: SigmoidLUT,
    pub log_ratio: LogRatioLUT,
    pub discount: DiscountLUT,
    pub distance_kernel: DistanceKernelLUT,
}

impl LookupTables {
    /// Create new lookup tables with default values
    pub fn new() -> Self {
        Self {
            sigmoid: SigmoidLUT::default(),
            log_ratio: LogRatioLUT::default(),
            discount: DiscountLUT::default(),
            distance_kernel: DistanceKernelLUT::default(),
        }
    }
}

impl Default for LookupTables {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid_lut() {
        let lut = SigmoidLUT::default();
        
        // σ(0) ≈ 0.5
        assert!((lut.lookup(0.0) - 0.5).abs() < 0.01);
        
        // σ(-4) ≈ 0.018
        assert!(lut.lookup(-4.0) < 0.05);
        
        // σ(4) ≈ 0.982
        assert!(lut.lookup(4.0) > 0.95);
        
        // Clamps outside range
        assert!(lut.lookup(-10.0) < 0.05);
        assert!(lut.lookup(10.0) > 0.95);
    }

    #[test]
    fn test_log_ratio_lut() {
        let lut = LogRatioLUT::default();
        
        // ln(1) = 0
        assert!((lut.lookup(1.0) - 0.0).abs() < 0.01);
        
        // ln(0.25) ≈ -1.386
        assert!((lut.lookup(0.25) + 1.386).abs() < 0.1);
        
        // ln(4) ≈ 1.386
        assert!((lut.lookup(4.0) - 1.386).abs() < 0.1);
    }

    #[test]
    fn test_discount_lut() {
        let lut = DiscountLUT::new(0.9, 8);
        
        // d^1 = 0.9
        assert!((lut.get(1) - 0.9).abs() < 0.01);
        
        // d^2 = 0.81
        assert!((lut.get(2) - 0.81).abs() < 0.01);
        
        // Out of range returns 0
        assert_eq!(lut.get(0), 0.0);
        assert_eq!(lut.get(9), 0.0);
    }

    #[test]
    fn test_distance_kernel_lut() {
        let lut = DistanceKernelLUT::new(10, 0.2);
        
        // K(0) = 1
        assert!((lut.get(0) - 1.0).abs() < 0.01);
        
        // K(d) decreases with distance
        assert!(lut.get(1) < lut.get(0));
        assert!(lut.get(2) < lut.get(1));
        
        // Out of range returns 0
        assert_eq!(lut.get(11), 0.0);
    }
}

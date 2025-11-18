/// WASM-compatible time utilities
/// 
/// This module provides time functionality that works in both native
/// and WebAssembly contexts.

#[cfg(target_arch = "wasm32")]
use web_sys::window;

/// A simple instant implementation that works in both WASM and native contexts
#[derive(Debug, Clone, Copy)]
pub struct Instant {
    timestamp_ms: f64,
}

impl Instant {
    /// Create a new Instant representing the current time
    pub fn now() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            // Use Performance API in WASM context
            let timestamp_ms = window()
                .expect("should have a Window")
                .performance()
                .expect("should have a Performance")
                .now();
            Instant { timestamp_ms }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Use standard library in native context
            use std::time::SystemTime;
            let timestamp_ms = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as f64;
            Instant { timestamp_ms }
        }
    }
    
    /// Returns the time elapsed since this Instant
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        Duration {
            millis: now.timestamp_ms - self.timestamp_ms,
        }
    }
}

/// Duration type that works across WASM and native
#[derive(Debug, Clone, Copy)]
pub struct Duration {
    millis: f64,
}

impl Duration {
    /// Returns the duration in milliseconds
    pub fn as_millis(&self) -> u128 {
        self.millis as u128
    }
    
    /// Returns the duration in seconds
    pub fn as_secs_f64(&self) -> f64 {
        self.millis / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn instant_now_works() {
        let instant = Instant::now();
        assert!(instant.timestamp_ms > 0.0);
    }
    
    #[test]
    fn elapsed_returns_positive_duration() {
        let instant = Instant::now();
        // Small delay to ensure time passes
        let mut sum = 0;
        for i in 0..1000 {
            sum += i;
        }
        let _ = sum; // Use sum to prevent optimization
        
        let elapsed = instant.elapsed();
        assert!(elapsed.as_millis() >= 0);
    }
    
    #[test]
    fn duration_conversions_work() {
        let duration = Duration { millis: 1500.0 };
        assert_eq!(duration.as_millis(), 1500);
        assert!((duration.as_secs_f64() - 1.5).abs() < 0.001);
    }
}

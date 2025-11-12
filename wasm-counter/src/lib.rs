use wasm_bindgen::prelude::*;

/// Counter struct to manage counter state and logic in Rust/WASM
#[wasm_bindgen]
pub struct Counter {
    value: i32,
}

#[wasm_bindgen]
impl Counter {
    /// Create a new counter with initial value of 0
    #[wasm_bindgen(constructor)]
    pub fn new() -> Counter {
        Counter { value: 0 }
    }

    /// Create a counter with a specific initial value
    #[wasm_bindgen]
    pub fn with_value(initial: i32) -> Counter {
        Counter { value: initial }
    }

    /// Increment the counter by 1
    #[wasm_bindgen]
    pub fn increment(&mut self) -> i32 {
        self.value = self.value.saturating_add(1);
        self.value
    }

    /// Decrement the counter by 1
    #[wasm_bindgen]
    pub fn decrement(&mut self) -> i32 {
        self.value = self.value.saturating_sub(1);
        self.value
    }

    /// Get the current value
    #[wasm_bindgen]
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// Reset the counter to 0
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.value = 0;
    }

    /// Set the counter to a specific value
    #[wasm_bindgen]
    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_counter() {
        let counter = Counter::new();
        assert_eq!(counter.get_value(), 0);
    }

    #[test]
    fn test_increment() {
        let mut counter = Counter::new();
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
    }

    #[test]
    fn test_decrement() {
        let mut counter = Counter::new();
        counter.set_value(5);
        assert_eq!(counter.decrement(), 4);
        assert_eq!(counter.decrement(), 3);
    }

    #[test]
    fn test_reset() {
        let mut counter = Counter::with_value(10);
        counter.reset();
        assert_eq!(counter.get_value(), 0);
    }
}

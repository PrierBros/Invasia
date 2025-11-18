use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

use crate::logic::SimulationLogic;

#[wasm_bindgen]
pub struct SimulationHandler {
    logic: SimulationLogic,
}

#[wasm_bindgen]
impl SimulationHandler {
    #[wasm_bindgen(constructor)]
    pub fn new(entity_count: usize) -> Self {
        Self {
            logic: SimulationLogic::new(entity_count),
        }
    }

    #[wasm_bindgen]
    pub fn init(entity_count: usize, tick_rate: u32) -> Self {
        let mut handler = Self::new(entity_count);
        handler.logic.set_tick_rate(tick_rate);
        handler
    }

    #[wasm_bindgen]
    pub fn start(&mut self) {
        self.logic.start();
    }

    #[wasm_bindgen]
    pub fn pause(&mut self) {
        self.logic.pause();
    }

    #[wasm_bindgen]
    pub fn resume(&mut self) {
        self.logic.resume();
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.logic.reset();
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.logic.step();
    }

    #[wasm_bindgen]
    pub fn update(&mut self) {
        self.logic.update();
    }

    #[wasm_bindgen]
    pub fn get_tick(&self) -> u64 {
        self.logic.tick()
    }

    #[wasm_bindgen]
    pub fn is_running(&self) -> bool {
        self.logic.running()
    }

    #[wasm_bindgen]
    pub fn get_entity_count(&self) -> usize {
        self.logic.entity_count()
    }

    #[wasm_bindgen]
    pub fn get_tick_rate(&self) -> u32 {
        self.logic.tick_rate()
    }

    #[wasm_bindgen]
    pub fn set_tick_rate(&mut self, tick_rate: u32) {
        self.logic.set_tick_rate(tick_rate);
    }

    #[wasm_bindgen]
    pub fn set_entity_count(&mut self, entity_count: usize) {
        self.logic.set_entity_count(entity_count);
    }

    #[wasm_bindgen]
    pub fn get_snapshot(&mut self) -> JsValue {
        match self.logic.request_snapshot() {
            Some(snapshot) => serde_wasm_bindgen::to_value(&snapshot).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn get_flat_snapshot(&mut self) -> js_sys::Float32Array {
        match self.logic.request_flat_snapshot() {
            Some(slice) => js_sys::Float32Array::from(slice),
            None => js_sys::Float32Array::new_with_length(0),
        }
    }

    #[wasm_bindgen]
    pub fn get_last_tick_duration(&self) -> f64 {
        self.logic.last_tick_duration()
    }

    #[wasm_bindgen]
    pub fn get_last_snapshot_duration(&self) -> f64 {
        self.logic.last_snapshot_duration()
    }

    #[wasm_bindgen]
    pub fn destroy(&mut self) {
        self.logic.destroy();
    }

    #[wasm_bindgen]
    pub fn is_complete(&self) -> bool {
        self.logic.is_complete()
    }

    #[wasm_bindgen]
    pub fn count_alive(&self) -> usize {
        self.logic.count_alive()
    }
}

#[cfg(test)]
impl SimulationHandler {
    pub fn logic(&self) -> &SimulationLogic {
        &self.logic
    }

    pub fn logic_mut(&mut self) -> &mut SimulationLogic {
        &mut self.logic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_simulation_with_entities() {
        let handler = SimulationHandler::new(10);
        assert_eq!(handler.get_entity_count(), 10);
        assert_eq!(handler.get_tick(), 0);
        assert!(!handler.is_running());
    }

    #[test]
    fn steps_and_updates_tick() {
        let mut handler = SimulationHandler::new(5);
        handler.step();
        assert_eq!(handler.get_tick(), 1);
    }

    #[test]
    fn toggles_running_state() {
        let mut handler = SimulationHandler::new(3);
        handler.start();
        assert!(handler.is_running());
        handler.pause();
        assert!(!handler.is_running());
        handler.resume();
        assert!(handler.is_running());
    }

    #[test]
    fn reset_rebuilds_entities() {
        let mut handler = SimulationHandler::new(4);
        handler.step();
        handler.reset();
        assert_eq!(handler.get_tick(), 0);
        assert_eq!(handler.get_entity_count(), 4);
        assert!(!handler.is_running());
    }

    #[test]
    fn counts_alive_entities() {
        let handler = SimulationHandler::new(5);
        assert_eq!(handler.count_alive(), 5);
    }

    #[test]
    fn detects_completion_when_one_alive() {
        use crate::types::AiState;
        
        let mut handler = SimulationHandler::new(3);
        
        // Initially not complete
        assert!(!handler.is_complete());
        assert_eq!(handler.count_alive(), 3);
        
        // Kill two entities manually for testing by removing their territory
        if let Some(entity) = handler.logic_mut().data_mut().entity_mut(0) {
            entity.state = AiState::Dead;
            entity.territory = 0;
        }
        if let Some(entity) = handler.logic_mut().data_mut().entity_mut(1) {
            entity.state = AiState::Dead;
            entity.territory = 0;
        }
        
        // Should be complete with only one alive
        assert!(handler.is_complete());
        assert_eq!(handler.count_alive(), 1);
    }

    #[test]
    fn simulation_stops_when_complete() {
        use crate::types::AiState;
        
        let mut handler = SimulationHandler::new(2);
        handler.start();
        assert!(handler.is_running());
        
        // Kill one entity by removing its territory
        if let Some(entity) = handler.logic_mut().data_mut().entity_mut(0) {
            entity.state = AiState::Dead;
            entity.territory = 0;
        }
        
        // Step should detect completion and stop
        handler.step();
        assert!(!handler.is_running(), "Simulation should stop when complete");
        assert!(handler.is_complete());
    }

    #[test]
    #[ignore] // This is a long-running test, run with --ignored flag
    fn small_grid_completes_within_time_limit() {
        // Test for a 50x50 grid simulation (2500 square units)
        // Should complete within 10 minutes per 50 unit square grid
        // For a 50x50 grid, that's 10 minutes total
        // 
        // Note: This test uses fewer entities (5) to speed up testing while
        // still validating the completion logic works correctly
        use std::time::{Duration, Instant};
        
        let entity_count = 5; // Small number of entities for faster completion
        let mut handler = SimulationHandler::new(entity_count);
        handler.start();
        
        let start = Instant::now();
        let timeout = Duration::from_secs(600); // 10 minutes
        let max_ticks = 1_000_000; // Safety limit to prevent infinite loops
        
        let mut tick_count = 0;
        while handler.is_running() && tick_count < max_ticks {
            handler.step();
            tick_count += 1;
            
            // Check timeout
            if start.elapsed() > timeout {
                panic!(
                    "Simulation did not complete within 10 minutes. Ticks: {}, Alive: {}",
                    tick_count,
                    handler.count_alive()
                );
            }
            
            // Log progress periodically
            if tick_count % 10000 == 0 {
                println!(
                    "Tick {}: {} alive, elapsed: {:?}",
                    tick_count,
                    handler.count_alive(),
                    start.elapsed()
                );
            }
        }
        
        // Verify simulation completed (stopped because only one AI left)
        assert!(
            handler.is_complete(),
            "Simulation should be complete. Alive: {}",
            handler.count_alive()
        );
        assert!(
            handler.count_alive() <= 1,
            "Should have at most one AI alive, got: {}",
            handler.count_alive()
        );
        
        let elapsed = start.elapsed();
        println!(
            "✓ Simulation completed in {:?} with {} ticks",
            elapsed, tick_count
        );
        assert!(
            elapsed < timeout,
            "Simulation took too long: {:?}",
            elapsed
        );
    }
}

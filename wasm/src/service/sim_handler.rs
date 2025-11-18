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
}

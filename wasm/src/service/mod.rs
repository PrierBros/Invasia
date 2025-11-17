use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

use crate::data::SimulationData;
use crate::dependency::performance_now;
use crate::logic::SimulationLogic;

#[wasm_bindgen]
pub struct Simulation {
    data: SimulationData,
    logic: SimulationLogic,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new(entity_count: usize) -> Self {
        Self {
            data: SimulationData::new(entity_count),
            logic: SimulationLogic::default(),
        }
    }

    #[wasm_bindgen]
    pub fn init(entity_count: usize, tick_rate: u32) -> Self {
        let mut sim = Self::new(entity_count);
        sim.data.tick_rate = tick_rate;
        sim
    }

    #[wasm_bindgen]
    pub fn start(&mut self) {
        self.data.running = true;
    }

    #[wasm_bindgen]
    pub fn pause(&mut self) {
        self.data.running = false;
    }

    #[wasm_bindgen]
    pub fn resume(&mut self) {
        self.data.running = true;
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.data.reset_entities();
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.logic.step(&mut self.data);
    }

    #[wasm_bindgen]
    pub fn update(&mut self) {
        if self.data.running {
            self.step();
        }
    }

    #[wasm_bindgen]
    pub fn get_tick(&self) -> u64 {
        self.data.tick
    }

    #[wasm_bindgen]
    pub fn is_running(&self) -> bool {
        self.data.running
    }

    #[wasm_bindgen]
    pub fn get_entity_count(&self) -> usize {
        self.data.entity_len()
    }

    #[wasm_bindgen]
    pub fn get_tick_rate(&self) -> u32 {
        self.data.tick_rate
    }

    #[wasm_bindgen]
    pub fn set_tick_rate(&mut self, tick_rate: u32) {
        self.data.tick_rate = tick_rate;
    }

    #[wasm_bindgen]
    pub fn set_entity_count(&mut self, entity_count: usize) {
        self.data.entity_count = entity_count;
        self.data.reset_entities();
    }

    #[wasm_bindgen]
    pub fn get_snapshot(&mut self) -> JsValue {
        if !self.data.snapshot_dirty {
            return JsValue::NULL;
        }

        let start = performance_now();
        let result = serde_wasm_bindgen::to_value(self.data.entities()).unwrap_or(JsValue::NULL);
        let end = performance_now();
        if start > 0.0 && end >= start {
            self.data.last_snapshot_duration_ms = end - start;
        }

        self.data.snapshot_dirty = false;
        result
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn get_flat_snapshot(&mut self) -> js_sys::Float32Array {
        let start = performance_now();
        self.data.ensure_flat_snapshot_ready();
        let end = performance_now();
        if start > 0.0 && end >= start {
            self.data.last_snapshot_duration_ms = end - start;
        }
        js_sys::Float32Array::from(self.data.flat_snapshot_slice())
    }

    #[wasm_bindgen]
    pub fn get_last_tick_duration(&self) -> f64 {
        self.data.last_tick_duration_ms
    }

    #[wasm_bindgen]
    pub fn get_last_snapshot_duration(&self) -> f64 {
        self.data.last_snapshot_duration_ms
    }

    #[wasm_bindgen]
    pub fn destroy(&mut self) {
        self.data.destroy();
    }
}

#[cfg(test)]
impl Simulation {
    pub(crate) fn data(&self) -> &SimulationData {
        &self.data
    }

    pub(crate) fn data_mut(&mut self) -> &mut SimulationData {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AiEntity, AiState};

    #[test]
    fn test_ai_entity_creation() {
        let entity = AiEntity::new(0);
        assert_eq!(entity.id, 0);
        assert!(entity.health >= 70.0 && entity.health <= 100.0);
        assert!(entity.military_strength >= 50.0 && entity.military_strength <= 100.0);
    }

    #[test]
    fn test_simulation_creation() {
        let sim = Simulation::new(10);
        assert_eq!(sim.get_entity_count(), 10);
        assert_eq!(sim.get_tick(), 0);
        assert!(!sim.is_running());
    }

    #[test]
    fn test_simulation_step() {
        let mut sim = Simulation::new(10);
        let initial_tick = sim.get_tick();
        sim.step();
        assert_eq!(sim.get_tick(), initial_tick + 1);
    }

    #[test]
    fn test_start_pause_resume() {
        let mut sim = Simulation::new(5);
        sim.start();
        assert!(sim.is_running());
        sim.pause();
        assert!(!sim.is_running());
        sim.resume();
        assert!(sim.is_running());
    }

    #[test]
    fn test_reset_rebuilds_entities() {
        let mut sim = Simulation::new(10);
        sim.step();
        assert!(sim.get_tick() > 0);
        sim.reset();
        assert_eq!(sim.get_tick(), 0);
        assert_eq!(sim.get_entity_count(), 10);
    }

    #[test]
    fn test_entity_death_transfers_resources() {
        let mut sim = Simulation::new(3);
        {
            let data = sim.data_mut();
            data.entities[0].health = 0.0;
            data.entities[0].military_strength = 100.0;
            data.entities[0].money = 200.0;
            data.entities[0].position_x = 0.0;
            data.entities[0].position_y = 0.0;

            data.entities[1].state = AiState::Active;
            data.entities[1].position_x = 3.0;
            data.entities[1].position_y = 0.0;

            data.entities[2].state = AiState::Active;
            data.entities[2].position_x = 50.0;
            data.entities[2].position_y = 50.0;
        }

        sim.step();

        let data = sim.data();
        assert_eq!(data.entities[0].state, AiState::Dead);
        assert!(data.entities[1].military_strength > 100.0);
        assert!(data.entities[1].money > 200.0);
    }

    #[test]
    fn test_tick_rate_configuration() {
        let mut sim = Simulation::new(10);
        assert_eq!(sim.get_tick_rate(), 60);
        sim.set_tick_rate(30);
        assert_eq!(sim.get_tick_rate(), 30);
    }
}

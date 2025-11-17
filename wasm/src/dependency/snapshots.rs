use crate::data::{AiEntity, EntitySnapshot};

pub struct SnapshotBuffer {
    inner: Vec<EntitySnapshot>,
}

impl SnapshotBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    pub fn rebuild(&mut self, entities: &[AiEntity]) {
        self.inner.clear();
        for entity in entities {
            self.inner.push(EntitySnapshot::from(entity));
        }
    }

    pub fn as_slice(&self) -> &[EntitySnapshot] {
        &self.inner
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

pub struct FlatSnapshotCache {
    data: Vec<f32>,
}

impl FlatSnapshotCache {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn resize(&mut self, new_len: usize) {
        if self.data.len() != new_len {
            self.data.resize(new_len, 0.0);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

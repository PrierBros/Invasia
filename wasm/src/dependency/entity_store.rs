use crate::data::AiEntity;

pub struct EntityStore {
    entities: Vec<AiEntity>,
}

impl EntityStore {
    pub fn new(entity_count: usize) -> Self {
        let mut entities = Vec::with_capacity(entity_count);
        for i in 0..entity_count {
            entities.push(AiEntity::new(i as u32));
        }
        Self { entities }
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn as_slice(&self) -> &[AiEntity] {
        &self.entities
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut AiEntity> {
        self.entities.get_mut(index)
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn rebuild(&mut self, entity_count: usize) {
        self.entities.clear();
        for i in 0..entity_count {
            self.entities.push(AiEntity::new(i as u32));
        }
    }
}

impl std::ops::Index<usize> for EntityStore {
    type Output = AiEntity;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entities[index]
    }
}

impl std::ops::IndexMut<usize> for EntityStore {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entities[index]
    }
}

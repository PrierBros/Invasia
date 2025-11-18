use super::SimulationData;

pub struct AiNeighborBuilder;

impl AiNeighborBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn rebuild_snapshots(&self, data: &mut SimulationData) {
        data.rebuild_snapshot_buffer();
    }
}

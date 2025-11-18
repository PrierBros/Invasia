use crate::types::{AiState, EntitySnapshot};

const GRID_SIZE: usize = 500;
const MAX_ENTITIES_PER_CELL: usize = 4;

pub struct GridUpdateBuilder {
    grid: SpatialGrid,
}

impl GridUpdateBuilder {
    pub fn new(cell_size: f32, search_radius: f32) -> Self {
        Self {
            grid: SpatialGrid::new(cell_size, search_radius),
        }
    }

    pub fn rebuild(&mut self, snapshots: &[EntitySnapshot]) {
        self.grid.rebuild(snapshots);
    }

    pub fn for_each_neighbor<F>(&self, x: f32, y: f32, f: F)
    where
        F: FnMut(usize),
    {
        self.grid.for_each_neighbor(x, y, f);
    }
}

struct SpatialGrid {
    cell_size: f32,
    _search_radius: f32,
    cells: Vec<([usize; MAX_ENTITIES_PER_CELL], usize)>,
    grid_min: (i32, i32),
    grid_max: (i32, i32),
    overflow_count: usize,
    neighbor_offsets: Vec<(i32, i32)>,
}

impl SpatialGrid {
    fn new(cell_size: f32, search_radius: f32) -> Self {
        let capacity = GRID_SIZE * GRID_SIZE;
        let mut cells = Vec::with_capacity(capacity);
        cells.resize(capacity, ([0; MAX_ENTITIES_PER_CELL], 0));

        let range = (search_radius / cell_size).ceil() as i32;
        let mut neighbor_offsets = Vec::with_capacity(((range * 2) + 1).pow(2) as usize);
        for dx in -range..=range {
            for dy in -range..=range {
                neighbor_offsets.push((dx, dy));
            }
        }

        Self {
            cell_size,
            _search_radius: search_radius,
            cells,
            grid_min: (-(GRID_SIZE as i32 / 2), -(GRID_SIZE as i32 / 2)),
            grid_max: (GRID_SIZE as i32 / 2, GRID_SIZE as i32 / 2),
            overflow_count: 0,
            neighbor_offsets,
        }
    }

    fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.1 = 0;
        }
        self.overflow_count = 0;
    }

    fn cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        let cx = (x / self.cell_size).floor() as i32;
        let cy = (y / self.cell_size).floor() as i32;
        (cx, cy)
    }

    fn cell_index(&self, cx: i32, cy: i32) -> Option<usize> {
        if cx < self.grid_min.0
            || cx >= self.grid_max.0
            || cy < self.grid_min.1
            || cy >= self.grid_max.1
        {
            return None;
        }
        let x = (cx - self.grid_min.0) as usize;
        let y = (cy - self.grid_min.1) as usize;
        Some(y * GRID_SIZE + x)
    }

    fn rebuild(&mut self, snapshots: &[EntitySnapshot]) {
        self.clear();
        for (index, entity) in snapshots.iter().enumerate() {
            // Only track Attacking and Defending entities in the spatial grid
            if entity.state != AiState::Attacking && entity.state != AiState::Defending {
                continue;
            }

            let coords = self.cell_coords(entity.position_x, entity.position_y);
            if let Some(cell_idx) = self.cell_index(coords.0, coords.1) {
                let cell = &mut self.cells[cell_idx];
                if cell.1 < MAX_ENTITIES_PER_CELL {
                    cell.0[cell.1] = index;
                    cell.1 += 1;
                } else {
                    self.overflow_count += 1;
                    #[cfg(debug_assertions)]
                    {
                        eprintln!(
                            "Warning: Spatial grid cell at ({}, {}) is full (max {} Attacking/Defending entities). Entity {} at ({:.2}, {:.2}) dropped. Total overflow: {}",
                            coords.0,
                            coords.1,
                            MAX_ENTITIES_PER_CELL,
                            index,
                            entity.position_x,
                            entity.position_y,
                            self.overflow_count
                        );
                    }
                }
            }
        }

        #[cfg(debug_assertions)]
        {
            if self.overflow_count > 0 {
                eprintln!(
                    "Spatial grid rebuild complete. {} Attacking/Defending entities couldn't be added due to cell capacity limits.",
                    self.overflow_count
                );
            }
        }
    }

    fn for_each_neighbor<F>(&self, x: f32, y: f32, mut f: F)
    where
        F: FnMut(usize),
    {
        let (cx, cy) = self.cell_coords(x, y);
        for &(dx, dy) in &self.neighbor_offsets {
            if let Some(cell_idx) = self.cell_index(cx + dx, cy + dy) {
                let cell = &self.cells[cell_idx];
                for &entity_idx in &cell.0[..cell.1] {
                    f(entity_idx);
                }
            }
        }
    }
}

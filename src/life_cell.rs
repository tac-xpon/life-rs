#[derive(Clone, Copy, PartialEq)]
pub enum CellState {
    Dead,
    Live,
}
impl Default for CellState {
    fn default() -> Self {
        Self::Dead
    }
}
impl CellState {
    pub fn invert(self) -> Self {
        if self == Self::Dead { Self::Live } else { Self::Dead }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Cell {
    pub live: CellState,
    pub neighbours: i32,
}

pub struct World {
    size: (usize, usize),
    grid: Vec<Cell>,
    changes: Vec<usize>,
}
impl World {
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            size,
            grid: vec![Cell::default(); size.0 * size.1],
            changes: Vec::with_capacity(size.0 * size.1 / 8),
        }
    }

    pub fn read_cell(&self, pos: (usize, usize)) -> Cell {
        let linear_pos = (pos.0 % self.size.0) + (pos.1 % self.size.1) * self.size.0;
        self.grid[linear_pos]
    }

    fn change_cell(&mut self, linear_pos: usize) -> i32 {
        let linear_size = self.size.0 * self.size.1;
        self.grid[linear_pos].live = self.grid[linear_pos].live.invert();
        let d = if self.grid[linear_pos].live == CellState::Live { 1 } else { -1 };
        let temp_pos_b = linear_pos + linear_size;
        let temp_pos_a = temp_pos_b - self.size.0;
        let temp_pos_c = temp_pos_b + self.size.0;
        self.grid[(temp_pos_a - 1) % linear_size].neighbours += d;
        self.grid[(temp_pos_a    ) % linear_size].neighbours += d;
        self.grid[(temp_pos_a + 1) % linear_size].neighbours += d;
        self.grid[(temp_pos_b - 1) % linear_size].neighbours += d;
        self.grid[(temp_pos_b + 1) % linear_size].neighbours += d;
        self.grid[(temp_pos_c - 1) % linear_size].neighbours += d;
        self.grid[(temp_pos_c    ) % linear_size].neighbours += d;
        self.grid[(temp_pos_c + 1) % linear_size].neighbours += d;
        d
    }

    pub fn set_cell(&mut self, pos: (usize, usize), state: CellState) -> i32 {
        let linear_pos = (pos.0 % self.size.0) + (pos.1 % self.size.1) * self.size.0;
        if self.grid[linear_pos].live != state {
            self.change_cell(linear_pos)
        } else {
            0
        }
    }

    pub fn update_world(&mut self) -> i32 {
        let mut growth = 0;
        for (linear_pos, cell) in self.grid.iter_mut().enumerate() {
            if cell.live == CellState::Live {
                match cell.neighbours {
                    2 | 3 => {},
                    _ => self.changes.push(linear_pos),
                }
            } else {
                if cell.neighbours == 3 {
                    self.changes.push(linear_pos);
                }
            }
        }
        while let Some(linear_pos) = self.changes.pop() {
            growth += self.change_cell(linear_pos);
        }
        growth
    }
}

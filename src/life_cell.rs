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
    pub state: CellState,
    pub neighbours: i32,
}

pub struct World {
    size: (usize, usize),
    linear_size: usize,
    grid: Vec<Cell>,
    changes: Vec<usize>,
}
impl World {
    pub fn new(size: (usize, usize)) -> Self {
        let linear_size = size.0 * size.1;
        Self {
            size,
            linear_size,
            grid: vec![Cell::default(); linear_size],
            changes: Vec::with_capacity(linear_size / 8),
        }
    }

    pub fn read_cell(&self, pos: (usize, usize)) -> Cell {
        let linear_pos = (pos.0 % self.size.0) + (pos.1 % self.size.1) * self.size.0;
        self.grid[linear_pos]
    }

    fn change_cell(&mut self, linear_pos: usize) -> i32 {
        self.grid[linear_pos].state = self.grid[linear_pos].state.invert();
        let d = if self.grid[linear_pos].state == CellState::Live { 1 } else { -1 };
        let temp_pos_b = linear_pos + self.linear_size;
        let temp_pos_a = temp_pos_b - self.size.0;
        let temp_pos_c = temp_pos_b + self.size.0;
        self.grid[(temp_pos_a - 1) % self.linear_size].neighbours += d;
        self.grid[(temp_pos_a    ) % self.linear_size].neighbours += d;
        self.grid[(temp_pos_a + 1) % self.linear_size].neighbours += d;
        self.grid[(temp_pos_b - 1) % self.linear_size].neighbours += d;
        self.grid[(temp_pos_b + 1) % self.linear_size].neighbours += d;
        self.grid[(temp_pos_c - 1) % self.linear_size].neighbours += d;
        self.grid[(temp_pos_c    ) % self.linear_size].neighbours += d;
        self.grid[(temp_pos_c + 1) % self.linear_size].neighbours += d;
        d
    }

    pub fn set_cell(&mut self, pos: (usize, usize), state: CellState) -> i32 {
        let linear_pos = (pos.0 % self.size.0) + (pos.1 % self.size.1) * self.size.0;
        if self.grid[linear_pos].state != state {
            self.change_cell(linear_pos)
        } else {
            0
        }
    }

    pub fn update_world(&mut self) -> i32 {
        let mut growth = 0;
        for (linear_pos, cell) in self.grid.iter_mut().enumerate() {
            if cell.state == CellState::Live {
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

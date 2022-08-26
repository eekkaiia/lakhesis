use macroquad::prelude::Color;
use macroquad::prelude::BLANK;
use macroquad::rand::gen_range;
use macroquad::rand::srand;
use macroquad::texture::Image;

// number of grains to initiate a 'collapse' of the sandpile
const CRITICAL: u8 = 4;
// default width of table
pub const MODEL_WIDTH: usize = 2_700; // 2_700 x 2_700 grid should contain a single 17M-grain sandpile
                                      // default height of table
pub const MODEL_HEIGHT: usize = 2_700; // this would = 7_290_000 cells
                                       // number of iterations before simulation resets
pub const MAX_ITERATIONS: usize = 16_777_216;
// maximum number of drop cells
pub const MAX_DROPS: usize = 32;

#[derive(Clone, Copy, Debug, Default)]
pub struct Hues {
    pub untouched: Color,
    pub zero_grains: Color,
    pub one_grain: Color,
    pub two_grains: Color,
    pub three_grains: Color,
    pub four_grains: Color, // not needed when collapse occurs at four grains
                            // left in for variation where collapse occurs at five grains
}

impl Hues {
    fn default() -> Self {
        let untouched: Color = Color::new(0.00, 0.00, 0.00, 0.00);
        let zero_grains: Color = Color::new(0.00, 0.47, 0.95, 1.00);
        let one_grain: Color = Color::new(0.00, 0.89, 0.19, 1.00);
        let two_grains: Color = Color::new(0.99, 0.98, 0.00, 1.00);
        let three_grains: Color = Color::new(0.00, 0.00, 0.00, 0.00);
        let four_grains: Color = Color::new(0.90, 0.16, 0.22, 1.00);
        Self {
            untouched,    // BLANK
            zero_grains,  // BLUE
            one_grain,    // GREEN
            two_grains,   // YELLOW
            three_grains, // BLANK stable three grain piles make up the large triangular areas
            // set to blank to highlight 'threads' of 0, 1, and 2 grain cells
            four_grains, // RED for version where collapse occurs at five grains
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Cell {
    pub grains: u8, // number of sand grains in cell
    pub collapses: u8, // number of times cell has collapsed
                    // > 255 recorded as 255
                    // grains == 0 && collapses == 0 is an untouched cell
}

impl Cell {
    fn default() -> Self {
        Self {
            grains: 0,
            collapses: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub cells: Vec<Cell>,    // 1D vec of cells indexed by total width * y + x
    pub width: usize,        // width of 'table' that sand falls on
    pub height: usize, // height of 'table' that sand falls on - not using "length" 2b compatible with screen terminology
    pub total_grains: usize, // current quantity of sand grains that have fallen on 'table'
    pub lost_grains: usize, // current quantity of sand grains that have fallen off 'table'
    pub drop_cells: [usize; MAX_DROPS],
    pub active_cells: usize,
    pub hues: Hues,
    pub interval: usize,
    pub avalanche: usize, // for future implementation
}

impl Model {
    pub fn default() -> Self {
        let size = MODEL_WIDTH
            .checked_mul(MODEL_HEIGHT)
            .expect("Table too big");
        let drop_cells: [usize; MAX_DROPS] = [0; MAX_DROPS];
        Self {
            cells: vec![Cell::default(); size],
            width: MODEL_WIDTH,
            height: MODEL_HEIGHT,
            total_grains: 0,
            lost_grains: 0,
            drop_cells,
            active_cells: 0,
            hues: Hues::default(),
            interval: 1_024,
            avalanche: 0,
        }
    }
    /// calc_center_idx() returns the index of the center cell
    pub fn calc_center_idx(&self) -> usize {
        let size = self.width.checked_mul(self.height).expect("Table too big");
        let center_idx: usize = {
            if self.height % 2 == 0 {
                if self.width % 2 == 0 {
                    (size / 2) + (self.width / 2)
                } else {
                    (size / 2) + ((self.width - 1) / 2)
                }
            } else {
                (size - 1) / 2
            }
        };
        center_idx
    }
    /// calc_center_xy() returns the (x, y) coordinates of the center cell
    pub fn calc_center_xy(&self) -> (usize, usize) {
        let temp: usize = self.calc_center_idx();
        self.idx_to_xy(temp)
    }
    /// xy_to_idx() converts cell (x, y) coordinates into vector index
    pub fn xy_to_idx(&self, x: usize, y: usize) -> usize {
        (y * self.width) + x
    }
    /// idx_to_xy() converts cell vector index into (x, y) coordinates
    pub fn idx_to_xy(&self, idx: usize) -> (usize, usize) {
        let y = (idx as f64 / self.width as f64).trunc() as usize;
        let x = idx % self.width;
        (x, y)
    }
    /// add_grain() drops one grain of sand on the designated cell and checks if it collapsed
    pub fn add_grain(&mut self, ac: usize) {
        self.total_grains += 1;
        self.avalanche = 0;
        self.cells[self.drop_cells[ac]].grains += 1;
        if self.cells[self.drop_cells[ac]].grains >= CRITICAL {
            self.unstable(self.drop_cells[ac]);
        }
    }
    // a previous version of unstable() recursively called itself and evaluated adj cells internally
    // this resulted in a stack overflow around 5M sand grains
    // a second version added a fn for each direction that then called unstable()
    // the second version resulted in a stack overflow around 10M sand grains
    // this version evaluates the second iteration in each adjacent cell and recursively calls the adjacent functions
    // this third version seems to overflow at ~20M grains - convert to iterative function?
    /// unstable() evaluates cell collapse for drop cell and calls a similar fn for each adjacent cell
    fn unstable(&mut self, idx: usize) {
        self.avalanche += 1;
        self.cells[idx].grains -= CRITICAL;
        if self.cells[idx].collapses < 255 {
            self.cells[idx].collapses += 1;
        };
        self.minusy(idx);
        self.plusy(idx);
        self.minusx(idx);
        self.plusx(idx);
    }
    /// minusy() evaluates the cell above
    fn minusy(&mut self, idx: usize) {
        if idx as i32 - (self.width as i32) < 0 {
            self.lost_grains += 1;
        } else {
            let nidx = idx - self.width;
            self.cells[nidx].grains += 1;
            if self.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.cells[nidx].grains -= CRITICAL;
                if self.cells[nidx].collapses < 255 {
                    self.cells[nidx].collapses += 1;
                };
                self.minusy(nidx);
                self.plusy(nidx);
                self.minusx(nidx);
                self.plusx(nidx);
            }
        }
    }
    /// plusy() evaluates the cell below
    fn plusy(&mut self, idx: usize) {
        if idx + self.width >= self.width * self.height {
            self.lost_grains += 1;
        } else {
            let nidx = idx + self.width;
            self.cells[nidx].grains += 1;
            if self.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.cells[nidx].grains -= CRITICAL;
                if self.cells[nidx].collapses < 255 {
                    self.cells[nidx].collapses += 1;
                };
                self.minusy(nidx);
                self.plusy(nidx);
                self.minusx(nidx);
                self.plusx(nidx);
            }
        }
    }
    /// minusx() evaluates the cell to the left
    fn minusx(&mut self, idx: usize) {
        if idx % self.width == 0 {
            self.lost_grains += 1;
        } else {
            let nidx = idx - 1;
            self.cells[nidx].grains += 1;
            if self.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.cells[nidx].grains -= CRITICAL;
                if self.cells[nidx].collapses < 255 {
                    self.cells[nidx].collapses += 1;
                };
                self.minusy(nidx);
                self.plusy(nidx);
                self.minusx(nidx);
                self.plusx(nidx);
            }
        }
    }
    // plusx() evaluates the cell to the right
    fn plusx(&mut self, idx: usize) {
        if (idx + 1) % self.width == 0 {
            self.lost_grains += 1;
        } else {
            let nidx = idx + 1;
            self.cells[nidx].grains += 1;
            if self.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.cells[nidx].grains -= CRITICAL;
                if self.cells[nidx].collapses < 255 {
                    self.cells[nidx].collapses += 1;
                };
                self.minusy(nidx);
                self.plusy(nidx);
                self.minusx(nidx);
                self.plusx(nidx);
            }
        }
    }
    /// random_colors() generates random RGBA values for 'Hues' using macroquads quad_rand crate
    pub fn random_colors(&mut self) {
        srand(self.total_grains as u64);
        self.hues.zero_grains = Color::new(
            gen_range::<f32>(0.0, 1.0),
            gen_range::<f32>(0.0, 1.0),
            gen_range::<f32>(0.0, 1.0),
            1.0,
        );
        self.hues.one_grain = Color::new(
            gen_range::<f32>(0.0, 1.0),
            gen_range::<f32>(0.0, 1.0),
            gen_range::<f32>(0.0, 1.0),
            1.0,
        );
        self.hues.two_grains = Color::new(
            gen_range::<f32>(0.0, 1.0),
            gen_range::<f32>(0.0, 1.0),
            gen_range::<f32>(0.0, 1.0),
            1.0,
        );
    }
    /// find_extent() returns the minimum x, minimum y, width, and height of the active area of the model
    // returned tuple matches arguments for paint()
    pub fn find_extent(&self) -> (u32, u32, u16, u16) {
        let (mut min_x, mut min_y) = self.calc_center_xy();
        let (mut max_x, mut max_y) = self.calc_center_xy();
        for i in 0..self.cells.len() {
            if self.cells[i].grains != 0 && self.cells[i].collapses != 0 {
                let (this_x, this_y) = self.idx_to_xy(i);
                if this_x < min_x {
                    min_x = this_x;
                };
                if this_x > max_x {
                    max_x = this_x;
                };
                if this_y < min_y {
                    min_y = this_y;
                };
                if this_y > max_y {
                    max_y = this_y;
                };
            }
        }
        if min_x >= 10 {
            min_x -= 10
        } else {
            min_x = 0
        };
        if max_x <= MODEL_WIDTH - 10 {
            max_x += 10;
        } else {
            max_x = MODEL_WIDTH;
        };
        if min_y >= 10 {
            min_y -= 10;
        } else {
            min_y = 0;
        };
        if max_y <= MODEL_HEIGHT - 10 {
            max_y += 10;
        } else {
            max_y = MODEL_HEIGHT;
        };
        (
            min_x.try_into().expect("Too big"),
            min_y.try_into().expect("Too big"),
            (max_x - min_x).try_into().expect("Too big"),
            (max_y - min_y).try_into().expect("Too big"),
        )
    }
    /// paint() exports a PNG image of the current model using the macroquad export_png() function
    pub fn paint(&self, tlx: u32, tly: u32, x_width: u16, y_height: u16) {
        let mut sand_painting = Image::gen_image_color(x_width, y_height, BLANK);
        for row in 0..y_height as usize - 1 {
            for column in 0..x_width as usize - 1 {
                let idx = self.xy_to_idx(column + tlx as usize, row + tly as usize);
                match self.cells[idx].grains {
                    0 => {
                        if self.cells[idx].grains == 0 && self.cells[idx].collapses == 0 {
                            sand_painting.set_pixel(column as u32, row as u32, self.hues.untouched);
                        } else {
                            sand_painting.set_pixel(
                                column as u32,
                                row as u32,
                                self.hues.zero_grains,
                            );
                        }
                    }
                    1 => sand_painting.set_pixel(column as u32, row as u32, self.hues.one_grain),
                    2 => sand_painting.set_pixel(column as u32, row as u32, self.hues.two_grains),
                    3 => sand_painting.set_pixel(column as u32, row as u32, self.hues.three_grains),
                    _ => sand_painting.set_pixel(column as u32, row as u32, self.hues.four_grains),
                }
            }
        }
        // format name & export as PNG
        let fname = format!("Lakhesis_{:08}.png", &self.total_grains);
        sand_painting.export_png(&fname);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        let model = Model::default();
        let idx: usize = 256_001;
        let x: usize = 1;
        let y: usize = 200;
        assert_eq!(model.xy_to_idx(x, y), 256_001);
        assert_eq!(model.idx_to_xy(idx), (1, 200));
        assert_eq!(model.calc_center_idx(), 461_440);
        assert_eq!(model.calc_center_xy(), (640, 360));
    }
}

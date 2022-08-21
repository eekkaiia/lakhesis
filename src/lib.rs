use macroquad::prelude::Color;
use macroquad::prelude::BLANK;
use macroquad::rand::gen_range;
use macroquad::rand::srand;
use macroquad::texture::Image;

// number of grains to initiate a 'collapse' of the sandpile
const CRITICAL: u8 = 4;
// default width of table
pub const MODEL_WIDTH: usize = 1280;
// default height of table
pub const MODEL_HEIGHT: usize = 720;
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
        let one_grain: Color = Color::new(0.99, 0.98, 0.00, 1.00);
        let two_grains: Color = Color::new(0.00, 0.89, 0.19, 1.00);
        let three_grains: Color = Color::new(0.00, 0.00, 0.00, 0.00);
        let four_grains: Color = Color::new(0.90, 0.16, 0.22, 1.00);
        Self {
            untouched,    // BLANK
            zero_grains,  // BLUE
            one_grain,    // YELLOW
            two_grains,   // GREEN
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
    pub fn calc_center_xy(&self) -> (usize, usize) {
        let temp: usize = self.calc_center_idx();
        self.idx_to_xy(temp)
    }
    pub fn xy_to_idx(&self, x: usize, y: usize) -> usize {
        (y * self.width) + x
    }
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
    /// paint() exports a PNG image of the current model using a macroquad function
    pub fn paint(&self) {
        let mut sand_painting = Image::gen_image_color(
            self.width.try_into().expect("width to wide"),
            self.height.try_into().expect("height to high"),
            BLANK,
        );
        for row in 0..self.height - 1 {
            for column in 0..self.width - 1 {
                match self.cells[(row * self.width) + column].grains {
                    0 => {
                        if self.cells[(row * self.width) + column].grains == 0
                            && self.cells[(row * self.width) + column].collapses == 0
                        {
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
        let fname = format!("Lakhesis_{:07}.png", &self.total_grains);
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

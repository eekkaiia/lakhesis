use rand::prelude::*;

// RGBA values for some colors
const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const YELLOW: [u8; 4] = [255, 255, 0, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];
// const BLACK_TRANSPARENT: [u8; 4] = [0, 0, 0, 0];
// const WHITE: [u8; 4] = [255, 255, 255, 255];
const GREY: [u8; 4] = [128, 128, 128, 255];
// const ORANGE: [u8; 4] = [255, 165, 0, 255];
// const PURPLE: [u8; 4] = [191, 64, 191, 255];

// number of grains to initiate a 'collapse' of the sandpile
const CRITICAL: u8 = 4;
// default width of table
const TABLE_WIDTH: usize = 1280;
// default height of table
const TABLE_HEIGHT: usize = 720;
// default buffer at edge of table free of drop_cells
const TABLE_BUFFER: usize = 250;
// number of iterations before simulation resets
pub const ITERATIONS: usize = 2_000_000;
// maximum number of drop cells
pub const MAX_DROPS: usize = 12;

#[derive(Clone, Copy, Debug, Default)]
pub struct Hues {
    pub untouched: [u8; 4],
    pub zero_grains: [u8; 4],
    pub one_grain: [u8; 4],
    pub two_grains: [u8; 4],
    pub three_grains: [u8; 4],
    pub four_grains: [u8; 4],       // not needed when collapse occurs at four grains
                                    // left in for variation where collapse occurs at five grains
}

impl Hues {
    fn default() -> Self {
        Self {
            untouched: BLACK,
            zero_grains: BLUE,
            one_grain: YELLOW,
            two_grains: GREEN,
            three_grains: BLACK,    // stable three grain piles make up the large triangular areas
                                    // set to black to highlight 'threads' of 0, 1, and 2 grain cells
            four_grains: RED,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Cell {
    pub grains: u8,                 // number of sand grains in cell
    pub collapses: u8,              // number of times cell has collapsed
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
pub struct Table {
    pub cells: Vec<Cell>,           // 1D vec of cells indexed by total width * y + x
    pub width: usize,               // width of 'table' that sand falls on
    pub height: usize,              // height of 'table' that sand falls on - not using "length" 2b compatible with screen terminology
    pub total_grains: usize,        // current quantity of sand grains that have fallen on 'table'
    pub lost_grains: usize,         // current quantity of sand grains that have fallen off 'table'
}

impl Table {
    pub fn default() -> Self {
        let table_width: usize = TABLE_WIDTH;
        let table_height: usize = TABLE_HEIGHT;
        let size = table_width.checked_mul(table_height).expect("Table too big");
        Self {
            cells: vec![Cell::default(); size],
            width: table_width,
            height: table_height,
            total_grains: 0,
            lost_grains: 0,
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
}

#[derive(Clone, Debug, Default)]
pub struct Model {
    pub table: Table,
    pub drop_cells: [usize; MAX_DROPS],
    pub drop_times: [usize; MAX_DROPS],
    pub active_cells: usize,
    pub hues: Hues,
    pub interval: usize,
    pub random: bool,
    pub avalanche: usize,       // for future implementation
}

impl Model {
    pub fn default() -> Self {
        // first grain drops at time = 0, the remaining MAX_DROPS possible new locations and times are randomly assigned
        let mut rng = rand::thread_rng();
        let mut drop_cells: [usize; MAX_DROPS]  = [0; MAX_DROPS];
        for i in 0..MAX_DROPS {
            let x: usize = rng.gen_range(TABLE_BUFFER..(TABLE_WIDTH - TABLE_BUFFER));
            let y: usize = rng.gen_range(TABLE_BUFFER..(TABLE_HEIGHT - TABLE_BUFFER));
            drop_cells[i] = (y * TABLE_WIDTH) + x;
        }
        let mut drop_times: [usize; MAX_DROPS]  = [0; MAX_DROPS];
        for i in 1..MAX_DROPS {
            drop_times[i] = rng.gen_range(0..ITERATIONS);
        }
        Self {
            table: Table::default(),
            drop_cells,
            drop_times,
            active_cells: 1,
            hues: Hues::default(),
            interval: 1000,
            random: true,
            avalanche: 0,
        }
    }
    /// add_grain() drops one grain of sand on the designated cell and checks if it collapsed
    pub fn add_grain(&mut self, ac: usize) {
        self.table.total_grains += 1;
        self.avalanche = 0;
        self.table.cells[self.drop_cells[ac]].grains += 1;
        if  self.table.cells[self.drop_cells[ac]].grains >= CRITICAL {
            self.unstable(self.drop_cells[ac]);
        }
    }
    // a previous version of unstable recursively called itself and evaluated adj cells internally
    // this resulted in a stack overflow around 5M sand grains
    // a second version added a fn for each direction that then called unstable
    // the second version resulted in a stack overflow around 10M sand grains
    // this version evaluates the second iteration in each adjacent cell and recursively calls the adjacent functions
    // this third version seems to overflow at ~20M grains - convert to iterative function?
    /// unstable() evaluates cell collapse for drop cell and calls a similar fn for each adjacent cell
    fn unstable (&mut self, idx: usize) {
        self.avalanche += 1;
        self.table.cells[idx].grains -= CRITICAL;
        if self.table.cells[idx].collapses < 255 { self.table.cells[idx].collapses += 1; };
        self.minusy(idx);
        self.plusy(idx);
        self.minusx(idx);
        self.plusx(idx);
    }
    /// minusy() evaluates the cell above
    fn minusy(&mut self, idx: usize) {
        if idx as i32 - (self.table.width as i32) < 0 {
            self.table.lost_grains += 1;
        } else {
            let nidx = idx - self.table.width;
            self.table.cells[nidx].grains += 1;
            if self.table.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.table.cells[nidx].grains -= CRITICAL;
                if self.table.cells[nidx].collapses < 255 { self.table.cells[nidx].collapses += 1; };
                self.minusy(nidx);
                self.plusy(nidx);
                self.minusx(nidx);
                self.plusx(nidx);
            }
        }
    }
    /// plusy() evaluates the cell below
    fn plusy(&mut self, idx: usize) {
        if idx + self.table.width >= self.table.width * self.table.height {
            self.table.lost_grains += 1;
        } else {
            let nidx = idx + self.table.width;
            self.table.cells[nidx].grains += 1;
            if self.table.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.table.cells[nidx].grains -= CRITICAL;
                if self.table.cells[nidx].collapses < 255 { self.table.cells[nidx].collapses += 1; };
                self.minusy(nidx);
                self.plusy(nidx);
                self.minusx(nidx);
                self.plusx(nidx);
            }
        }
    }
    /// minusx() evaluates the cell to the left
    fn minusx (&mut self, idx: usize) {
        if idx % self.table.width == 0 {
            self.table.lost_grains += 1;
        } else {
            let nidx = idx - 1;
            self.table.cells[nidx].grains += 1;
            if self.table.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.table.cells[nidx].grains -= CRITICAL;
                if self.table.cells[nidx].collapses < 255 { self.table.cells[nidx].collapses += 1; };
                self.minusy(nidx);
                self.plusy(nidx);
                self.minusx(nidx);
                self.plusx(nidx);
            }
        }
    }
    // plusx() evaluates the cell to the right
    fn plusx (&mut self, idx: usize) {
        if (idx + 1) % self.table.width == 0 {
            self.table.lost_grains += 1;
        } else {
            let nidx = idx + 1;
            self.table.cells[nidx].grains += 1;
            if self.table.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.table.cells[nidx].grains -= CRITICAL;
                if self.table.cells[nidx].collapses < 255 { self.table.cells[nidx].collapses += 1; };
                self.minusy(nidx);
                self.plusy(nidx);
                self.minusx(nidx);
                self.plusx(nidx);
            }
        }
    }
    /// draw() updates the pixel frame with new RGBA colors
    pub fn draw(&self, frame: &mut [u8]) {  
        debug_assert_eq!(frame.len(), 4 * self.table.cells.len());
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            match self.table.cells[i].grains {
                0 => {
                    if self.table.cells[i].collapses == 0 {
                        if  // edge of table
                            !(TABLE_WIDTH..=(TABLE_WIDTH * TABLE_HEIGHT - TABLE_WIDTH)).contains(&i) ||
                            i % TABLE_WIDTH == 0 ||
                            (i + 1) % TABLE_WIDTH == 0 ||
                            // four corners of buffer
                            i == self.table.xy_to_idx(TABLE_BUFFER, TABLE_BUFFER) ||
                            i == self.table.xy_to_idx(TABLE_WIDTH - TABLE_BUFFER, TABLE_BUFFER) ||
                            i == self.table.xy_to_idx(TABLE_BUFFER, TABLE_HEIGHT - TABLE_BUFFER) ||
                            i == self.table.xy_to_idx(TABLE_WIDTH - TABLE_BUFFER, TABLE_HEIGHT - TABLE_BUFFER)
                        {
                                pixel.copy_from_slice(&GREY);
                        } else { pixel.copy_from_slice(&self.hues.untouched); }
                    } else { pixel.copy_from_slice(&self.hues.zero_grains); }
                },
                1 => pixel.copy_from_slice(&self.hues.one_grain),
                2 => pixel.copy_from_slice(&self.hues.two_grains),
                3 => pixel.copy_from_slice(&self.hues.three_grains),
                _ => pixel.copy_from_slice(&self.hues.four_grains),
            };
        }
    }
    /// paint() exports a PNG image of the current frame
    pub fn paint(&self) {
        let mut grains_img = image::RgbaImage::new(self.table.width as u32, self.table.height as u32);
        for row in 0..self.table.height - 1 {
            for column in 0..self.table.width - 1 {
                match self.table.cells[(row * self.table.width) + column].grains {
                    0 => {
                        if self.table.cells[(row * self.table.width) + column].grains == 0
                            && self.table.cells[(row * self.table.width) + column].collapses == 0 {
                            grains_img.put_pixel(column as u32, row as u32, image::Rgba(self.hues.untouched));
                        } else {
                            grains_img.put_pixel(column as u32, row as u32, image::Rgba(self.hues.zero_grains));
                        }
                    },
                    1 => grains_img.put_pixel(column as u32, row as u32, image::Rgba(self.hues.one_grain)),
                    2 => grains_img.put_pixel(column as u32, row as u32, image::Rgba(self.hues.two_grains)),
                    3 => grains_img.put_pixel(column as u32, row as u32, image::Rgba(self.hues.three_grains)),
                    _ => grains_img.put_pixel(column as u32, row as u32, image::Rgba(self.hues.four_grains)),
                }
            }
        }
        // format name
        let fname = format!("Lakhesis_{:07}", &self.table.total_grains);
        // export PNG image
        let fname_png = format!("{}.png", &fname);
        grains_img.save_with_format(&fname_png, image::ImageFormat::Png).unwrap();
    }
    /// random_colors() generates random RGBA values for 'Hues'
    pub fn random_colors(&mut self) {
        let mut rng = rand::thread_rng();
        self.hues.zero_grains = [
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            255];
        self.hues.one_grain = [
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            255];
        self.hues.two_grains = [
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            255];
    }
}

use macroquad::prelude::Color;
use macroquad::prelude::BLANK;
use macroquad::rand::gen_range;
use macroquad::rand::srand;
use macroquad::texture::Image;

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::io::{LineWriter, Write};

const CRITICAL: u8 = 4; // number of grains to initiate a 'collapse' of the sandpile
const MODEL_WIDTH: usize = 3_000; // 3_000 x 3_000 grid should contain a single 16M-grain sandpile
const MODEL_HEIGHT: usize = 3_000; // this would = 9_000_000 cells
pub const MAX_ITERATIONS: usize = 16_777_216; // number of iterations before simulation pauses - due to stack overflow
pub const MAX_DROPS: usize = 32; // maximum number of drop cells = max array size

/// A Cell is point in the lattice that accumulates sand grains
#[derive(Clone, Copy, Debug, Default)]
pub struct Cell {
    pub grains: u8,   // number of sand grains in cell
    pub borged: bool, // has the cell become part of a sandpile
}
impl Cell {
    fn default() -> Self {
        Self {
            grains: 0,
            borged: false,
        }
    }
}
/// Hues are the colors indicating the different states of a cell in the lattice
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
/// A model represents the lattice on which sandpiles form
#[derive(Clone, Debug, Default)]
pub struct Model {
    pub cells: Vec<Cell>,    // 1D vec of cells indexed by total width * y + x
    pub width: usize,        // width of 'table' that sand falls on
    pub height: usize, // height of 'table' that sand falls on - not using "length" 2b compatible with screen terminology
    pub total_grains: usize, // current quantity of sand grains that have fallen on 'table'
    pub lost_grains: usize, // current quantity of sand grains that have fallen off 'table'
    pub drop_cells: [usize; MAX_DROPS], // idx of each active cell - up to 32
    pub ac: usize,     // current active cell
    pub active_cells: usize, // total number of active cells
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
            ac: 0,
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
    pub fn add_grain(&mut self) {
        self.total_grains += 1;
        self.avalanche = 0;
        self.cells[self.drop_cells[self.ac]].grains += 1;
        self.cells[self.drop_cells[self.ac]].borged = true;
        if self.cells[self.drop_cells[self.ac]].grains >= CRITICAL {
            self.unstable(self.drop_cells[self.ac]);
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
            self.cells[nidx].borged = true;
            if self.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.cells[nidx].grains -= CRITICAL;
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
            self.cells[nidx].borged = true;
            if self.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.cells[nidx].grains -= CRITICAL;
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
            self.cells[nidx].borged = true;
            if self.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.cells[nidx].grains -= CRITICAL;
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
            self.cells[nidx].borged = true;
            if self.cells[nidx].grains == CRITICAL {
                self.avalanche += 1;
                self.cells[nidx].grains -= CRITICAL;
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
        self.hues.untouched = Color::new(0.00, 0.00, 0.00, 0.00);
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
        self.hues.three_grains = Color::new(
            gen_range::<f32>(0.0, 1.0),
            gen_range::<f32>(0.0, 1.0),
            gen_range::<f32>(0.0, 1.0),
            1.0,
        );
        self.hues.four_grains = Color::new(1.00, 0.00, 0.00, 1.00);
    }
    /// find_extent() returns the minimum x, minimum y, width, and height of the active area of the model
    pub fn find_extent(&self) -> (u32, u32, u16, u16) {
        // returned tuple matches arguments for paint()
        let (mut min_x, mut min_y) = self.calc_center_xy();
        let (mut max_x, mut max_y) = self.calc_center_xy();
        for i in 0..self.cells.len() {
            if self.cells[i].borged {
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
                        if !self.cells[idx].borged {
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
    /// curate() saves the model in its current state
    pub fn curate(&self) {
        let filename = format!("lakhesis_model_{:08}.lak", &self.total_grains);
        match File::create(filename) {
            Err(why) => {
                eprintln!("Error creating text file {}", why);
                std::process::exit(1);
            }
            Ok(model_lines) => {
                let mut model_lines = LineWriter::new(model_lines);
                // usize fields
                let mut entry = format!(
                    "lakhesis,alpha,{},{},{},{},{},{},{}\ndrops,",
                    &self.width,
                    &self.height,
                    &self.total_grains,
                    &self.lost_grains,
                    &self.interval,
                    &self.active_cells,
                    &self.avalanche
                );
                model_lines.write_all(entry.as_bytes()).unwrap();
                // array of 32 usize
                for i in 0..MAX_DROPS {
                    if i == MAX_DROPS - 1 {
                        entry = format!("{}\nhues,", &self.drop_cells[i]);
                        model_lines.write_all(entry.as_bytes()).unwrap();
                    } else {
                        entry = format!("{},", &self.drop_cells[i]);
                        model_lines.write_all(entry.as_bytes()).unwrap();
                    }
                }
                // hues are six sets of four f32s
                entry = format!(
                    "{},{},{},{},",
                    &self.hues.untouched.r,
                    &self.hues.untouched.g,
                    &self.hues.untouched.b,
                    &self.hues.untouched.a
                );
                model_lines.write_all(entry.as_bytes()).unwrap();
                entry = format!(
                    "{},{},{},{},",
                    &self.hues.zero_grains.r,
                    &self.hues.zero_grains.g,
                    &self.hues.zero_grains.b,
                    &self.hues.zero_grains.a
                );
                model_lines.write_all(entry.as_bytes()).unwrap();
                entry = format!(
                    "{},{},{},{},",
                    &self.hues.one_grain.r,
                    &self.hues.one_grain.g,
                    &self.hues.one_grain.b,
                    &self.hues.one_grain.a
                );
                model_lines.write_all(entry.as_bytes()).unwrap();
                entry = format!(
                    "{},{},{},{},",
                    &self.hues.two_grains.r,
                    &self.hues.two_grains.g,
                    &self.hues.two_grains.b,
                    &self.hues.two_grains.a
                );
                model_lines.write_all(entry.as_bytes()).unwrap();
                entry = format!(
                    "{},{},{},{},",
                    &self.hues.three_grains.r,
                    &self.hues.three_grains.g,
                    &self.hues.three_grains.b,
                    &self.hues.three_grains.a
                );
                model_lines.write_all(entry.as_bytes()).unwrap();
                entry = format!(
                    "{},{},{},{}\n",
                    &self.hues.four_grains.r,
                    &self.hues.four_grains.g,
                    &self.hues.four_grains.b,
                    &self.hues.four_grains.a
                );
                model_lines.write_all(entry.as_bytes()).unwrap();
                // cells - a bit more complicated
                let mut cursor: usize = 0;
                let mut eof: bool = false;
                let mut subtotal: usize = 0;
                while !eof {
                    // never touched cells
                    if !self.cells[cursor].borged {
                        let mut alt_cursor: usize = cursor;
                        while !self.cells[alt_cursor + 1].borged && !eof {
                            if alt_cursor < self.cells.len() - 2 {
                                alt_cursor += 1;
                            } else {
                                eof = true;
                            };
                        }
                        let mut count: usize = alt_cursor - cursor + 1;
                        if eof {
                            count += 1;
                        };
                        subtotal += count;
                        entry = format!("{},f\n", &count);
                        model_lines.write_all(entry.as_bytes()).unwrap();
                        cursor = alt_cursor + 1;
                    } else {
                        // cells that have contained grains
                        let mut alt_cursor: usize = cursor;
                        while self.cells[alt_cursor + 1].borged && !eof {
                            if alt_cursor < self.cells.len() - 2 {
                                alt_cursor += 1;
                            } else {
                                eof = true;
                            };
                        }
                        let mut count: usize = alt_cursor - cursor + 1;
                        if eof {
                            count += 1;
                        };
                        subtotal += count;
                        entry = format!("{},t,", &count);
                        model_lines.write_all(entry.as_bytes()).unwrap();
                        for i in cursor..cursor + count {
                            entry = format!("{}", &self.cells[i].grains);
                            model_lines.write_all(entry.as_bytes()).unwrap();
                        }
                        model_lines.write_all("\n".to_string().as_bytes()).unwrap();
                        cursor = alt_cursor + 1;
                    }
                }
                entry = format!(
                    "Checksum: {} of {} cells recorded",
                    &subtotal,
                    &self.cells.len()
                );
                model_lines.write_all(entry.as_bytes()).unwrap();
            }
        }
    }
    /// uncurate() loads a model saved using curate() - currently filename must be "lakhesis.lak"
    pub fn uncurate(&mut self, filename: String) {
        let mut temp_cells: Vec<Cell> = Vec::new();
        let source = File::open(&filename).expect("Unable to open file");
        let reader = BufReader::new(source);
        let lines = reader.lines();
        for line in lines {
            match line {
                Err(_) => eprintln!("Error reading line: {:?}", line),
                Ok(line) => {
                    if line.contains("lakhesis") {
                        let pieces: Vec<&str> = line.split(',').collect();
                        self.width = pieces[2].parse::<usize>().unwrap();
                        self.height = pieces[3].parse::<usize>().unwrap();
                        self.total_grains = pieces[4].parse::<usize>().unwrap();
                        self.lost_grains = pieces[5].parse::<usize>().unwrap();
                        self.interval = pieces[6].parse::<usize>().unwrap();
                        self.active_cells = pieces[7].parse::<usize>().unwrap();
                        self.avalanche = pieces[8].parse::<usize>().unwrap();
                    } else if line.contains("drops") {
                        let pieces: Vec<&str> = line.split(',').collect();
                        for i in 0..MAX_DROPS {
                            self.drop_cells[i] = pieces[i + 1].parse::<usize>().unwrap();
                        }
                    } else if line.contains("hues") {
                        let pieces: Vec<&str> = line.split(',').collect();
                        self.hues.untouched.r = pieces[1].parse::<f32>().unwrap();
                        self.hues.untouched.g = pieces[2].parse::<f32>().unwrap();
                        self.hues.untouched.b = pieces[3].parse::<f32>().unwrap();
                        self.hues.untouched.a = pieces[4].parse::<f32>().unwrap();
                        self.hues.zero_grains.r = pieces[5].parse::<f32>().unwrap();
                        self.hues.zero_grains.g = pieces[6].parse::<f32>().unwrap();
                        self.hues.zero_grains.b = pieces[7].parse::<f32>().unwrap();
                        self.hues.zero_grains.a = pieces[8].parse::<f32>().unwrap();
                        self.hues.one_grain.r = pieces[9].parse::<f32>().unwrap();
                        self.hues.one_grain.g = pieces[10].parse::<f32>().unwrap();
                        self.hues.one_grain.b = pieces[11].parse::<f32>().unwrap();
                        self.hues.one_grain.a = pieces[12].parse::<f32>().unwrap();
                        self.hues.two_grains.r = pieces[13].parse::<f32>().unwrap();
                        self.hues.two_grains.g = pieces[14].parse::<f32>().unwrap();
                        self.hues.two_grains.b = pieces[15].parse::<f32>().unwrap();
                        self.hues.two_grains.a = pieces[16].parse::<f32>().unwrap();
                        self.hues.three_grains.r = pieces[17].parse::<f32>().unwrap();
                        self.hues.three_grains.g = pieces[18].parse::<f32>().unwrap();
                        self.hues.three_grains.b = pieces[19].parse::<f32>().unwrap();
                        self.hues.three_grains.a = pieces[20].parse::<f32>().unwrap();
                        self.hues.four_grains.r = pieces[21].parse::<f32>().unwrap();
                        self.hues.four_grains.g = pieces[22].parse::<f32>().unwrap();
                        self.hues.four_grains.b = pieces[23].parse::<f32>().unwrap();
                        self.hues.four_grains.a = pieces[24].parse::<f32>().unwrap();
                    } else if line.contains("Checksum:") {
                        let pieces: Vec<&str> = line.split(' ').collect();
                        if pieces[1] != pieces[3] {
                            eprintln!(
                                "Checksum error in lakhesis.lak: {} != {}",
                                &pieces[1], &pieces[3]
                            );
                        }
                    } else {
                        let pieces: Vec<&str> = line.split(',').collect();
                        if pieces[1] == "f" {
                            for _ in 0..pieces[0].parse::<usize>().unwrap() {
                                let temp: Cell = Cell {
                                    grains: 0,
                                    borged: false,
                                };
                                temp_cells.push(temp);
                            }
                        };
                        if pieces[1] == "t" {
                            let grains: Vec<char> = pieces[2].chars().collect();
                            for grain in grains {
                                let temp: Cell = Cell {
                                    grains: grain
                                        .to_digit(10)
                                        .unwrap()
                                        .try_into()
                                        .expect("Too big"),
                                    borged: true,
                                };
                                temp_cells.push(temp);
                            }
                        }
                    }
                }
            }
        }
        self.cells = temp_cells;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        let mut model = Model::default();
        model.width = 2_700;
        model.height = 2_700;
        let idx: usize = 540_001;
        let x: usize = 1;
        let y: usize = 200;
        assert_eq!(model.xy_to_idx(x, y), 540_001);
        assert_eq!(model.idx_to_xy(idx), (1, 200));
        assert_eq!(model.calc_center_idx(), 3_646_350);
        assert_eq!(model.calc_center_xy(), (1350, 1350));
    }
}

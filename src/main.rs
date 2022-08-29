use lakhesis::{Model, MAX_DROPS, MAX_ITERATIONS, MODEL_HEIGHT, MODEL_WIDTH};

use macroquad::color::colors::*;
use macroquad::color::Color;
use macroquad::input::*;
use macroquad::math::*;
use macroquad::shapes::*;
use macroquad::text::*;
use macroquad::time::*;
use macroquad::window::*;

// maximum interval in sand grains added between screen updates - any higher and simulation becomes unresponsive
pub const MAX_INTERVAL: usize = 16_384;
// number of PNG frames to create 10 second video at 60fps
const VIDEO_FRAME_COUNT: usize = 600;
// set IO_SUPPORTED to false when compiling for WASM - currently saving an image to disk is not supported from web browser
const IO_SUPPORTED: bool = true;

#[derive(Clone, Debug, Default)]
pub struct Screen {
    pub tlx: f32,
    pub tly: f32,
    pub width: f32,
    pub height: f32,
    pub mx: f32,
    pub my: f32,
    pub background: bool,
    pub info: bool,
    pub paused: bool,
    pub magnify: bool,
    pub add: bool,
    pub reset: bool,
    pub increment: bool,
    pub video: usize,
    pub ac: usize,
    pub context: String,
    pub longest_ft: f32,
    pub past_ft: [f32; 32],
    pub ft_idx: usize,
}

impl Screen {
    pub fn default() -> Self {
        let width = screen_width();
        let height = screen_height();
        let tlx: f32 = (MODEL_WIDTH as f32 - width).abs() / 2.0; // initial top left corner of screen
        let tly: f32 = (MODEL_HEIGHT as f32 - height).abs() / 2.0; // centered on model
        let (mx, my) = mouse_position();
        let past_ft = [0.0; 32];
        Self {
            tlx,
            tly,
            width,
            height,
            mx,
            my,
            background: true,
            info: true,
            paused: true,
            magnify: false,
            add: false,
            reset: false,
            increment: false,
            video: 0,
            ac: 0,
            context: "Start a simulation by pressing the [A] key to Add a new sandpile".to_string(),
            longest_ft: 0.0,
            past_ft,
            ft_idx: 0,
        }
    }
    /// get_average_ft() calculates the average frame time over the last 32 frames
    pub fn get_average_ft(&self) -> f32 {
        let mut average: f32 = 0.0;
        for i in 0..32 {
            average += self.past_ft[i];
        }
        average / 32.0
    }
    /// draw() maps visible portion of model to screen
    pub fn draw(&self, model: &Model) {
        // redrawing entire screen reduces fps - use libraries find_extent() to only update active part of model
        let mut render: bool = true;
        let (left, top, left_plus, top_plus) = model.find_extent();
        if (left + left_plus as u32) < self.tlx.trunc() as u32
            || left > (self.tlx + self.width).trunc() as u32
            || (top + top_plus as u32) < self.tly.trunc() as u32
            || top > (self.tly + self.height).trunc() as u32
        {
            render = false;
        }
        if render {
            let mut xstart: usize = 0;
            let mut xstop: usize = self.width.trunc() as usize;
            let mut ystart: usize = 0;
            let mut ystop: usize = self.height.trunc() as usize;
            if left > self.tlx.trunc() as u32 {
                xstart = (left - self.tlx.trunc() as u32) as usize
            };
            if (left + left_plus as u32) < (self.tlx + self.width).trunc() as u32 {
                xstop = ((left + left_plus as u32) - self.tlx.trunc() as u32) as usize
            };
            if top > self.tly.trunc() as u32 {
                ystart = (top - self.tly.trunc() as u32) as usize
            };
            if (top + top_plus as u32) < (self.tly + self.height).trunc() as u32 {
                ystop = ((top + top_plus as u32) - self.tly.trunc() as u32) as usize
            };

            for i in ystart..ystop {
                for j in xstart..xstop {
                    let idx = model
                        .xy_to_idx(j + self.tlx.trunc() as usize, i + self.tly.trunc() as usize);
                    let mut pixel_color: Color = BLANK;
                    match model.cells[idx].grains {
                        0 => {
                            if model.cells[i].borged {
                                // untouched pixels are left transparent black
                                pixel_color = model.hues.zero_grains;
                            }
                        }
                        1 => pixel_color = model.hues.one_grain,
                        2 => pixel_color = model.hues.two_grains,
                        3 => pixel_color = model.hues.three_grains,
                        _ => pixel_color = model.hues.four_grains,
                    };
                    draw_rectangle(j as f32, i as f32, 1.0, 1.0, pixel_color);
                }
            }
        }
    }
    /// magnify_box() magnifies a 32 by 32 pixel area within the image
    pub fn magnify_box(&self, model: &Model) {
        let top_left_x = (self.mx - 16.0).trunc() as usize;
        let top_left_y = (self.my - 16.0).trunc() as usize;
        let mut bg: Color = BLACK;
        let mut curs: Color = WHITE;
        if !self.background {
            bg = WHITE;
            curs = BLACK;
        };
        for i in 0..32 {
            for j in 0..32 {
                let idx = model.xy_to_idx(
                    j + self.tlx.trunc() as usize + top_left_x,
                    i + self.tly.trunc() as usize + top_left_y,
                );
                let mut pixel_color: Color = bg; // background color instead of BLANK so unmagnified image is blocked
                match model.cells[idx].grains {
                    0 => {
                        if model.cells[idx].borged {
                            pixel_color = model.hues.zero_grains;
                        }
                    }
                    1 => pixel_color = model.hues.one_grain,
                    2 => pixel_color = model.hues.two_grains,
                    3 => pixel_color = bg,
                    _ => pixel_color = model.hues.four_grains,
                };
                draw_rectangle(
                    (self.width - 150.0) + (j * 4) as f32,
                    (self.height - 150.0) + (i * 4) as f32,
                    4.0,
                    4.0,
                    pixel_color,
                );
            }
        }
        draw_rectangle_lines(
            // draw magnified image border
            self.width - 151.0,
            self.height - 151.0,
            128.0,
            128.0,
            2.0,
            curs,
        );
        draw_rectangle_lines(
            // box around cursor
            self.mx - 16.0,
            self.my - 16.0,
            32.0,
            32.0,
            2.0,
            curs,
        );
    }
}

//marcroquad window initialization
fn window_configuration() -> Conf {
    Conf {
        window_title: "L A K H E S I S".to_owned(),
        window_width: 960,
        window_height: 540,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_configuration)]
async fn main() {
    let mut model = Model::default();
    let mut screen = Screen::default();
    // start macroquad loop
    loop {
        screen.width = screen_width(); // in case user has resized the window
        screen.height = screen_height();
        let current_ft = get_frame_time();
        if current_ft > screen.longest_ft {
            screen.longest_ft = current_ft;
        };
        screen.past_ft[screen.ft_idx] = current_ft;
        if screen.ft_idx < 31 {
            screen.ft_idx += 1
        } else {
            screen.ft_idx = 0
        };
        if screen.background {
            clear_background(BLACK);
        } else {
            clear_background(WHITE);
        }
        // draw a dot at the center of the model if visible
        let (navel_x, navel_y) = model.calc_center_xy();
        if (navel_x as f32) >= screen.tlx
            && (navel_x as f32) <= (screen.tlx + screen.width)
            && (navel_y as f32) >= screen.tly
            && (navel_y as f32) <= (screen.tly + screen.height)
        {
            let center_x: f32 = navel_x as f32 - screen.tlx;
            let center_y: f32 = navel_y as f32 - screen.tly;
            draw_rectangle(center_x, center_y, 2.0, 2.0, GRAY);
        }
        // draw abelian sand model
        screen.draw(&model);
        // get current mouse position and limit extent if magnify is on
        (screen.mx, screen.my) = mouse_position();
        if screen.magnify {
            if screen.mx < 16.0 {
                screen.mx = 16.0;
            };
            if screen.mx > screen.width - 16.0 {
                screen.mx = screen.width - 16.0;
            };
            if screen.my < 16.0 {
                screen.my = 16.0;
            };
            if screen.my > screen.height - 16.0 {
                screen.my = screen.height - 16.0;
            };
            screen.magnify_box(&model);
        }
        // is control panel visible?
        if screen.info {
            draw_rectangle(2.0, 2.0, 956.0, 75.0, DARKBLUE);
            let cross_x: usize = (screen.tlx + screen.mx).trunc() as usize;
            let cross_y: usize = (screen.tly + screen.my).trunc() as usize;
            let mut label1 = "[A]dd | [C]olors | [I]nfo | [P]ause/resume | [Spacebar]step | [Up]interval | [Down]interval | [M]agnify | [S]napshot | [CTRL][N]ew".to_string();
            if !IO_SUPPORTED {
                label1 = "[A]dd | [C]olors | [I]nfo | [P]ause/resume | [Spacebar]step | [Up]interval | [Down]interval | [M]agnify".to_string();
            }
            let label2 = format!(
				"Interval: {:5}     Active Cells: {:2}     Lattice Coordinates: ({:4}, {:4})     Sand Grain Total: {:8}     Sand Grains Lost: {}",
				&model.interval,
				&model.active_cells,
				&cross_x, &cross_y,
				&model.total_grains,
				&model.lost_grains
			);
            let label3 = format!(
                "FPS: {:2}       Current Frame Time: {:0.3} seconds       Moving Average: {:0.3} seconds       Longest Frame Time: {:0.3} seconds",
                &get_fps(),
                &current_ft,
				&screen.get_average_ft(),
                &screen.longest_ft,
            );
            draw_text(&label1, 8.0, 15.0, 16.0, WHITE);
            draw_text(&label2, 8.0, 33.0, 16.0, LIGHTGRAY);
            draw_text(&label3, 8.0, 51.0, 16.0, LIGHTGRAY);
            draw_text(&screen.context, 8.0, 69.0, 16.0, YELLOW);
            if screen.paused {
                draw_text("PAUSED", 868.0, 69.0, 16.0, ORANGE);
                if model.total_grains >= MAX_ITERATIONS {
                    screen.context = "Exceeded maximum number of sand grains".to_string();
                }
            }
        } else {
            // if control panel is hidden draw a reminder how to bring it back
            draw_rectangle(2.0, 2.0, 49.0, 18.0, DARKBLUE);
            draw_text("[I]nfo", 4.0, 15.0, 16.0, WHITE);
        }
        // draw crosshairs - thick ones if waiting to add active cell
        if screen.mx >= 0.0
            && screen.mx < screen.width
            && screen.my >= 0.0
            && screen.my < screen.height
        {
            let mut curs: Color = WHITE;
            if !screen.background {
                curs = BLACK;
            };
            if screen.add {
                draw_line(
                    screen.mx - 10.0,
                    screen.my,
                    screen.mx + 10.0,
                    screen.my,
                    3.0,
                    curs,
                );
                draw_line(
                    screen.mx,
                    screen.my - 10.0,
                    screen.mx,
                    screen.my + 10.0,
                    3.0,
                    curs,
                );
            } else {
                draw_line(
                    screen.mx - 10.0,
                    screen.my,
                    screen.mx + 10.0,
                    screen.my,
                    1.0,
                    curs,
                );
                draw_line(
                    screen.mx,
                    screen.my - 10.0,
                    screen.mx,
                    screen.my + 10.0,
                    1.0,
                    curs,
                );
            }
        }
        // has a key been pressed?
        match get_last_key_pressed() {
            Some(KeyCode::A) => {
                // add a new active cell
                if model.active_cells < MAX_DROPS {
                    screen.paused = true;
                    screen.add = true;
                    screen.context = "Use the crosshair to choose a starting point and click the left mouse button - press [ESC] to cancel".to_string();
                } else {
                    screen.context =
                        format!("Maximum number of active points ({}) reached", MAX_DROPS);
                }
            }
            Some(KeyCode::B) => screen.background = !screen.background, // toggle background between BLACK and WHITE
            Some(KeyCode::C) => model.random_colors(), // cause a random color change for sandpiles
            Some(KeyCode::G) => {
                if IO_SUPPORTED {
                    model.curate();
                } else {
                    screen.context = "Exporting data to file not supported".to_string();
                }
            }
            Some(KeyCode::H) => {
                // new simulation - reset to default
                if IO_SUPPORTED {
                    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                        model.uncurate("lakhesis.lak".to_string());
                    } else {
                        screen.context =
                            "Press [CTRL][H] to load a saved simulation named 'lakhesis.lak' or [ESC] to cancel"
                                .to_string();
                    }
                } else {
                    screen.context = "Loading from a saved file is not supported".to_string();
                }
            }
            Some(KeyCode::I) => {
                screen.info = !screen.info;
                screen.context = "Press [I] to hide this information panel          Left click the mouse to recenter the image under the crosshair".to_string();
            }
            Some(KeyCode::M) => screen.magnify = !screen.magnify,
            Some(KeyCode::N) => {
                // new simulation - reset to default
                if IO_SUPPORTED {
                    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                        screen.reset = true;
                    } else {
                        screen.context =
                            "Press [CTRL][N] to start a new simulation or [ESC] to cancel"
                                .to_string();
                    }
                } else {
                    screen.context =
                        "Click the refresh button in this browser tab to start a new simulation"
                            .to_string();
                }
            }
            Some(KeyCode::P) => screen.paused = !screen.paused, // pause or restart the simulation
            Some(KeyCode::S) => {
                if IO_SUPPORTED {
                    let (min_x, min_y, extant_width, extant_height) = model.find_extent();
                    model.paint(min_x, min_y, extant_width, extant_height);
                    screen.context = format!("Lakhesis_{:08}.png exported", &model.total_grains);
                } else {
                    screen.context = "Exporting images to file not supported".to_string();
                }
            }
            Some(KeyCode::V) => {
                // collect frames at set interval for use as a GIF or video - the PNGs will encompass the visible portion of the model
                if IO_SUPPORTED {
                    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                        screen.video = VIDEO_FRAME_COUNT;
                    } else {
                        screen.context = format!(
							"WARNING: Video will export {} PNG images! Press [Ctrl][V] to start export - pressing [ESC] will cancel this action",
							&VIDEO_FRAME_COUNT);
                    }
                } else {
                    screen.context = "Exporting images to file not supported".to_string();
                }
            }
            Some(KeyCode::Space) => {
                // spacebar to step one interval at a time
                screen.paused = true; // spacebar is frame-step, so ensure we're paused
                screen.increment = true;
                screen.context = "Press [Spacebar] to increment model one interval - press [P] to resume automatic updates".to_string();
            }
            Some(KeyCode::Up) => {
                // increase interval by 4x up to 65_536 (4^8)
                if model.interval < MAX_INTERVAL && screen.video == 0 {
                    model.interval *= 4;
                };
            }
            Some(KeyCode::Down) => {
                // decrease interval by 10x down to 1
                if model.interval > 1 && screen.video == 0 {
                    model.interval /= 4;
                };
            }
            Some(KeyCode::Escape) => {
                if screen.add {
                    screen.add = false;
                    screen.paused = false;
                }
                if screen.video > 0 {
                    screen.video = 0;
                };
                screen.context = "Press [I] to hide this control panel          Left click the mouse to recenter the image under the crosshair".to_string();
            }
            None => (),
            _ => (),
        }
        // is left mouse button pressed
        if is_mouse_button_pressed(MouseButton::Left) {
            if screen.add {
                // if add - get mouse position to add new drop cell
                model.active_cells += 1;
                model.drop_cells[model.active_cells - 1] = model.xy_to_idx(
                    (screen.mx + screen.tlx).trunc() as usize,
                    (screen.my + screen.tly).trunc() as usize,
                );
                screen.paused = false;
                screen.add = false;
                screen.context = "Press [I] to hide this control panel          Left click the mouse to recenter the image under the crosshair".to_string();
            } else {
                // recenter screen on mouse position and check boundaries
                screen.tlx += screen.mx - (screen.width / 2.0).trunc();
                if screen.tlx < 0.0 {
                    screen.tlx = 0.0;
                };
                if (screen.tlx + screen.width) >= MODEL_WIDTH as f32 {
                    screen.tlx = MODEL_WIDTH as f32 - screen.width;
                };
                screen.tly += screen.my - (screen.height / 2.0).trunc();
                if screen.tly < 0.0 {
                    screen.tly = 0.0;
                };
                if (screen.tly + screen.height) >= MODEL_HEIGHT as f32 {
                    screen.tly = MODEL_HEIGHT as f32 - screen.height;
                };
            }
        }
        // if !paused or spacebar pressed and a drop cell is added, drop sand grains and resolve unstable sandpiles
        if (!screen.paused || screen.increment) && model.active_cells > 0 {
            for _ in 0..model.interval {
                model.add_grain(screen.ac);
                if model.active_cells - 1 > screen.ac {
                    screen.ac += 1;
                } else {
                    screen.ac = 0;
                };
            }
            if screen.video > 0 {
                model.paint(
                    screen.tlx.trunc() as u32,
                    screen.tly.trunc() as u32,
                    screen.width.trunc() as u16,
                    screen.height.trunc() as u16,
                );
                screen.video -= 1;
                screen.context = format!(
                    "Recording video frame {} of {} - Press [ESC] to cancel",
                    VIDEO_FRAME_COUNT - screen.video,
                    VIDEO_FRAME_COUNT
                );
            }
            screen.increment = false;
        }
        if model.total_grains >= MAX_ITERATIONS {
            screen.paused = true;
        }
        if screen.reset {
            model = Model::default();
            screen = Screen::default();
        }
        next_frame().await;
    }
}

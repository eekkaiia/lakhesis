use crate::{Model, Screen, MAX_DROPS};

use macroquad::input::*;
use macroquad::math::*;
use macroquad::time::*;
use macroquad::ui::{hash, root_ui, widgets};

// maximum interval in sand grains added between screen updates - any higher and simulation becomes unresponsive
const MAX_INTERVAL: usize = 16_384;
// number of PNG frames to create 10 second video at 60fps
const VIDEO_FRAME_COUNT: usize = 600;
// set IO_SUPPORTED to false when compiling for WASM - currently saving an image to disk is not supported from web browser
const IO_SUPPORTED: bool = true;

#[derive(Clone, Debug)]
pub struct Info {
    pub lattice_x: f32,
    pub lattice_y: f32,
    pub context: String,
    pub current_ft: f32,
    pub longest_ft: f32,
    pub running_ft: [f32; 1024],
    pub ft_idx: usize,
    pub average_ft: f32,
}
impl Info {
    pub fn default() -> Self {
        let running_ft = [0.0; 1024];
        Self {
            lattice_x: 0.0,
            lattice_y: 0.0,
            context: "Start a simulation by clicking the 'Add' button below or by pressing the [A] key to add a new sandpile".to_string(),
            current_ft: 0.0,
            longest_ft: 0.0,
            running_ft,
            ft_idx: 0,
            average_ft: 0.0,
        }
    }
    /// get_average_ft() calculates the average frame time over the last 32 frames
    fn get_average_ft(&self) -> f32 {
        let mut average: f32 = 0.0;
        for i in 0..1024 {
            average += self.running_ft[i];
        }
        average / 1024.0
    }
    /// update() the info panel this frame
    pub fn update(&mut self, screen: &Screen) {
        self.current_ft = get_frame_time();
        if self.current_ft > self.longest_ft {
            self.longest_ft = self.current_ft;
        }
        self.running_ft[self.ft_idx] = self.current_ft;
        if self.ft_idx < 1023 {
            self.ft_idx += 1
        } else {
            self.ft_idx = 0
        }
        self.average_ft = self.get_average_ft();
        self.lattice_x = screen.tlx + screen.mx;
        self.lattice_y = screen.tly + screen.my;
    }
}

#[derive(Clone, Debug, Default)]
pub struct Control {
    pub visible: bool,
    pub magnify: bool,
    pub paused: bool,
    pub add: bool,
    pub reset: bool,
    pub increment: bool,
    pub video: usize,
    pub color: bool,
}
impl Control {
    pub fn default() -> Self {
        Self {
            visible: true,
            magnify: false,
            paused: true,
            add: false,
            reset: false,
            increment: false,
            video: 0,
            color: false,
        }
    }
    /// draw_panel(), if visible, to provide control options
    pub fn draw_panel(&mut self, model: &mut Model, info: &mut Info, screen: &mut Screen) {
        root_ui().window(hash!(), Vec2::new(18., 19.), Vec2::new(238., 276.), |ui| {
            widgets::Group::new(hash!(), Vec2::new(80., 200.))
                .position(Vec2::new(155., 1.))
                .ui(ui, |ui| {
                    if widgets::Button::new("ADD").size(vec2(75., 26.)).ui(ui) {
                        if model.active_cells < MAX_DROPS {
                            self.paused = true;
                            self.add = true;
                            info.context = "Use the crosshair to choose a starting point and click the left mouse button - press [ESC] to cancel".to_string();
                        } else {
                            info.context =
                                format!("Maximum number of active points ({}) reached", MAX_DROPS);
                        }
                    }
                    if widgets::Button::new("PAUSE").size(vec2(75., 26.)).ui(ui) {
                        self.paused = !self.paused;
                    }
                    if widgets::Button::new("STEP").size(vec2(75., 26.)).ui(ui) {
                        self.increment = !self.increment;
                        self.paused = true; // spacebar is frame-step, so ensure we're paused
                        info.context = "Click 'STEP' again or press [Spacebar] to increment model one interval - 'PAUSE' or [P] key to resume automatic updates".to_string();

                    }
                    if widgets::Button::new("MAGNIFY").size(vec2(75., 26.)).ui(ui) {
                        self.magnify = !self.magnify;
                    }
                    if widgets::Button::new("COLOR").size(vec2(75., 26.)).ui(ui) {
                        self.color = true;
                        self.paused = true;
                    }
                    if widgets::Button::new("SNAPSHOT").size(vec2(75., 26.)).ui(ui) {
                        if IO_SUPPORTED {
                            let (min_x, min_y, extant_width, extant_height) = model.find_extent();
                            model.paint(min_x, min_y, extant_width, extant_height);
                            info.context = format!("Lakhesis_{:08}.png exported", &model.total_grains);
                        } else {
                            info.context = "Exporting images to file not supported".to_string();
                        }
                    }
                    if widgets::Button::new("RESET").size(vec2(75., 26.)).ui(ui) {
                        self.reset = true;
                    }
                });
            widgets::Group::new(hash!(), Vec2::new(153., 200.))
                .position(Vec2::new(1., 1.))
                .ui(ui, |ui| {
                    ui.label(Vec2::new(7., 0.), &format!("Interval:     {:5}", &model.interval));
                    ui.label(Vec2::new(7., 15.), &format!("Sandpiles:       {:2}", &model.active_cells));
                    ui.label(Vec2::new(7., 35.), "Sand Grains");
                    ui.label(Vec2::new(7., 50.), &format!("Total:   {:10}", &model.total_grains));
                    ui.label(Vec2::new(7., 65.), &format!("Lost:    {:10}", &model.lost_grains));
                    ui.label(Vec2::new(7., 85.), "Frame Times");
                    ui.label(Vec2::new(7., 100.), &format!("FPS:       {:2}", &get_fps()));
                    ui.label(Vec2::new(7., 115.), &format!("Current:   {:8.5}", &info.current_ft));
                    ui.label(Vec2::new(7., 130.), &format!("Average:   {:8.5}", &info.average_ft));
                    ui.label(Vec2::new(7., 145.), &format!("Longest:   {:8.5}", &info.longest_ft));
                    ui.label(Vec2::new(7., 165.), "Lattice Coordinates");
                    ui.label(Vec2::new(7., 180.), &format!("x: {:4}     y: {:4}", &info.lattice_x, &info.lattice_y));
                });
            widgets::Group::new(hash!(), Vec2::new(80. ,70.))
                .position(Vec2::new(1., 202.))
                .ui(ui, |ui| {
                    if widgets::Button::new("INCREASE").size(vec2(75., 20.)).ui(ui) {
                        // increase interval by 4x up to 65_536 (4^8)
                        if model.interval < MAX_INTERVAL && self.video == 0 {
                            model.interval *= 4;
                        };
                    }
                    if widgets::Button::new("DECREASE").size(vec2(75., 20.)).ui(ui) {
                        // decrease interval by 4x down to 1
                        if model.interval > 1 && self.video == 0 {
                            model.interval /= 4;
                        };
                    }
                    ui.label(Vec2::new(8., 45.), "Interval");
                });
            widgets::Group::new(hash!(), Vec2::new(153. ,70.))
                .position(Vec2::new(82., 202.))
                .ui(ui, |ui| {
                    ui.label(None, "     ");
                    ui.same_line(0.);
                    if widgets::Button::new("UP").size(vec2(70., 20.)).ui(ui) {
                        // move screen up 64 pixels
                        screen.tly -= 64.0;
                        if screen.tly < 0.0 {
                            screen.tly = 0.0;
                        };
                    }
                    if widgets::Button::new("LEFT").size(vec2(60., 20.)).ui(ui) {
                        // move screen left 64 pixels
                        screen.tlx -= 64.0;
                        if screen.tlx < 0.0 {
                            screen.tlx = 0.0;
                        };
                    }
                    ui.same_line(0.);
                    if widgets::Button::new("O").size(vec2(20., 20.)).ui(ui) {
                        // center screen
                        screen.tlx = (model.width as f32 - screen.width).abs() / 2.0;
                        screen.tly = (model.height as f32 - screen.height).abs() / 2.0;
                    }
                    ui.same_line(0.);
                    if widgets::Button::new("RIGHT").size(vec2(60., 20.)).ui(ui) {
                        // move screen right 64 pixels
                        screen.tlx += 64.0;
                        if (screen.tlx + screen.width) >= model.width as f32 {
                            screen.tlx = model.width as f32 - screen.width;
                        };
                    }
                    ui.label(None, "     ");
                    ui.same_line(0.);
                    if widgets::Button::new("DOWN").size(vec2(70., 20.)).ui(ui) {
                        // move screen down 64 pixels
                        screen.tly += 64.0;
                        if (screen.tly + screen.height) >= model.height as f32 {
                            screen.tly = model.height as f32 - screen.height;
                        };
                    }
                });
        });
    }
    /// Check for keyboard commands
    pub fn check_keyboard(&mut self, model: &mut Model, info: &mut Info, screen: &mut Screen) {
        match get_last_key_pressed() {
            Some(KeyCode::A) => {
                // add a new active cell
                if model.active_cells < MAX_DROPS {
                    self.paused = true;
                    self.add = true;
                    info.context = "Use the crosshair to choose a starting point and click the left mouse button - press [ESC] to cancel".to_string();
                } else {
                    info.context =
                        format!("Maximum number of active points ({}) reached", MAX_DROPS);
                }
            }
            Some(KeyCode::B) => screen.background = !screen.background, // toggle background between BLACK and WHITE
            Some(KeyCode::C) => {
                // cause a color change for sandpiles
                self.color = true;
                self.paused = true;
            }
            Some(KeyCode::G) => {
                if IO_SUPPORTED {
                    model.curate();
                } else {
                    info.context = "Exporting data to file not supported".to_string();
                }
            }
            Some(KeyCode::H) => {
                // new simulation - reset to default
                if IO_SUPPORTED {
                    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                        model.uncurate("lakhesis.lak".to_string());
                    } else {
                        info.context =
                            "Press [CTRL][H] to load a saved simulation named 'lakhesis.lak' or [ESC] to cancel"
                                .to_string();
                    }
                } else {
                    info.context = "Loading from a saved file is not supported".to_string();
                }
            }
            Some(KeyCode::I) => {
                self.visible = !self.visible;
                info.context = "<--Click here to hide the control panel".to_string();
            }
            Some(KeyCode::M) => self.magnify = !self.magnify,
            Some(KeyCode::N) => {
                // new simulation - reset to default
                if IO_SUPPORTED {
                    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                        self.reset = true;
                    } else {
                        info.context =
                            "Press [CTRL][N] to start a new simulation or [ESC] to cancel"
                                .to_string();
                    }
                } else {
                    info.context =
                        "Click the refresh button in this browser tab to start a new simulation"
                            .to_string();
                }
            }
            Some(KeyCode::P) => self.paused = !self.paused, // pause or restart the simulation
            Some(KeyCode::S) => {
                if IO_SUPPORTED {
                    let (min_x, min_y, extant_width, extant_height) = model.find_extent();
                    model.paint(min_x, min_y, extant_width, extant_height);
                    info.context = format!("Lakhesis_{:08}.png exported", &model.total_grains);
                } else {
                    info.context = "Exporting images to file not supported".to_string();
                }
            }
            Some(KeyCode::V) => {
                // collect frames at set interval for use as a GIF or video - the PNGs will encompass the visible portion of the model
                if IO_SUPPORTED {
                    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                        self.video = VIDEO_FRAME_COUNT;
                    } else {
                        info.context = format!(
							"WARNING: Video will export {} PNG images! Press [Ctrl][V] to start export - pressing [ESC] will cancel this action",
							&VIDEO_FRAME_COUNT);
                    }
                } else {
                    info.context = "Exporting images to file not supported".to_string();
                }
            }
            Some(KeyCode::Space) => {
                // spacebar to step one interval at a time
                self.paused = true; // spacebar is frame-step, so ensure we're paused
                self.increment = true;
                info.context = "Press [Spacebar] again or click 'STEP' to increment model one interval - 'PAUSE' or [P] key to resume automatic updates".to_string();
            }
            Some(KeyCode::Up) => {
                // increase interval by 4x up to 65_536 (4^8)
                if model.interval < MAX_INTERVAL && self.video == 0 {
                    model.interval *= 4;
                };
            }
            Some(KeyCode::Down) => {
                // decrease interval by 4x down to 1
                if model.interval > 1 && self.video == 0 {
                    model.interval /= 4;
                };
            }
            Some(KeyCode::Escape) => {
                if self.add {
                    self.add = false;
                    self.paused = false;
                }
                if self.video > 0 {
                    self.video = 0;
                };
                info.context = "<--Click here to hide this control panel".to_string();
            }
            None => (),
            _ => (),
        }
    }
}

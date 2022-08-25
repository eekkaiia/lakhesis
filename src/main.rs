use lakhesis::{Model, MAX_DROPS, MAX_ITERATIONS, MODEL_HEIGHT, MODEL_WIDTH};

// use macroquad::prelude::*;
// use macroquad::camera::*;
// use macroquad::file::*;
use macroquad::input::*;
//use macroquad::material::*;
use macroquad::math::*;
//use macroquad::models::*;
use macroquad::shapes::*;
use macroquad::text::*;
// use macroquad::texture::*;
use macroquad::color::colors::*;
use macroquad::color::Color;
use macroquad::time::*;
use macroquad::window::*;
// use glam;
// use quad_rand as rand;
// use macroquad::experimental::*;
// use macroquad::color_u8;

// maximum interval in sand grains added between screen updates - any higher and simulation becomes unresponsive
pub const MAX_INTERVAL: usize = 16_384;
// number of PNG frames to create 10 second video at 60fps
const VIDEO_FRAME_COUNT: usize = 600;
// set IO_SUPPORTED to false when compiling for WASM - currently saving an image to disk is not supported from web browser
const IO_SUPPORTED: bool = true;

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
    let mut ac: usize = 0;
    let mut add: bool = false;
    let mut reset: bool = false;

    let mut paused: bool = true;
    let mut increment: bool = false;
    let mut info: bool = true;
    let mut magnify: bool = false;
    let mut background: bool = true; // toggles between a black or white background
    let mut video: usize = 0;
    let mut temp: String = "To start the abelian sand model press the [A] key".to_string();
    let mut context: Option<&str> = Some(&temp);
    let mut longest_ft: f32 = 0.0;

    let mut tlx_screen: f32 = (MODEL_WIDTH as f32 - 960.0).abs() / 2.0; // initial top left corner of screen
    let mut tly_screen: f32 = (MODEL_HEIGHT as f32 - 540.0).abs() / 2.0; // centered on model

    // start macroquad loop
    loop {
        let screen_width = screen_width(); // in case user has resized the window
        let screen_height = screen_height();
        let current_ft = get_frame_time();
        if current_ft > longest_ft {
            longest_ft = current_ft;
        };

        if background {
            clear_background(BLACK);
        } else {
            clear_background(WHITE);
        }
        // draw a dot at the center of the model if visible
        let (navel_x, navel_y) = model.calc_center_xy();
        if (navel_x as f32) >= tlx_screen
            && (navel_x as f32) <= (tlx_screen + screen_width)
            && (navel_y as f32) >= tly_screen
            && (navel_y as f32) <= (tly_screen + screen_height)
        {
            let center_x: f32 = navel_x as f32 - tlx_screen;
            let center_y: f32 = navel_y as f32 - tly_screen;
            draw_rectangle(center_x, center_y, 2.0, 2.0, LIGHTGRAY);
        }
        // draw abelian sand model
        draw(&model, tlx_screen, tly_screen, screen_width, screen_height);
        // get mouse position and limit extent if magnify is on
        let (mut mx, mut my) = mouse_position();
        if magnify {
            if mx < 16.0 {
                mx = 16.0;
            };
            if mx > screen_width - 16.0 {
                mx = screen_width - 16.0;
            };
            if my < 16.0 {
                my = 16.0;
            };
            if my > screen_height - 16.0 {
                my = screen_height - 16.0;
            };
            magnify_box(
                &model,
                &mx,
                &my,
                tlx_screen,
                tly_screen,
                screen_width,
                screen_height,
                background,
            );
        }
        if info {
            // is control panel visible?
            draw_rectangle(2.0, 2.0, 958.0, 75.0, DARKBLUE);
            let cross_x = tlx_screen + mx;
            let cross_y = tly_screen + my;
            let mut label1 = "[A]dd | [C]olors | [I]nfo | [P]ause/resume | [Spacebar]step | [Up]interval | [Down]interval | [M]agnify | [S]napshot | [CTRL][N]ew".to_string();
            if !IO_SUPPORTED {
                label1 = "[A]dd | [C]olors | [I]nfo | [P]ause/resume | [Spacebar]step | [Up]interval | [Down]interval | [M]agnify".to_string();
            }
            let label2 = format!("Interval: {}   Active Cells: {}   Coordinates: ({}, {})   Sand grain total: {}   Sand grains lost: {}",
							&model.interval, &model.active_cells, &cross_x, &cross_y, &model.total_grains, &model.lost_grains);
            let label3 = format!(
                "FPS: {}   Last frame time: {:08.4} seconds,   Longest frame time: {:08.4} seconds",
                &get_fps(),
                &current_ft,
                &longest_ft
            );
            draw_text(&label1, 5.0, 15.0, 16.0, WHITE);
            draw_text(&label2, 5.0, 33.0, 16.0, WHITE);
            draw_text(&label3, 5.0, 51.0, 16.0, WHITE);
            if context == None {
                draw_text("Press [I] to hide this control panel          Left click the mouse to recenter the image under the crosshair", 5.0, 69.0, 16.0, YELLOW)
            } else {
                draw_text(context.unwrap(), 5.0, 69.0, 16.0, YELLOW)
            };

            if paused {
                draw_text("PAUSED", 868.0, 69.0, 16.0, ORANGE);
                if model.total_grains >= MAX_ITERATIONS {
                    context = Some("Exceeded maximum number of sand grains");
                }
            }
        } else {
            // if control panel is hidden draw a reminder how to bring it back
            draw_rectangle(2.0, 2.0, 49.0, 18.0, DARKBLUE);
            draw_text("[I]nfo", 4.0, 15.0, 16.0, WHITE);
        }
        // draw crosshairs - thick ones if waiting to add active cell
        if mx >= 0.0 && mx < screen_width && my >= 0.0 && my < screen_height {
            if add {
                draw_line(mx - 10.0, my, mx + 10.0, my, 3.0, BLUE);
                draw_line(mx, my - 10.0, mx, my + 10.0, 3.0, BLUE);
            } else {
                draw_line(mx - 10.0, my, mx + 10.0, my, 1.0, BLUE);
                draw_line(mx, my - 10.0, mx, my + 10.0, 1.0, BLUE);
            }
        }
        // has a key been pressed?
        match get_last_key_pressed() {
            Some(KeyCode::A) => {
                // add a new active cell
                if model.active_cells < MAX_DROPS {
                    paused = true;
                    add = true;
                    context = Some("Use the crosshair to choose a starting point and click the left mouse button - press [ESC] to cancel");
                } else {
                    temp = format!("Maximum number of active points ({}) reached", MAX_DROPS);
                    context = Some(&temp);
                }
            }
            Some(KeyCode::B) => background = !background, // toggle background between BLACK and WHITE
            Some(KeyCode::C) => model.random_colors(), // cause a random color change for sandpiles
            Some(KeyCode::I) => {
                info = !info;
                context = None;
            }
            Some(KeyCode::M) => magnify = !magnify,
            Some(KeyCode::N) => {
                // new simulation - reset to default
                if IO_SUPPORTED {
                    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                        reset = true;
                    } else {
                        context =
                            Some("Press [CTRL][N] to start a new simulation or [ESC] to cancel");
                    }
                } else {
                    context = Some(
                        "Click the refresh button in this browser tab to start a new simulation",
                    );
                }
            }
            Some(KeyCode::P) => paused = !paused, // pause or restart the simulation
            Some(KeyCode::S) => {
                if IO_SUPPORTED {
                    let (min_x, min_y, extant_width, extant_height) = model.find_extent();
                    model.paint(min_x, min_y, extant_width, extant_height);
                    temp = format!("Lakhesis_{:08}.png exported", &model.total_grains);
                    context = Some(&temp);
                } else {
                    context = Some("Exporting images to file not supported");
                }
            }
            Some(KeyCode::V) => {
                // collect frames at set interval for use as a GIF or video - the PNGs will encompass the visible portion of the model
                if IO_SUPPORTED {
                    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                        video = VIDEO_FRAME_COUNT;
                    } else {
                        temp = format!(
							"WARNING: Video will export {} PNG images! Press [Ctrl][V] to start export - pressing [ESC] will cancel this action",
							&VIDEO_FRAME_COUNT);
                        context = Some(&temp);
                    }
                } else {
                    context = Some("Exporting images to file not supported");
                }
            }
            Some(KeyCode::Space) => {
                // spacebar to step one interval at a time
                paused = true; // spacebar is frame-step, so ensure we're paused
                increment = true;
                context = Some("Press [Spacebar] to increment model one interval - press [P] to resume automatic updates");
            }
            Some(KeyCode::Up) => {
                // increase interval by 4x up to 65_536 (4^8)
                if model.interval < MAX_INTERVAL && video == 0 {
                    model.interval *= 4;
                };
            }
            Some(KeyCode::Down) => {
                // decrease interval by 10x down to 1
                if model.interval > 1 && video == 0 {
                    model.interval /= 4;
                };
            }
            Some(KeyCode::Escape) => {
                if add {
                    add = false;
                    paused = false;
                }
                if video > 0 {
                    video = 0;
                };
                context = None;
            }
            None => (),
            _ => (),
        }
        // is left mouse button pressed
        if is_mouse_button_pressed(MouseButton::Left) {
            if add {
                // if add - get mouse position to add new drop cell
                // model.xy_to_idx((mx + tlx_screen).trunc() as usize, (my + tly_screen).trunc() as usize) < model.cells.len()

                model.active_cells += 1;
                model.drop_cells[model.active_cells - 1] = model.xy_to_idx(
                    (mx + tlx_screen).trunc() as usize,
                    (my + tly_screen).trunc() as usize,
                );
                paused = false;
                add = false;
                context = None;
            } else {
                // recenter screen on mouse position
                tlx_screen += mx - (screen_width / 2.0).trunc();
                if tlx_screen < 0.0 {
                    tlx_screen = 0.0;
                };
                if (tlx_screen + screen_width) > MODEL_WIDTH as f32 {
                    tlx_screen = MODEL_WIDTH as f32 - screen_width
                };
                tly_screen += my - (screen_height / 2.0).trunc();
                if tly_screen < 0.0 {
                    tlx_screen = 0.0;
                };
                if (tly_screen + screen_height) > MODEL_HEIGHT as f32 {
                    tly_screen = MODEL_HEIGHT as f32 - screen_height
                };
            }
        }
        // if !paused or spacebar pressed and a drop cell is added, drop sand grains and resolve unstable sandpiles
        if (!paused || increment) && model.active_cells > 0 {
            for _ in 0..model.interval {
                model.add_grain(ac);
                if model.active_cells - 1 > ac {
                    ac += 1;
                } else {
                    ac = 0;
                };
            }
            if video > 0 {
                model.paint(
                    tlx_screen.trunc() as u32,
                    tly_screen.trunc() as u32,
                    screen_width.trunc() as u16,
                    screen_height.trunc() as u16,
                );
                video -= 1;
                temp = format!(
                    "Recording video frame {} of {} - Press [ESC] to cancel",
                    VIDEO_FRAME_COUNT - video,
                    VIDEO_FRAME_COUNT
                );
                context = Some(&temp);
            }
            increment = false;
        }
        if model.total_grains >= MAX_ITERATIONS {
            paused = true;
        }
        if reset {
            model = Model::default();
            paused = true;
            increment = false;
            video = 0;
            reset = false;
            add = false;
            ac = 0;
            magnify = false;
            context = Some("To start the abelian sand model press the [A] key");
        }
        next_frame().await;
    }
}
/// draw() maps visible portion of model to screen
fn draw(model: &Model, tlx_screen: f32, tly_screen: f32, screen_width: f32, screen_height: f32) {
    for i in 0..screen_height.trunc() as usize {
        for j in 0..screen_width.trunc() as usize {
            let idx = model.xy_to_idx(
                j + tlx_screen.trunc() as usize,
                i + tly_screen.trunc() as usize,
            );
            let mut pixel_color: Color = BLANK;
            match model.cells[idx].grains {
                0 => {
                    if model.cells[i].collapses != 0 {
                        // if untouched pixels are left transparent black
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
/// magnify_box() magnifies a 32 by 32 pixel area within the image
fn magnify_box(
    model: &Model,
    mx: &f32,
    my: &f32,
    tlx_screen: f32,
    tly_screen: f32,
    screen_width: f32,
    screen_height: f32,
    background: bool,
) {
    let top_left_x = (mx - 16.0).trunc() as usize;
    let top_left_y = (my - 16.0).trunc() as usize;
    let mut bg: Color = BLACK;
    if !background {
        bg = WHITE;
    };
    for i in 0..32 {
        for j in 0..32 {
            let idx = model.xy_to_idx(
                j + tlx_screen.trunc() as usize + top_left_x,
                i + tly_screen.trunc() as usize + top_left_y,
            );
            let mut pixel_color: Color = bg; // background color instead of BLANK so unmagnified image is blocked
            match model.cells[idx].grains {
                0 => {
                    if model.cells[idx].collapses != 0 {
                        pixel_color = model.hues.zero_grains;
                    }
                }
                1 => pixel_color = model.hues.one_grain,
                2 => pixel_color = model.hues.two_grains,
                3 => pixel_color = bg,
                _ => pixel_color = model.hues.four_grains,
            };
            draw_rectangle(
                (screen_width - 150.0) + (j * 4) as f32,
                (screen_height - 150.0) + (i * 4) as f32,
                4.0,
                4.0,
                pixel_color,
            );
        }
    }
    draw_rectangle_lines(
        // draw magnified image border
        screen_width - 151.0,
        screen_height - 151.0,
        128.0,
        128.0,
        2.0,
        SKYBLUE,
    );
    draw_rectangle_lines(
        // box around cursor
        mx - 16.0,
        my - 16.0,
        32.0,
        32.0,
        2.0,
        SKYBLUE,
    );
}

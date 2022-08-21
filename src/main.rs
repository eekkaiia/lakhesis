use lakhesis::{Model, MAX_DROPS, MAX_ITERATIONS, MODEL_HEIGHT, MODEL_WIDTH};
use macroquad::prelude::*;

// number of PNG frames to create 10 second video at 60fps
const VIDEO_FRAME_COUNT: usize = 600;
// set IO_SUPPORTED to false when compiling for WASM - currently saving an image to disk is not supported from web browser
const IO_SUPPORTED: bool = true;

//marcroquad window initialization
fn window_configuration() -> Conf {
    Conf {
        window_title: "L A K H E S I S".to_owned(),
        window_width: 1280, // 1280 x 720 for 720p images
        window_height: 800, // top 720 for image - bottom 80 for info and stats
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_configuration)]
async fn main() {
    let mut model = Model::default();
    let mut user_decided_to_exit: bool = false;
    let mut paused: bool = true;
    let mut increment: bool = false;
    let mut reset: bool = false;
    let mut video: usize = 0;
    let mut add: bool = false;
    let mut ac: usize = 0;
    let mut magnify: bool = false;
    let mut temp: String =
        "To start the abelian sand model press the [A] key - the dot is the center of the frame"
            .to_string();
    let mut context: Option<&str> = Some(&temp);

    clear_background(BLACK);
    // start macroquad loop
    loop {
        // draw centerpoint - will eventually be hidden by model
        let (ccol, crow) = model.calc_center_xy();
        draw_rectangle(ccol as f32 - 1.0, crow as f32 - 1.0, 2.0, 2.0, WHITE);
        //draw abelian sand model
        draw(&model);
        // get mouse position and limit extent if magnify is on
        let (mut mx, mut my) = mouse_position();
        if magnify {
            if mx < 16.0 {
                mx = 16.0;
            };
            if mx > (MODEL_WIDTH - 16) as f32 {
                mx = (MODEL_WIDTH - 16) as f32;
            };
            if my < 16.0 {
                my = 16.0;
            };
            if my > (MODEL_HEIGHT - 16) as f32 {
                my = (MODEL_HEIGHT - 16) as f32;
            };
            magnify_box(&model, &mx, &my);
        }

        let mut label1 = "[A]dd | [C]olors | [P]ause/resume | [Spacebar]step | [Up]interval | [Down]interval | [M]agnify | [S]napshot | [V]ideo | [CTRL][N]ew | [CTRL][Q]uit".to_string();
        if !IO_SUPPORTED {
            label1 = "[A]dd | [C]olors | [P]ause/resume | [Spacebar]step | [Up]interval | [Down]interval | [M]agnify | [CTRL][N]ew | [Q]uit".to_string();
        }
        let label2 = format!("Grains Dropped: {}    Grains Lost: {}    Interval: {}    Active Cells: {}    Mouse position: {} {}    FPS: {}    Frame time: {:07.4} seconds",
							&model.total_grains, &model.lost_grains, &model.interval, &model.active_cells, &mx, &my, &get_fps(), &get_frame_time());

        draw_text(&label1, 20.0, 739.0, 16.0, WHITE);
        draw_text(&label2, 20.0, 759.0, 16.0, WHITE);
        if context != None {
            draw_text(context.unwrap(), 20.0, 779.0, 16.0, YELLOW)
        };

        if paused {
            draw_text("PAUSED", 1163.0, 742.0, 24.0, ORANGE);
            if model.total_grains >= MAX_ITERATIONS {
                context = Some("Exceeded maximum number of sand grains");
            }
        }
        // draw box around control area
        draw_rectangle_lines(
            1.0,
            MODEL_HEIGHT as f32 + 1.0,
            MODEL_WIDTH as f32 - 1.0,
            79.0,
            2.0,
            WHITE,
        );
        // draw crosshairs - thick ones if waiting to add active cell
        if mx >= 0.0 && mx < MODEL_WIDTH as f32 && my >= 0.0 && my < MODEL_HEIGHT as f32 {
            if add {
                draw_line(mx - 10.0, my, mx + 10.0, my, 3.0, WHITE);
                draw_line(mx, my - 10.0, mx, my + 10.0, 3.0, WHITE);
            } else {
                draw_line(mx - 10.0, my, mx + 10.0, my, 1.0, WHITE);
                draw_line(mx, my - 10.0, mx, my + 10.0, 1.0, WHITE);
            }
        }
        match get_last_key_pressed() {
            Some(KeyCode::A) => {
                // add a new active cell
                if model.active_cells < MAX_DROPS {
                    paused = true;
                    add = true;
                    temp = format!("Use the crosshairs to pick a point in the area above and click the left mouse button - up to {} points can be added", MAX_DROPS);
                    context = Some(&temp);
                } else {
                    temp = format!("Maximum number of active points ({}) reached", MAX_DROPS);
                    context = Some(&temp);
                }
            }
            Some(KeyCode::C) => model.random_colors(), // cause a random color change for sandpiles
            Some(KeyCode::M) => magnify = !magnify,
            Some(KeyCode::N) => {
                // new simulation - reset to default
                if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                    reset = true;
                } else {
                    context = Some("Press [CTRL][N] to start a new simulation or [ESC] to cancel");
                }
            }
            Some(KeyCode::P) => paused = !paused, // pause or restart the simulation
            Some(KeyCode::Q) => {
                // Q to quit
                if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                    user_decided_to_exit = true;
                } else {
                    context = Some("Press [CTRL][Q] to Exit");
                }
            }
            Some(KeyCode::S) => {
                // save image and model up to this point
                if IO_SUPPORTED {
                    model.paint();
                    temp = format!("Lakhesis_{:07}.png exported", &model.total_grains);
                    context = Some(&temp);
                } else {
                    context = Some("Exporting images to file not supported");
                }
            }
            Some(KeyCode::V) => {
                // collect frames at set interval for use as a GIF or video
                if IO_SUPPORTED {
                    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
                        video = VIDEO_FRAME_COUNT;
                    } else {
                        temp = format!("WARNING: Video will export {} PNG images! Press [Ctrl][V] to start export - pressing [ESC] will cancel this action", &VIDEO_FRAME_COUNT);
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
            }
            Some(KeyCode::Up) => {
                // increase interval by 4x up to 65_536 (4^8)
                if model.interval < 65_536 && video == 0 {
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
        // if add - get mouse position on left click to add new drop cell
        if add
            && is_mouse_button_pressed(MouseButton::Left)
            && model.xy_to_idx(mx.trunc() as usize, my.trunc() as usize) < model.cells.len()
        {
            model.active_cells += 1;
            model.drop_cells[model.active_cells - 1] =
                model.xy_to_idx(mx.trunc() as usize, my.trunc() as usize);
            paused = false;
            add = false;
            context = None;
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
                model.paint();
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
        if user_decided_to_exit {
            break;
        }

        next_frame().await;
    }
}
/// draw() refreshs image on screen
fn draw(model: &Model) {
    for i in 0..model.cells.len() {
        let mut pixel_color: Color = BLANK;
        match model.cells[i].grains {
            0 => {
                if model.cells[i].collapses != 0 {
                    pixel_color = model.hues.zero_grains;
                }
            } // if untouched pixels are transparent black
            1 => pixel_color = model.hues.one_grain,
            2 => pixel_color = model.hues.two_grains,
            3 => pixel_color = model.hues.three_grains,
            _ => pixel_color = model.hues.four_grains,
        };
        let (col, row) = model.idx_to_xy(i);
        draw_rectangle(col as f32, row as f32, 1.0, 1.0, pixel_color);
    }
}
/// magnify_box() creates a small area magnifying a 32 by 32 pixel area within the image
fn magnify_box(model: &Model, mx: &f32, my: &f32) {
    let top_left_x = (mx - 16.0).trunc() as usize;
    let top_left_y = (my - 16.0).trunc() as usize;
    for i in 0..32 {
        for j in 0..32 {
            let idx = model.xy_to_idx(j + top_left_x, i + top_left_y);
            let mut pixel_color: Color = BLANK;
            match model.cells[idx].grains {
                0 => {
                    if model.cells[idx].collapses != 0 {
                        pixel_color = model.hues.zero_grains;
                    }
                }
                1 => pixel_color = model.hues.one_grain,
                2 => pixel_color = model.hues.two_grains,
                3 => pixel_color = model.hues.three_grains,
                _ => pixel_color = model.hues.four_grains,
            };

            draw_rectangle(
                (MODEL_WIDTH as f32 - 150.0) + (j * 4) as f32,
                (MODEL_HEIGHT as f32 - 150.0) + (i * 4) as f32,
                4.0,
                4.0,
                pixel_color,
            );
        }
    }
    draw_rectangle_lines(
        // draw magnified image border
        MODEL_WIDTH as f32 - 151.0,
        MODEL_HEIGHT as f32 - 151.0,
        128.0,
        128.0,
        2.0,
        WHITE,
    );
    draw_rectangle_lines(
        // box around cursor
        mx - 16.0,
        my - 16.0,
        32.0,
        32.0,
        2.0,
        WHITE,
    );
}

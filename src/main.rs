use lakhesis::Screen;
use lakhesis::{Control, Info};
use lakhesis::{Csliders, RevertColor, Selected};
use lakhesis::{Model, MAX_ITERATIONS};

use macroquad::color::colors::*;
use macroquad::input::*;
use macroquad::math::*;
use macroquad::shapes::*;
use macroquad::text::*;
use macroquad::ui::root_ui;
use macroquad::window::*;

// number of PNG frames to create 10 second video at 60fps
const VIDEO_FRAME_COUNT: usize = 600;

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
    let mut screen = Screen::default(&model);
    let mut info = Info::default();
    let mut control = Control::default();
    let mut csliders: Csliders = Csliders {
        selected: Selected::One,
        red: model.hues.one_grain.r,
        green: model.hues.one_grain.g,
        blue: model.hues.one_grain.b,
        alpha: model.hues.one_grain.a,
    };
    let mut rcolor: RevertColor = RevertColor::default(&model);
    loop {
        screen.width = screen_width(); // start macroquad loop
        screen.height = screen_height(); // check screen size in case user has resized the window
        clear_background(model.hues.untouched); // clear background using color designated for untouched cells
        screen.draw(&model); // draw sandpile model
        screen.crosshairs(&model, &control); // add lakhesis cursor on top of model
                                             // check if a new sandpile is pending and if the left mouse button is pressed
        if control.add && is_mouse_button_pressed(MouseButton::Left) {
            model.active_cells += 1;
            model.drop_cells[model.active_cells - 1] = model.xy_to_idx(
                (screen.mx + screen.tlx).trunc() as usize,
                (screen.my + screen.tly).trunc() as usize,
            );
            control.paused = false;
            control.add = false;
            info.context = "<--Click here to hide the control panel".to_string();
        }
        // if !paused or spacebar pressed and a drop cell is active, drop sand grains and resolve unstable sandpiles
        if (!control.paused || control.increment) && model.active_cells > 0 {
            for _ in 0..model.interval {
                model.add_grain();
                if model.active_cells - 1 > model.ac {
                    model.ac += 1;
                } else {
                    model.ac = 0;
                };
            }
            if control.video > 0 {
                model.paint(
                    screen.tlx.trunc() as u32,
                    screen.tly.trunc() as u32,
                    screen.width.trunc() as u16,
                    screen.height.trunc() as u16,
                );
                control.video -= 1;
                info.context = format!(
                    "Recording video frame {} of {} - Press [ESC] to cancel",
                    VIDEO_FRAME_COUNT - control.video,
                    VIDEO_FRAME_COUNT
                );
            }
            control.increment = false;
        }
        // check if model is approaching undefined behaviour around 20M sand grains
        if model.total_grains >= MAX_ITERATIONS {
            control.paused = true;
        }
        // reset, if requested
        if control.reset {
            model = Model::default();
            screen = Screen::default(&model);
            info = Info::default();
            control = Control::default();
        }
        // change model colors, if requested
        if control.color {
            // info.context = "Click one of the five colors and move sliders to adjust color - changes are shown on main screen when box with new color is clicked".to_string();
            control.change_color(
                &mut model,
                &mut screen,
                &mut info,
                &mut rcolor,
                &mut csliders,
            );
        }
        // display an icon in top left corner that toggles panel visibilities
        if root_ui().button(None, "<>") {
            control.visible = !control.visible;
        }
        // update info and show control panel
        info.update(&screen);
        if control.paused {
            draw_rectangle_lines(0.0, 0.0, 21.0, 23.0, 3.0, ORANGE);
        }
        if control.visible {
            draw_rectangle(25.0, 0.0, screen.width, 19.0, BLACK);
            draw_text(&info.context, 27.0, 13.0, 16.0, YELLOW);
            control.draw_panel(&mut model, &mut info, &mut screen);
        }
        // has a key been pressed?
        control.check_keyboard(&mut model, &mut info);
        next_frame().await;
    }
}

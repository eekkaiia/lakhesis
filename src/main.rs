use lakhesis::Screen;
use lakhesis::{color_button, Csliders, Ncolor};
use lakhesis::{Control, Info};
use lakhesis::{Model, MAX_ITERATIONS};

use macroquad::color::colors::*;
use macroquad::color::Color;
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

    // color box ui
    let mut ncolor: Ncolor = Ncolor::default(&model);
    let mut csliders: Csliders = Csliders::default();

    // info box ui
    let mut info = Info::default();
    let mut control = Control::default();
    // start macroquad loop
    loop {
        screen.width = screen_width(); // in case user has resized the window
        screen.height = screen_height();

        clear_background(model.hues.untouched);

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
        if control.magnify {
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
            if control.add {
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

        // is left mouse button pressed
        if control.add && is_mouse_button_pressed(MouseButton::Left) {
            // if add - get mouse position to add new drop cell
            model.active_cells += 1;
            model.drop_cells[model.active_cells - 1] = model.xy_to_idx(
                (screen.mx + screen.tlx).trunc() as usize,
                (screen.my + screen.tly).trunc() as usize,
            );
            control.paused = false;
            control.add = false;
            info.context = "<--Click here to hide the control panel".to_string();
        }
        // if !paused or spacebar pressed and a drop cell is added, drop sand grains and resolve unstable sandpiles
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
        if model.total_grains >= MAX_ITERATIONS {
            control.paused = true;
        }
        if control.reset {
            model = Model::default();
            screen = Screen::default(&model);
            // color box ui
            ncolor = Ncolor::default(&model);
            csliders = Csliders::default();
            // info box ui
            info = Info::default();
            control = Control::default();
        }

        // TESTING colors dialog
        if control.color {
            color_button(
                &mut model,
                &mut control,
                &mut screen,
                &mut ncolor,
                &mut csliders,
            );
        }

        if root_ui().button(None, "<>") {
            control.visible = !control.visible;
        }

        // update info and show control panel
        info.update(&screen);
        if control.visible {
            draw_rectangle(18.0, 0.0, screen.width, 19.0, BLACK);
            draw_text(&info.context, 24.0, 13.0, 16.0, YELLOW);
            control.draw_panel(&mut model, &mut info, &mut screen);
        }
        // has a key been pressed?
        control.check_keyboard(&mut model, &mut info, &mut screen);
        next_frame().await;
    }
}

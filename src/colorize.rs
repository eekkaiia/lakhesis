use crate::{Control, Hues, Model, Screen};

use macroquad::color::Color;
use macroquad::math::*;
use macroquad::texture::*;

use macroquad::ui::{hash, root_ui, widgets};

#[derive(Clone, Copy, Debug, Default)]
pub struct Csliders {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Clone, Debug, Default)]
pub enum Selected {
    #[default]
    Untouched,
    Zero,
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Debug)]
pub struct Ncolor {
    pub selected: Selected,
    pub untouched: Color,
    pub zero_grains: Color,
    pub one_grain: Color,
    pub two_grains: Color,
    pub three_grains: Color,
    pub four_grains: Color, // not needed when collapse occurs at four grains
                            // left in for variation where collapse occurs at five grains
}
impl Ncolor {
    pub fn default(model: &Model) -> Self {
        let selected: Selected = Selected::Untouched;
        let untouched: Color = model.hues.untouched;
        let zero_grains: Color = model.hues.zero_grains;
        let one_grain: Color = model.hues.one_grain;
        let two_grains: Color = model.hues.two_grains;
        let three_grains: Color = model.hues.three_grains;
        let four_grains: Color = model.hues.four_grains;
        Self {
            selected,
            untouched,
            zero_grains,
            one_grain,
            two_grains,
            three_grains,
            four_grains,
        }
    }
}

pub fn color_button(
    model: &mut Model,
    control: &mut Control,
    screen: &mut Screen,
    ncolor: &mut Ncolor,
    csliders: &mut Csliders,
) {
    let untouched_color = Image::gen_image_color(90, 60, model.hues.untouched);
    let untouched_texture = Texture2D::from_image(&untouched_color);
    let zero_color = Image::gen_image_color(90, 60, model.hues.zero_grains);
    let zero_texture = Texture2D::from_image(&zero_color);
    let one_color = Image::gen_image_color(90, 60, model.hues.one_grain);
    let one_texture = Texture2D::from_image(&one_color);
    let two_color = Image::gen_image_color(90, 60, model.hues.two_grains);
    let two_texture = Texture2D::from_image(&two_color);
    let three_color = Image::gen_image_color(90, 60, model.hues.three_grains);
    let three_texture = Texture2D::from_image(&three_color);

    let slider_color: Color =
        Color::new(csliders.red, csliders.green, csliders.blue, csliders.alpha);
    let selected_color = Image::gen_image_color(90, 60, slider_color);
    let selected_texture = Texture2D::from_image(&selected_color);

    let w_width: f32 = 517.0;
    let w_height: f32 = 420.0;
    let w_tlx = (screen.width - w_width) / 2.0;
    let w_tly = (screen.height - w_height) / 2.0;
    widgets::Window::new(hash!(), vec2(w_tlx, w_tly), vec2(w_width, w_height))
        .label("Colors")
        .ui(&mut *root_ui(), |ui| {
            if widgets::Button::new(untouched_texture)
                .size(vec2(100., 70.))
                .ui(ui)
            {
                ncolor.selected = Selected::Untouched;
                csliders.red = model.hues.untouched.r;
                csliders.green = model.hues.untouched.g;
                csliders.blue = model.hues.untouched.b;
                csliders.alpha = model.hues.untouched.a;
            }
            ui.same_line(0.);
            if widgets::Button::new(zero_texture)
                .size(vec2(100., 70.))
                .ui(ui)
            {
                ncolor.selected = Selected::Zero;
                csliders.red = model.hues.zero_grains.r;
                csliders.green = model.hues.zero_grains.g;
                csliders.blue = model.hues.zero_grains.b;
                csliders.alpha = model.hues.zero_grains.a;
            }
            ui.same_line(0.);
            if widgets::Button::new(one_texture)
                .size(vec2(100., 70.))
                .ui(ui)
            {
                ncolor.selected = Selected::One;
                csliders.red = model.hues.one_grain.r;
                csliders.green = model.hues.one_grain.g;
                csliders.blue = model.hues.one_grain.b;
                csliders.alpha = model.hues.one_grain.a;
            }
            ui.same_line(0.);
            if widgets::Button::new(two_texture)
                .size(vec2(100., 70.))
                .ui(ui)
            {
                ncolor.selected = Selected::Two;
                csliders.red = model.hues.two_grains.r;
                csliders.green = model.hues.two_grains.g;
                csliders.blue = model.hues.two_grains.b;
                csliders.alpha = model.hues.two_grains.a;
            }
            ui.same_line(0.);
            if widgets::Button::new(three_texture)
                .size(vec2(100., 70.))
                .ui(ui)
            {
                ncolor.selected = Selected::Three;
                csliders.red = model.hues.three_grains.r;
                csliders.green = model.hues.three_grains.g;
                csliders.blue = model.hues.three_grains.b;
                csliders.alpha = model.hues.three_grains.a;
            }
            ui.label(
                None,
                "    Null           Zero           One           Two           Three",
            );

            if widgets::Button::new(selected_texture)
                .size(vec2(w_width - 5.0, 70.0))
                .ui(ui)
            {
                match ncolor.selected {
                    Selected::Untouched => {
                        ncolor.untouched =
                            Color::new(csliders.red, csliders.green, csliders.blue, csliders.alpha);
                        model.hues.untouched = ncolor.untouched;
                    }
                    Selected::Zero => {
                        ncolor.zero_grains =
                            Color::new(csliders.red, csliders.green, csliders.blue, csliders.alpha);
                        model.hues.zero_grains = ncolor.zero_grains;
                    }
                    Selected::One => {
                        ncolor.one_grain =
                            Color::new(csliders.red, csliders.green, csliders.blue, csliders.alpha);
                        model.hues.one_grain = ncolor.one_grain;
                    }
                    Selected::Two => {
                        ncolor.two_grains =
                            Color::new(csliders.red, csliders.green, csliders.blue, csliders.alpha);
                        model.hues.two_grains = ncolor.two_grains;
                    }
                    Selected::Three => {
                        ncolor.three_grains =
                            Color::new(csliders.red, csliders.green, csliders.blue, csliders.alpha);
                        model.hues.three_grains = ncolor.three_grains;
                    }
                    _ => (),
                }
            }
            ui.label(
                None,
                &format!("       {:?}-grain color selected", &ncolor.selected),
            );
            ui.slider(hash!(), "  Red", 0f32..1f32, &mut csliders.red);
            ui.slider(hash!(), "  Green", 0f32..1f32, &mut csliders.green);
            ui.slider(hash!(), "  Blue", 0f32..1f32, &mut csliders.blue);
            ui.slider(hash!(), "  Alpha", 0f32..1f32, &mut csliders.alpha);
            ui.label(None, "       Alpha of 0.0 is completely transparent");
            ui.label(
                None,
                "After adjusting color click the new colored box to set that color",
            );
            if widgets::Button::new("Exit").size(vec2(100., 70.)).ui(ui) {
                control.color = false;
                control.paused = false;
            }
            ui.same_line(0.);
            if widgets::Button::new("Random").size(vec2(100., 70.)).ui(ui) {
                model.random_colors();
                control.color = false;
                control.paused = false;
            }
            ui.same_line(0.);
            if widgets::Button::new("Default").size(vec2(100., 70.)).ui(ui) {
                model.hues = Hues {
                    untouched: Color::new(0.00, 0.00, 0.00, 0.00),
                    zero_grains: Color::new(0.00, 0.47, 0.95, 1.00),
                    one_grain: Color::new(0.00, 0.89, 0.19, 1.00),
                    two_grains: Color::new(0.99, 0.98, 0.00, 1.00),
                    three_grains: Color::new(0.00, 0.00, 0.00, 0.00),
                    four_grains: Color::new(0.90, 0.16, 0.22, 1.00),
                };
                control.color = false;
                control.paused = false;
            }
        });
}

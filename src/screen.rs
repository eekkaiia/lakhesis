use crate::Model;

use macroquad::color::colors::*;
use macroquad::color::Color;
use macroquad::input::*;
use macroquad::math::*;
use macroquad::shapes::*;
use macroquad::window::*;

#[derive(Clone, Debug, Default)]
pub struct Screen {
    pub tlx: f32,
    pub tly: f32,
    pub width: f32,
    pub height: f32,
    pub mx: f32,
    pub my: f32,
    pub background: bool,
}
impl Screen {
    pub fn default(model: &Model) -> Self {
        let width = screen_width();
        let height = screen_height();
        let tlx: f32 = (model.width as f32 - width).abs() / 2.0; // initial top left corner of screen
        let tly: f32 = (model.height as f32 - height).abs() / 2.0; // centered on model
        let (mx, my) = mouse_position();
        Self {
            tlx,
            tly,
            width,
            height,
            mx,
            my,
            background: true,
        }
    }
    /// draw() maps visible portion of model to screen
    pub fn draw(&self, model: &Model) {
        // redrawing entire screen reduces fps - use model find_extent() to only update active part of model
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
                    let mut pixel_color: Color = model.hues.untouched;
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
                let mut pixel_color: Color = bg; // background color instead of untouched so unmagnified image is blocked
                match model.cells[idx].grains {
                    0 => {
                        if model.cells[idx].borged {
                            pixel_color = model.hues.zero_grains;
                        }
                    }
                    1 => pixel_color = model.hues.one_grain,
                    2 => pixel_color = model.hues.two_grains,
                    3 => pixel_color = model.hues.three_grains,
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

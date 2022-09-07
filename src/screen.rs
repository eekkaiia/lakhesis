use crate::{Control, Model};

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
        }
    }
    /// draw() maps visible portion of model to screen
    pub fn draw(&self, model: &Model) {
        // draw a dot at the center of the model if visible
        let (navel_x, navel_y) = model.calc_center_xy();
        if (navel_x as f32) >= self.tlx
            && (navel_x as f32) <= (self.tlx + self.width)
            && (navel_y as f32) >= self.tly
            && (navel_y as f32) <= (self.tly + self.height)
        {
            let center_x: f32 = navel_x as f32 - self.tlx;
            let center_y: f32 = navel_y as f32 - self.tly;
            draw_rectangle(center_x, center_y, 2.0, 2.0, GRAY);
        }
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
        let mut bg: Color = model.hues.untouched;
        if bg.a <= 0.5 {
            bg.a = 1.0;
        };
        draw_rectangle(
            // draw magnified area background
            self.width - 151.0,
            self.height - 151.0,
            128.0,
            128.0,
            bg,
        );
        let curs: Color = WHITE;
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
                    1 => {
                        pixel_color = model.hues.one_grain;
                    }
                    2 => {
                        pixel_color = model.hues.two_grains;
                    }
                    3 => {
                        pixel_color = model.hues.three_grains;
                    }
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
    /// crosshairs() draws the lakhesis cursor
    pub fn crosshairs(&mut self, model: &Model, control: &Control) {
        // get current mouse position and limit extent if magnify is on
        (self.mx, self.my) = mouse_position();
        if control.magnify {
            if self.mx < 16.0 {
                self.mx = 16.0;
            };
            if self.mx > self.width - 16.0 {
                self.mx = self.width - 16.0;
            };
            if self.my < 16.0 {
                self.my = 16.0;
            };
            if self.my > self.height - 16.0 {
                self.my = self.height - 16.0;
            };
            self.magnify_box(model);
        }
        // draw crosshairs - thick ones if waiting to add active cell
        if self.mx >= 0.0 && self.mx < self.width && self.my >= 0.0 && self.my < self.height {
            let curs: Color = WHITE;
            if control.add {
                draw_line(self.mx - 10.0, self.my, self.mx + 10.0, self.my, 3.0, curs);
                draw_line(self.mx, self.my - 10.0, self.mx, self.my + 10.0, 3.0, curs);
            } else {
                draw_line(self.mx - 10.0, self.my, self.mx + 10.0, self.my, 1.0, curs);
                draw_line(self.mx, self.my - 10.0, self.mx, self.my + 10.0, 1.0, curs);
            }
        }
    }
}

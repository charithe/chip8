use crate::emulator::display;
use crate::emulator::display::Pixels;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    symbols,
    widgets::{
        canvas::{Painter, Shape},
        Block, Widget,
    },
};

pub struct Screen<'a> {
    block: Option<Block<'a>>,
    pixels: Option<&'a Pixels>,
}

impl<'a> Default for Screen<'a> {
    fn default() -> Screen<'a> {
        Screen {
            block: None,
            pixels: None,
        }
    }
}

impl<'a> Screen<'a> {
    pub fn block(mut self, block: Block<'a>) -> Screen<'a> {
        self.block = Some(block);
        self
    }

    pub fn pixels(mut self, pixels: &'a Pixels) -> Screen<'a> {
        self.pixels = Some(pixels);
        self
    }
}

impl<'a> Widget for Screen<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let screen_area = match self.block {
            Some(ref mut b) => {
                b.render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        buf.set_background(screen_area, Color::Green);

        if let Some(pixels) = self.pixels {
            pixels.iter().for_each(|p| {
                buf.get_mut(
                    screen_area.left() + p.x as u16,
                    screen_area.top() + p.y as u16,
                )
                .set_symbol(symbols::block::FULL)
                .set_fg(Color::Black);
            });
        }
    }
}

impl<'a> Shape for Screen<'a> {
    fn draw(&self, painter: &mut Painter) {
        if let Some(pixels) = self.pixels {
            pixels.iter().for_each(|p| {
                if let Some((x, y)) =
                    painter.get_point(p.x as f64, display::HEIGHT as f64 - p.y as f64)
                {
                    painter.paint(x, y, Color::Red);
                }
            });
        }
    }
}

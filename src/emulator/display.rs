use std::fmt;

pub const WIDTH: u8 = 64;
pub const HEIGHT: u8 = 32;

const SPRITE_WIDTH: u8 = 8;
const TOTAL_PIXELS: usize = 64 * 32;

pub type Pixels = Vec<Pixel>;

pub struct Pixel {
    pub x: u8,
    pub y: u8,
    pub value: u8,
}

pub struct Sprite {
    x: u8,
    y: u8,
    data: Vec<u8>,
}

impl Sprite {
    pub fn new(x: u8, y: u8, data: Vec<u8>) -> Self {
        Sprite { x, y, data }
    }
}

pub struct Screen {
    pixels: [u8; TOTAL_PIXELS],
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            pixels: [0u8; TOTAL_PIXELS],
        }
    }
}

impl Screen {
    pub fn clear(&mut self) {
        self.pixels.iter_mut().for_each(|p| *p = 0u8);
    }

    pub fn draw(&mut self, sprite: Sprite) -> Option<u8> {
        if sprite.x >= WIDTH || sprite.y >= HEIGHT {
            return None;
        }

        let mut vf = 0;
        let width = if sprite.x >= (WIDTH - SPRITE_WIDTH) {
            WIDTH - sprite.x
        } else {
            SPRITE_WIDTH
        };

        for (h, v) in sprite.data.iter().enumerate() {
            for w in 0..width {
                let sprite_pixel = v & (0x80 >> w);
                if sprite_pixel != 0 {
                    let index = Screen::calc_index(sprite.x, sprite.y, w, h as u8);
                    if index >= self.pixels.len() {
                        return Some(vf);
                    }

                    if self.pixels[index] == 1 {
                        vf = 1;
                    }
                    self.pixels[index] ^= 1;
                }
            }
        }

        Some(vf)
    }

    fn calc_index(base_x: u8, base_y: u8, x: u8, y: u8) -> usize {
        let y_offset = (base_y + y) as u64 * WIDTH as u64;
        let x_offset = (base_x + x) as u64;
        (x_offset + y_offset) as usize
    }

    pub fn pixels(&self) -> Pixels {
        self.pixels
            .iter()
            .enumerate()
            .filter_map(|(i, value)| {
                if *value == 0u8 {
                    return None;
                }

                let x = (i as u64 % WIDTH as u64) as u8;
                let y = (i as u64 / WIDTH as u64) as u8;
                Some(Pixel {
                    x,
                    y,
                    value: *value,
                })
            })
            .collect()
    }
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, v) in self.pixels.iter().enumerate() {
            if i % WIDTH as usize == 0 {
                write!(f, "\n")?;
            }
            let symbol = if *v == 0 { "·" } else { "█" };
            write!(f, "{}", symbol)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_draw_sprite() {
        let mut scr = Screen::default();
        let result = scr.draw(Sprite::new(10, 10, vec![0xF0, 0x90, 0xF0, 0x10, 0xF0]));
        assert_eq!(result, Some(0));
        println!("{}", scr);

        let result = scr.draw(Sprite::new(10, 10, vec![0xF0, 0x90, 0xF0, 0x10, 0xF0]));
        assert_eq!(result, Some(1));
    }
}

//! CPU-side software rasterizing primitives that draw directly into a framebuffer's pixel slice.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Black,
    White,
    Red,
    Green,
    Blue,
}

impl Color {
    // 0x00RRGGBB
    pub const fn to_u32(self) -> u32 {
        match self {
            Color::Black => 0x0000_0000,
            Color::White => 0x00FF_FFFF,
            Color::Red   => 0x00FF_0000,
            Color::Green => 0x0000_FF00,
            Color::Blue  => 0x0000_00FF,
        }
    }
}

pub struct Rectangle {
    pub width:  u32,
    pub height: u32,
    pub color:  Color,
}

impl Rectangle {
    pub fn draw(
        &self,
        framebuffer: &mut [u32],
        pos_x: i32,
        pos_y: i32,
        pitch_pixels: usize,
    ) -> Result<(), &'static str> {
        let max_pos_x: i32 = pitch_pixels as i32;
        let max_pos_y: i32 = (framebuffer.len() / pitch_pixels) as i32;

        if pos_x > max_pos_x || pos_x + self.width as i32 > max_pos_x ||
            pos_y > max_pos_y || pos_y + self.height as i32 > max_pos_y {
            return Err("PosX or PosY overflows from the buffer or the rectangle is drawn out of bounds from the framebuffer");
        }

        for y in 0..self.height {
            for x in 0..self.width {
                framebuffer[y as usize * pitch_pixels + x as usize] = self.color.to_u32();
            }
        }

        Ok(())
    }
}

pub struct Circle {
    pub radius: u32,
    pub color:  Color,
}

impl Circle {
    pub fn draw(
        &self,
        framebuffer: &mut [u32],
        pos_x: i32,
        pos_y: i32,
        pitch_pixels: usize,
    ) -> Result<(), &'static str> {
        let max_pos_x: i32 = pitch_pixels as i32;
        let max_pos_y: i32 = (framebuffer.len() / pitch_pixels) as i32;
        let r = self.radius as i32;

        // Reject if the circle's bounding box leaves the framebuffer.
        if pos_x - r < 0 || pos_x + r >= max_pos_x || pos_y - r < 0 || pos_y + r >= max_pos_y {
            return Err("Circle out of the bounds of the framebuffer");
        }

        let r2 = r * r;

        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r2 {
                    let x = (pos_x + dx) as usize;
                    let y = (pos_y + dy) as usize;
                    framebuffer[y * pitch_pixels + x] = self.color.to_u32();
                }
            }
        }

        Ok(())
    }
}

pub struct Rasterizer {}

impl Rasterizer {
    pub fn clear_screen(buffer: &mut [u32], height: u32, width: u32, pitch_pixels: usize) {
        for y in 0..height {
            for x in 0..width {
                buffer[y as usize * pitch_pixels + x as usize] = Color::Black.to_u32();
            }
        }
    }
}

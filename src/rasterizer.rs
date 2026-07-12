//! CPU-side software rasterizing primitives that draw directly into a framebuffer's pixel slice.

pub struct Rectangle {
    pub width:  u32,
    pub height: u32,
    pub color:  u32,
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
                framebuffer[y as usize * pitch_pixels + x as usize] = self.color;
            }
        }

        Ok(())
    }
}

pub struct Circle {
    radius: u32
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

        if pos_x > max_pos_x || pos_x + self.radius as i32 > max_pos_x || pos_x - self.radius as i32 >= 0;
            pos_y > max_pos_y || pos_y + self.radius as i32 > max_pos_y || pos_y - self.radius as i32 >= 0 {
            return Err("Circle out of the bound of the framebuffer");
        }

        for y in 0..self.height {
            for x in 0..self.width {
                framebuffer[y as usize * pitch_pixels + x as usize] = self.color;
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
                buffer[y as usize * pitch_pixels + x as usize] = 0x0000_0000;
            }
        }
    }
}

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

pub struct Triangle {
    /// Vertices, relative to the `pos_x` / `pos_y` passed to `draw`.
    pub p1: [i32; 2],
    pub p2: [i32; 2],
    pub p3: [i32; 2],
    pub color: Color,
}

impl Triangle {
    // Signed area of the trianlge used to know if a given point is inside a triangle or not
    fn edge(a: [i32; 2], b: [i32; 2], p: [i32; 2]) -> i32 {
        (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
    }

    pub fn draw(
        &self,
        framebuffer: &mut [u32],
        pos_x: i32,
        pos_y: i32,
        pitch_pixels: usize,
    ) -> Result<(), &'static str> {
        let max_pos_x: i32 = pitch_pixels as i32;
        let max_pos_y: i32 = (framebuffer.len() / pitch_pixels) as i32;

        // vertices
        let a = [self.p1[0] + pos_x, self.p1[1] + pos_y];
        let b = [self.p2[0] + pos_x, self.p2[1] + pos_y];
        let c = [self.p3[0] + pos_x, self.p3[1] + pos_y];

        // reject if a vertice is out of bound
        for v in [a, b, c] {
            if v[0] < 0 || v[0] >= max_pos_x || v[1] < 0 || v[1] >= max_pos_y {
                return Err("Triangle out of the bounds of the framebuffer");
            }
        }

        // Degenerate trianlges that have no area
        let area = Self::edge(a, b, c);
        if area == 0 {
            return Err("Degenerate triangle (vertices are collinear)");
        }

        // Bounding box of the triangle.
        let min_x = a[0].min(b[0]).min(c[0]);
        let max_x = a[0].max(b[0]).max(c[0]);
        let min_y = a[1].min(b[1]).min(c[1]);
        let max_y = a[1].max(b[1]).max(c[1]);

        // A pixel is inside when all three edge functions share the triangle's windind sign
        let sign = area.signum();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = [x, y];
                let w0 = Self::edge(b, c, p) * sign;
                let w1 = Self::edge(c, a, p) * sign;
                let w2 = Self::edge(a, b, p) * sign;
                if w0 >= 0 && w1 >= 0 && w2 >= 0 {
                    framebuffer[y as usize * pitch_pixels + x as usize] = self.color.to_u32();
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

// framebuffer.rs
use raylib::prelude::*;

pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
    depth_buffer: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        let background_color = Color::BLACK; // Un color por defecto
        let color_buffer = Image::gen_image_color(width, height, background_color);
        let depth_buffer = vec![f32::INFINITY; (width * height) as usize];
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
            depth_buffer,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer.clear_background(self.background_color);
        self.depth_buffer.fill(f32::INFINITY);
    }

    pub fn set_pixel(&mut self, x: i32, y: i32) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.color_buffer.draw_pixel(x, y, self.current_color);
        }
    }
    
    pub fn point(&mut self, x: i32, y: i32, color: Vector3, depth: f32) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            let index = (y * self.width + x) as usize;

            if depth < self.depth_buffer[index] {
                self.depth_buffer[index] = depth;
                let pixel_color = Color::new(
                    (color.x.clamp(0.0, 1.0) * 255.0) as u8,
                    (color.y.clamp(0.0, 1.0) * 255.0) as u8,
                    (color.z.clamp(0.0, 1.0) * 255.0) as u8,
                    255,
                );
                self.color_buffer.draw_pixel(x, y, pixel_color);
            }
        }
    }
    
    pub fn get_pixel_color(&mut self, x: i32, y: i32) -> Option<Color> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some(self.color_buffer.get_color(x, y))
        } else {
            None
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn swap_buffers(&self, d: &mut RaylibHandle, thread: &RaylibThread) {
        if let Ok(texture) = d.load_texture_from_image(thread, &self.color_buffer) {
            let mut d = d.begin_drawing(thread);
            d.clear_background(self.background_color);
            d.draw_texture(&texture, 0, 0, Color::WHITE);
        }
    }

    pub fn draw_line(&mut self, p0: Vector3, p1: Vector3, color: Vector3) {
        let mut x0 = p0.x as i32;
        let mut y0 = p0.y as i32;
        let x1 = p1.x as i32;
        let y1 = p1.y as i32;

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            self.point(x0, y0, color, p0.z); // Use p0.z for depth, or interpolate
            if x0 == x1 && y0 == y1 { break; }
            let e2 = 2 * err;
            if e2 > -dy { err -= dy; x0 += sx; }
            if e2 < dx { err += dx; y0 += sy; }
        }
    }
}
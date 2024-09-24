#![allow(dead_code)]

use crate::WIDTH;
use crate::HEIGHT;
use crate::SCALE;

use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct Screen<'a> {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec<usize>>,
    pub canvas: Option<&'a mut Canvas<Window>>,
    pub update_screen: bool,
}

impl<'a> Screen<'a> {

    pub fn new(canvas: Option<&'a mut Canvas<Window>>) -> Screen {
        let pixels: Vec<Vec<usize>> = vec![vec![0; WIDTH as usize]; HEIGHT as usize];
        Screen { width: WIDTH, height: HEIGHT, pixels, canvas, update_screen: false }
    }

    pub fn setup(&mut self) {
        if let Some(canvas) = &mut self.canvas {
            canvas.set_draw_color(Color::RGB(0, 255, 255));
            canvas.clear();
            canvas.present();
        }
    }

    pub fn get_scaled_width(&self) -> u32 {
        (self.width * SCALE) as u32
    }

    pub fn get_scaled_height(&self) -> u32 {
        (self.height * SCALE) as u32
    }

    pub fn get_pixel(&self, x: usize, y: usize) ->  usize {
        self.pixels[y][x]
    }

    pub fn update_pixel(&mut self, x: usize, y: usize) {
        self.pixels[y as usize][x as usize] ^= 1;
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, v: usize) {
        self.pixels[y as usize][x as usize] = v;
    }

    /// Updates every pixel to a random value.
    pub fn four_corners(&mut self) {
        self.set_pixel(0, 0, 1);
        self.set_pixel(WIDTH - 1, HEIGHT - 1, 1);
        self.set_pixel(WIDTH - 1, 0, 1);
        self.set_pixel(0, HEIGHT - 1, 1);
    }

    pub fn clear(&mut self) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                self.set_pixel(x, y, 0);
            }
        }

        // The rectangle we will use to fill with our color value.
        let rect = Rect::new(
            0,
            0,
            (WIDTH * SCALE) as u32,
            (HEIGHT * SCALE) as u32,
        );

        if self.canvas.is_none() {
            return
        }

        if let Some(canvas) = &mut self.canvas {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.fill_rect(rect).unwrap();
            canvas.present();
        }
    }

    pub fn draw(&mut self) {

        if self.canvas.is_none() {
            return
        }

        if let Some(canvas) = &mut self.canvas {

            for x in 0..self.width {
                for y in 0..self.height {
    
                    // Value to determine if we are drawing a black or white pixel.
                    let v = self.pixels[y as usize][x as usize];
                    
                    // The rectangle we will use to fill with our color value.
                    let rect = Rect::new(
                        (x * SCALE) as i32,
                        (y * SCALE) as i32,
                        SCALE as u32,
                        SCALE as u32,
                    );
    
                    if v <= 0 {
                        canvas.set_draw_color(Color::RGB(0, 0, 0));
                    } else {
                        canvas.set_draw_color(Color::RGB(255, 255, 255));
                    }
    
                    canvas.fill_rect(rect).unwrap();
                }
            }
    
            canvas.present();
        }
    }
}
extern crate sdl2;

mod screen;
mod memory;
mod collections;

use screen::Screen;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use memory::memory::Memory;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCALE: usize = 16;

pub fn main() {
    let sdl_context: sdl2::Sdl = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", (WIDTH * SCALE) as u32, (HEIGHT * SCALE) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut screen: Screen = Screen::new(Some(&mut canvas));

    screen.setup();

    let mut memory: Memory = Memory::new(&mut screen);

    load_file(&mut memory);

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },

                _ => {}
            }
        }

        memory.fetch();
        memory.execute();

        if memory.screen.update_screen {
            memory.screen.draw();
            memory.screen.update_screen = false;
        }
        
        ::std::thread::sleep(Duration::from_millis(1));
    }

}

fn load_file(memory: &mut Memory) {
    // Read in file
    let mut file = File::open("roms/test_opcode.ch8").expect("File not found");

    let mut buffer: Vec<u8> = Vec::new();

    file.read_to_end(&mut buffer).expect("Failed to read the file");

    memory.load(buffer);
}
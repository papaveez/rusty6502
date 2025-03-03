use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;
// use std::env;
use std::process;

mod args;
mod bus;
mod cpu;

use args::EmuArgs;
use bus::Bus;
use clap::Parser;
use cpu::CPU;

#[derive(Default)]
pub struct Queue {
    tail: usize,
    data: [u8; 32],
}

impl Queue {
    fn shift(&mut self) {
        if self.tail == 0 {
            return;
        }

        for i in 0..(self.data.len() - 1) {
            self.data[i] = self.data[i + 1];
        }

        self.tail -= 1;
    }

    fn pop(&mut self) -> u8 {
        let v = self.data[0];
        self.shift();
        v
    }

    fn push(&mut self, d: u8) {
        if self.tail >= (self.data.len() - 1) {
            self.shift();
        }

        self.data[self.tail] = d;
        self.tail += 1;
    }
}

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

fn update_input(q: &mut Queue, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        let w = match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => 0x77,

            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => 0x73,
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => 0x61,
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => 0x64,
            _ => 0x00,
        };

        if w > 0 {
            q.push(w);
        }
    }
}

fn handle_user_input(cpu: &mut CPU, q: &mut Queue) {
    let w = q.pop();
    if w > 0 {
        cpu.bus.write(0xFF, w);
    };
}

fn read_screen_state(cpu: &mut CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_idx = cpu.bus.read(i as u16);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}

fn main() {
    // let args: Vec<String> = env::args().collect();
    let args = EmuArgs::parse();

    let path = &args.file_name;

    println!("Initialising CPU");
    let mut c = CPU::new(Bus { memory: [0; 65535] });
    // let path = "roms/snake.nes";
    match c.load_rom_file(path) {
        Ok(()) => println!("Loaded {}", path),
        _ => {
            println!("IOERROR: File not found");
            process::exit(1);
        }
    };

    println!("Initialising SDL2");
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("6502emu", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(10.0, 10.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
        .unwrap();

    let mut screen_state = [0_u8; 32 * 3 * 32];
    let mut rng = rand::thread_rng();

    let mut key_queue = Queue::default();

    println!("Running main loop");
    c.run(move |cpu| {
        update_input(&mut key_queue, &mut event_pump);
        handle_user_input(cpu, &mut key_queue);
        cpu.bus.write(0xfe, rng.gen_range(1, 16));

        if read_screen_state(cpu, &mut screen_state) {
            texture.update(None, &screen_state, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        ::std::thread::sleep(std::time::Duration::new(0, 70_000));
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eztest() {
        let mut c = CPU::new(Bus { memory: [0; 65535] });
        // let mut rng = rand::thread_rng();

        let ezcode = vec![
            0xa9, 0x10, // LDA #$10     -> A = #$10
            0x85, 0x20, // STA $20      -> $20 = #$10
            0xa9, 0x01, // LDA #$1      -> A = #$1
            0x65, 0x20, // ADC $20      -> A = #$11
            0x85, 0x21, // STA $21      -> $21=#$11
            0xe6, 0x21, // INC $21      -> $21=#$12
            0xa4, 0x21, // LDY $21      -> Y=#$12
            0xc8, // INY          -> Y=#$13
            0x00, // BRK
        ];

        c.load(ezcode);
        c.run(move |_cpu| {});
        assert_eq!(c.bus.read(0x20), 0x10);
        assert_eq!(c.bus.read(0x21), 0x12);
        assert_eq!(c.reg.a, 0x11);
        assert_eq!(c.reg.y, 0x13);
    }

    fn run_testrom(romname: &str) {
        let mut c = CPU::new(Bus { memory: [0; 65535] });
        let mut file = String::from("./test_roms/");
        file.push_str(romname);

        match c.load_rom_file(&file) {
            Ok(()) => (),
            Err(_) => {
                panic!("IOERROR: File not found");
            }
        }

        c.run(move |_cpu| {});
        assert_eq!(c.bus.read(0x6000), 0)
    }

    #[test]
    fn implied() {
        run_testrom("01-implied.nes");
    }

    #[test]
    fn immediate() {
        run_testrom("02-immediate.nes");
    }

    #[test]
    fn zero_page() {
        run_testrom("03-zero_page.nes");
    }

    #[test]
    fn zp_xy() {
        run_testrom("04-zp_xy.nes");
    }
}

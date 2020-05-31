use crate::emulator;
use crate::emulator::common::{Error, Result};
use crossbeam_channel;
use std::time::Duration;

const CLOCK_SPEED_HZ: u32 = 60;

pub fn start(emu: &mut emulator::Emulator) -> Result<()> {
    let ticker = crossbeam_channel::tick(Duration::from_secs(1) / CLOCK_SPEED_HZ);
    for _tick in ticker.iter() {
        match emu.step() {
            Ok(Some(emulator::Step::Draw(pixels))) => draw(pixels),
            Ok(_) => {}
            Err(err) => {
                eprintln!("{}", err);
                return Err(Error::Unexpected(Box::new(err)));
            }
        }
    }

    Ok(())
}

fn draw(pixels: emulator::display::Pixels) {
    let mut screen = [['·'; emulator::display::WIDTH as usize]; emulator::display::HEIGHT as usize];
    pixels.iter().for_each(|p| {
        screen[p.y as usize][p.x as usize] = '█';
    });

    for row in screen.iter() {
        for col in row.iter() {
            print!("{}", col);
        }
        print!("\n");
    }

    println!("");
}

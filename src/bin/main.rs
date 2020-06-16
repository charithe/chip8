use chip8::emulator;
use pretty_env_logger;
use std::env;
use std::fs::File;
use std::io;

fn main() -> emulator::Result<()> {
    pretty_env_logger::init();

    if let Some(rom_file) = env::args().skip(1).next() {
        start_emu(rom_file)?
    }

    Ok(())
}

fn start_emu(rom: String) -> emulator::Result<()> {
    let input = File::open(rom)?;
    let buffered = io::BufReader::new(input);
    let mut emu = emulator::Emulator::new(buffered)?;

    emulator::ui::gui::start_loop(&mut emu)
    //emulator::debugger::start(&mut emu)
}

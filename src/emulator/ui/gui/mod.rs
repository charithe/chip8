use crate::emulator;
use crate::emulator::common::Result;
use piston_window::*;

const TICK_RATE: u64 = 15;

pub fn start_loop(emu: &mut emulator::Emulator) -> Result<()> {
    let mut window: PistonWindow = WindowSettings::new("Chip-8", [640, 320])
        .exit_on_esc(true)
        .samples(4)
        .resizable(false)
        .build()
        .unwrap();

    window.set_ups(TICK_RATE);
    window.set_ups_reset(0);
    window.set_max_fps(60);
    window.set_swap_buffers(true);
    window.set_lazy(false);

    while let Some(e) = window.next() {
        to_input(&e).iter().for_each(|i| emu.send_input(*i));

        let step = emu.step();
        match step {
            Ok(Some(emulator::Step::Exit)) => {
                return Ok(());
            }
            Ok(Some(emulator::Step::Draw(pixels))) => {
                window.draw_2d(&e, |c, g, _| {
                    clear([0.0, 0.0, 0.0, 0.0], g);
                    pixels.iter().for_each(|p| {
                        Rectangle::new([0.0, 1.0, 0.0, 1.0]).draw(
                            [p.x as f64 * 10.0, p.y as f64 * 10.0, 10.0, 10.0],
                            &c.draw_state,
                            c.transform,
                            g,
                        );
                    });
                });
            }
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        };
    }

    Ok(())
}

fn to_input(evt: &Event) -> Option<emulator::Input> {
    if let Some(Button::Keyboard(key)) = evt.press_args() {
        match key {
            Key::D1 => Some(emulator::Input::Key1),
            Key::D2 => Some(emulator::Input::Key2),
            Key::D3 => Some(emulator::Input::Key3),
            Key::D4 => Some(emulator::Input::KeyC),
            Key::Q => Some(emulator::Input::Key4),
            Key::W => Some(emulator::Input::Key5),
            Key::E => Some(emulator::Input::Key6),
            Key::R => Some(emulator::Input::KeyD),
            Key::A => Some(emulator::Input::Key7),
            Key::S => Some(emulator::Input::Key8),
            Key::D => Some(emulator::Input::Key9),
            Key::F => Some(emulator::Input::KeyE),
            Key::Z => Some(emulator::Input::KeyA),
            Key::X => Some(emulator::Input::Key0),
            Key::C => Some(emulator::Input::KeyB),
            Key::V => Some(emulator::Input::KeyF),
            _ => None,
        }
    } else {
        None
    }
}

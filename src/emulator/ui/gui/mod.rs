use crate::emulator;
use crate::emulator::common::Result;
use crate::emulator::display;
use piston_window::*;

const CLOCK_SPEED: f64 = 60.0f64;

pub fn start_loop(emu: &mut emulator::Emulator) -> Result<()> {
    let mut window: PistonWindow = WindowSettings::new("Chip-8", [640, 320])
        .exit_on_esc(true)
        .vsync(true)
        .resizable(false)
        .build()
        .unwrap();

    window.set_ups(CLOCK_SPEED as u64);
    window.set_max_fps(CLOCK_SPEED as u64);
    window.set_swap_buffers(true);
    window.set_lazy(false);

    let mut pixels = display::Pixels::default();

    while let Some(e) = window.next() {
        match e {
            Event::Loop(Loop::Update(args)) => {
                let num_steps = (args.dt * CLOCK_SPEED).round() as usize;
                for _i in 0..num_steps {
                    match emu.step() {
                        Ok(Some(emulator::Step::Exit)) => {
                            return Ok(());
                        }
                        Ok(Some(emulator::Step::Draw(p))) => {
                            pixels = p.clone();
                        }
                        Ok(_) => {}
                        Err(err) => {
                            return Err(err);
                        }
                    };
                }
            }
            Event::Loop(Loop::Render(_)) => {
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
            Event::Input(Input::Button(args), _) => {
                if let Button::Keyboard(key) = args.button {
                    let input = match key {
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
                    };

                    input.iter().for_each(|i| match args.state {
                        ButtonState::Press => emu.key_press(*i),
                        ButtonState::Release => emu.key_release(*i),
                    });
                }
            }
            _ => {}
        }
    }

    Ok(())
}

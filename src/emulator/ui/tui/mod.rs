use crate::emulator;
use crate::emulator::common::{Error, Result};
use crate::emulator::{display, Input};
use log::{debug, error};
use std::{io, thread, time::Duration};
use termion::{event::Key, input::TermRead, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::Rect,
    widgets::{Block, Borders},
    Terminal,
};

use crossbeam_channel;
use crossbeam_channel::select;

mod widgets;

const CLOCK_SPEED_HZ: u32 = 60;

pub fn start_loop(emu: &mut emulator::Emulator) -> Result<()> {
    let (input_tx, input_rx) = crossbeam_channel::bounded(1);
    let ticker = crossbeam_channel::tick(Duration::from_secs(1) / CLOCK_SPEED_HZ);

    start_input_loop(input_tx);

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut pixels: Option<display::Pixels> = None;

    loop {
        select! {
            recv(input_rx) -> key => {
                match key {
                    Ok(Input::Quit) => return Ok(()),
                    Ok(k) => emu.send_input(k),
                    Err(err) => return Err(Error::Unexpected(Box::new(err))),
                }
            },
            recv(ticker) -> _tick => {
                let step = emu.step();
                match step {
                    Ok(Some(emulator::Step::Exit)) =>  {
                        return Ok(());
                    },
                    Ok(Some(emulator::Step::Draw(p))) =>  {
                        pixels = Some(p.to_vec());
                    },
                    Ok(_) => {},
                    Err(err) => {
                        return Err(err);
                    },
                };

                draw_screen(&mut terminal, &pixels);
            },
        }
    }
}

fn draw_screen<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    pixels: &Option<display::Pixels>,
) {
    let scr = if let Some(ref p) = pixels {
        widgets::Screen::default()
            .block(Block::default().borders(Borders::ALL))
            .pixels(&p)
    } else {
        widgets::Screen::default().block(Block::default().borders(Borders::ALL))
    };

    terminal
        .draw(|mut f| {
            let size = f.size();
            let padded_width = display::WIDTH as u16 + 10;
            let padded_height = display::HEIGHT as u16 + 10;
            let area = Rect::new(
                (size.width / 2) - (padded_width / 2),
                (size.height / 2) - (padded_height / 2),
                padded_width,
                padded_height,
            );
            f.render_widget(scr, area);
        })
        .unwrap();
}

fn start_input_loop(
    input_tx: crossbeam_channel::Sender<emulator::Input>,
) -> thread::JoinHandle<()> {
    debug!("Starting input loop");
    thread::spawn(move || {
        let stdin = io::stdin();
        for evt in stdin.keys() {
            if let Ok(key) = evt {
                let input = match key {
                    Key::Char('1') => Some(Input::Key1),
                    Key::Char('2') => Some(Input::Key2),
                    Key::Char('3') => Some(Input::Key3),
                    Key::Char('4') => Some(Input::KeyC),
                    Key::Char('q') => Some(Input::Key4),
                    Key::Char('w') => Some(Input::Key5),
                    Key::Char('e') => Some(Input::Key6),
                    Key::Char('r') => Some(Input::KeyD),
                    Key::Char('a') => Some(Input::Key7),
                    Key::Char('s') => Some(Input::Key8),
                    Key::Char('d') => Some(Input::Key9),
                    Key::Char('f') => Some(Input::KeyE),
                    Key::Char('z') => Some(Input::KeyA),
                    Key::Char('x') => Some(Input::Key0),
                    Key::Char('c') => Some(Input::KeyB),
                    Key::Char('v') => Some(Input::KeyF),
                    Key::Esc => Some(Input::Quit),
                    _ => None,
                };

                debug!("Input: {:?}", input);
                input.iter().for_each(|i| {
                    if let Err(err) = input_tx.try_send(*i) {
                        error!("Error sending input: {}", err);
                        return;
                    }

                    if *i == Input::Quit {
                        return;
                    }
                });
            }
        }
    })
}

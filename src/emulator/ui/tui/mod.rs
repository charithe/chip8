use crate::emulator;
use crate::emulator::common::{Error, Result};
use crate::emulator::{display, Input};
use log::{debug, error};
use std::{io, thread, time::Duration};
use termion::{
    event::Key,
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
    screen::AlternateScreen,
};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Text},
    Terminal,
};

#[macro_use]
use crossbeam_channel::select;
use crossbeam_channel;

mod widgets;

enum Command {
    Draw(display::Pixels),
    Quit,
}

pub fn start_loop(emu: &mut emulator::Emulator) -> Result<()> {
    let (cmd_tx, cmd_rx) = crossbeam_channel::bounded(0);
    let (input_tx, input_rx) = crossbeam_channel::bounded(0);
    let ticker = crossbeam_channel::tick(Duration::from_millis(2));

    start_render_loop(cmd_rx)?;
    start_input_loop(input_tx);

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
                match emu.step() {
                    Ok(Some(emulator::Step::Draw(pixels))) => {
                        match cmd_tx.try_send(Command::Draw(pixels)) {
                            Ok(_) => {},
                            Err(crossbeam_channel::TrySendError::Full(_)) => {},
                            Err(err) => return Err(Error::Unexpected(Box::new(err))),
                        }
                    },
                    Ok(_) => {},
                    Err(err) => return Err(err),
                }
            },
        }
    }
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
                    Key::Char('7') => Some(Input::Key1),
                    Key::Char('8') => Some(Input::Key2),
                    Key::Char('9') => Some(Input::Key3),
                    Key::Char('0') => Some(Input::KeyC),
                    Key::Char('u') => Some(Input::Key4),
                    Key::Char('i') => Some(Input::Key5),
                    Key::Char('o') => Some(Input::Key6),
                    Key::Char('p') => Some(Input::KeyD),
                    Key::Char('j') => Some(Input::Key7),
                    Key::Char('k') => Some(Input::Key8),
                    Key::Char('l') => Some(Input::Key9),
                    Key::Char(';') => Some(Input::KeyE),
                    Key::Char('m') => Some(Input::KeyA),
                    Key::Char(',') => Some(Input::Key0),
                    Key::Char('.') => Some(Input::KeyB),
                    Key::Char('/') => Some(Input::KeyF),
                    Key::Char('q') => Some(Input::Quit),
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

fn start_render_loop(
    cmd_rx: crossbeam_channel::Receiver<Command>,
) -> Result<thread::JoinHandle<()>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    debug!("Starting render loop");
    Ok(thread::spawn(move || loop {
        let mut scr = widgets::Screen::default().block(Block::default().borders(Borders::ALL));
        match cmd_rx.try_recv() {
            Ok(Command::Quit) => return,
            Ok(Command::Draw(p)) => scr = scr.pixels(p),
            _ => {}
        };

        terminal
            .draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref())
                    .split(f.size());

                f.render_widget(scr, chunks[0]);

                let text = [Text::styled("Second line", Style::default().fg(Color::Red))];
                let paragraph = Paragraph::new(text.iter())
                    .block(Block::default().title("Instruction").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(true);

                f.render_widget(paragraph, chunks[1]);
            })
            .unwrap();
    }))
}

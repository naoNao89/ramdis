use color_eyre::Result;
use crossterm::event::{DisableMouseCapture, Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::EnableMouseCapture;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use ratatui::prelude::*;
use ratatui::Frame;
use std::io::stdout;
use std::vec::Vec;

mod ui;
use crate::ui::ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    enable_raw_mode()?;

    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let result = App::new().run(&mut terminal).await;
    
    ratatui::restore();
    // ratatui::restore() does not disable mouse capture so...
    execute!(stdout(), DisableMouseCapture)?;
    terminal.show_cursor()?;
    result
}

#[derive(Debug)]
pub struct App {
    running: bool,
    event_stream: EventStream,
    choices: Vec<String>,
    current_input: String,
    current_index: u32,
    current_screen: CurrentScreen,
    current_emode: CurrentEditMode,
    current_curpos: u16
}

#[derive(Debug)]
pub enum CurrentScreen { Main, Editing, Exiting, Creating }

#[derive(Debug)]
pub enum CurrentEditMode { Data, Index, None }

impl App {
    pub fn new() -> Self {
        App {
            running: false,
            event_stream: EventStream::new(),
            choices: Vec::new(),
            current_input: String::new(),
            current_index: 0,
            current_screen: CurrentScreen::Main,
            current_emode: CurrentEditMode::None,
            current_curpos: 0
        }
    }

    pub async fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        ui(frame, self);
    }

    async fn handle_crossterm_events(&mut self) -> Result<()> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                match event {
                    Some(Ok(evt)) => {
                        match evt {
                            Event::Key(key)
                                if key.kind == KeyEventKind::Press
                                    => self.on_key_event(key),
                            Event::Mouse(_) => {}
                            Event::Resize(_, _) => {}
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                // Sleep for a short duration to avoid busy waiting.
            }
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            
            (_, KeyCode::Esc) => {
                self.current_screen = CurrentScreen::Main;
                self.current_emode = CurrentEditMode::None;
                self.current_curpos = 0;
            },

            (_, KeyCode::Char('n') | KeyCode::Char('N')) =>
                match self.current_emode
                {
                    CurrentEditMode::None => self.current_screen = CurrentScreen::Creating,
                    _ => {}
                },

            (_, KeyCode::Char('e') | KeyCode::Char('E')) =>
                match self.current_emode
                {
                    CurrentEditMode::None => self.current_screen = CurrentScreen::Editing,
                    _ => {}
                },
            
            (KeyModifiers::CONTROL, KeyCode::Char('i') | KeyCode::Char('I')) => {
                match self.current_screen
                {
                    CurrentScreen::Creating | CurrentScreen::Editing =>
                        self.current_emode = CurrentEditMode::Index,
                    
                    _ => self.current_emode = CurrentEditMode::None
                }
            },

            (KeyModifiers::CONTROL, KeyCode::Char('d') | KeyCode::Char('D')) => {
                match self.current_screen
                {
                    CurrentScreen::Creating | CurrentScreen::Editing =>
                        self.current_emode = CurrentEditMode::Data,
                    
                    _ => self.current_emode = CurrentEditMode::None
                }
            }
            (_, code) => {
                match code
                {
                    KeyCode::Left => self.current_curpos = self.current_curpos.saturating_sub(1),
                    KeyCode::Right => self.current_curpos = self.current_curpos.saturating_add(1),
                    KeyCode::Backspace => {
                        self.current_curpos = self.current_curpos.saturating_sub(1);
                        match self.current_emode
                        {
                            CurrentEditMode::Data => {
                                self.current_input.split_off(self.current_input.len() - 1).truncate(0);
                            },
                            _ => todo!()
                        }
                    },
                    KeyCode::Char(to_insert) => {
                        self.current_curpos = self.current_curpos.saturating_add(1);
                        match self.current_emode
                        {
                            CurrentEditMode::Data => self.current_input.push(to_insert),
                            _ => todo!()
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    fn quit(&mut self) {
        if let CurrentScreen::Exiting = self.current_screen
        {
            self.running = false;
        }
        else if let CurrentScreen::Main = self.current_screen
        {
            self.current_screen = CurrentScreen::Exiting;
        }
    }
}

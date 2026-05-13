pub mod events;
mod ui;

use crate::tui::events::{BackendEvent, FrontendEvent};
use crate::vector_db::SearchResult;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListState;
use std::io;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub enum Screen {
    Indexing,
    Searching,
}

pub struct TuiApp {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub list_state: ListState,
    pub progress: f64,
    pub status_text: String,
    pub current_screen: Screen,
    pub should_quit: bool,
    pub tx: Sender<FrontendEvent>,
    pub rx: Receiver<BackendEvent>,
}

impl TuiApp {
    pub fn new(tx: Sender<FrontendEvent>, rx: Receiver<BackendEvent>) -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            list_state: ListState::default(),
            progress: 0.0,
            status_text: String::from("Preparing..."),
            current_screen: Screen::Indexing,
            should_quit: false,
            tx,
            rx,
        }
    }

    pub fn update_from_backend(&mut self) {
        for event in self.rx.try_iter() {
            match event {
                BackendEvent::Progress(p, file) => {
                    self.progress = p;
                    self.status_text = format!("Indexing: {}", file);
                }
                BackendEvent::IndexingFinished => {
                    self.current_screen = Screen::Searching;
                    self.status_text = String::from("Ready to search");
                }
                BackendEvent::SearchResults(res) => {
                    self.status_text = if res.is_empty() {
                        "No results found".to_string()
                    } else {
                        format!("Found {} result(s)  |  ↑↓ to scroll", res.len())
                    };
                    self.list_state = ListState::default();
                    if !res.is_empty() {
                        self.list_state.select(Some(0));
                    }
                    self.results = res;
                }
                BackendEvent::Error(err) => {
                    self.status_text = format!("Error: {}", err);
                }
            }
        }
    }

    fn scroll_up(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn scroll_down(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let last = self.results.len() - 1;
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(last),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn run(mut self) -> Result<(), io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        while !self.should_quit {
            terminal.draw(|f| ui::render(f, &mut self))?;

            self.update_from_backend();

            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        KeyCode::Esc => {
                            let _ = self.tx.send(FrontendEvent::Quit);
                            self.should_quit = true;
                        }
                        KeyCode::Up => self.scroll_up(),
                        KeyCode::Down => self.scroll_down(),
                        KeyCode::Char(c) => {
                            if matches!(self.current_screen, Screen::Searching) {
                                self.query.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            self.query.pop();
                        }
                        KeyCode::Enter => {
                            if matches!(self.current_screen, Screen::Searching)
                                && !self.query.is_empty()
                            {
                                self.status_text = format!("Searching for: '{}'...", self.query);
                                self.results.clear();
                                self.list_state = ListState::default();
                                let _ = self.tx.send(FrontendEvent::Search(self.query.clone()));
                            }
                        }
                        _ => {}
                    },

                    Event::Resize(_, _) => {}

                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }
}

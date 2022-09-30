use std::borrow::Cow;

#[allow(unused_imports)]
use crate::{dbg, debug, error};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui::{
    style::{Modifier, Style},
    text::{Span, Spans},
};

use crate::{
    app::AppAction,
    display::Line,
    key,
    traits::{EventAction, EventHandler},
};

#[derive(Debug, Clone)]
pub struct InsertMode {
    listening: bool,
    pos: usize,
    text: Vec<char>,
}

impl InsertMode {
    pub fn listen(&mut self) {
        self.listening = true;
    }

    pub fn stop_listen(&mut self) {
        self.listening = false;
    }

    pub fn is_listening(&self) -> bool {
        self.listening
    }

    pub fn replace_text(&mut self, text: Cow<'static, str>) {
        self.text = text.chars().collect();
        self.pos = self.text.len();
    }

    pub fn line(&self) -> Line<'static> {
        Line::new(Spans::from(vec![
            Span::raw(self.text.iter().take(self.pos).collect::<String>()),
            Span {
                content: format!(
                    "{}",
                    self.text
                        .iter()
                        .cloned()
                        .skip(self.pos)
                        .next()
                        .unwrap_or(' ')
                )
                .into(),
                style: Style::default().add_modifier(Modifier::REVERSED),
            },
            Span::raw(self.text.iter().skip(self.pos + 1).collect::<String>()),
        ]))
    }
}
impl Default for InsertMode {
    fn default() -> Self {
        Self {
            listening: false,
            pos: 0,
            text: Default::default(),
        }
    }
}

pub enum InsertAction<T> {
    Action(EventAction<T>),
    Accepted { action: T, text: String },
    Rejected(T),
}

impl<'a> EventHandler<'a> for InsertMode {
    type Action = InsertAction<AppAction>;
    type Context = ();
    fn handle_events(&mut self, event: &Event, _ctx: Self::Context) -> Self::Action {
        let unabsorbed = InsertAction::Action(EventAction::Unabsorbed(AppAction::None));
        if !self.listening {
            return unabsorbed;
        }
        match event {
            Event::Key(k) => {
                match k {
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: _,
                    }
                    | KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: KeyModifiers::SHIFT,
                        kind: KeyEventKind::Press,
                        state: _,
                    } => {
                        self.text.insert(self.pos, *c);
                        self.pos += 1;
                    }
                    key!(Right) => {
                        self.pos = self.text.len().min(self.pos + 1);
                    }
                    key!(Left) => {
                        self.pos = if self.pos > 0 { self.pos - 1 } else { 0 };
                    }
                    key!(Home) => {
                        self.pos = 0;
                    }
                    key!(End) => {
                        self.pos = self.text.len();
                    }
                    key!(Backspace) => {
                        if self.pos > 0 {
                            self.text.remove(self.pos - 1);
                            self.pos -= 1;
                        }
                    }
                    key!(Esc) => {
                        *self = Default::default();
                        return InsertAction::Rejected(AppAction::None);
                    }
                    key!(Enter) => {
                        let s = std::mem::replace(self, Default::default());
                        // let text = s.text.iter().collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ");
                        // let text = s
                        //     .text
                        //     .into_iter()
                        //     .collect::<String>()
                        //     .trim_start_matches(' ')
                        //     .trim_end_matches(' ')
                        //     .to_owned();
                        let text = s.text.iter().collect();
                        return InsertAction::Accepted {
                            action: AppAction::None,
                            text,
                        };
                    }
                    _ => (), // all keys are absorbed
                }
            }
            _ => return unabsorbed,
        }
        InsertAction::Action(EventAction::Absorbed(AppAction::None))
    }
}

#[macro_export]
macro_rules! key {
    ($key:ident) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$key,
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: _,
        }
    };
    ($($ch:tt)*) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($($ch)*),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: _,
        }
    };
}

#[macro_export]
macro_rules! shift {
    ($key:ident) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$key,
            modifiers: crossterm::event::KeyModifiers::SHIFT,
            kind: crossterm::event::KeyEventKind::Press,
            state: _,
        }
    };
    ($($ch:tt)*) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($($ch)*),
            modifiers: crossterm::event::KeyModifiers::SHIFT,
            kind: crossterm::event::KeyEventKind::Press,
            state: _,
        }
    };
}

#[macro_export]
macro_rules! ctrl {
    ($key:ident) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$key,
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: _,
        }
    };
    ($($ch:tt)*) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($($ch)*),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: _,
        }
    };
}

#[macro_export]
macro_rules! alt {
    ($key:ident) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$key,
            modifiers: crossterm::event::KeyModifiers::ALT,
            kind: crossterm::event::KeyEventKind::Press,
            state: _,
        }
    };
    ($($ch:tt)*) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($($ch)*),
            modifiers: crossterm::event::KeyModifiers::ALT,
            kind: crossterm::event::KeyEventKind::Press,
            state: _,
        }
    };
}

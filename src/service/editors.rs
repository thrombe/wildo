#[allow(unused_imports)]
use crate::{dbg, debug, error};

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Debug};
use tui::{
    style::{Color, Style},
    text::Span,
};

use crate::{
    app::AppAction,
    content::{todo, traits::Content},
    ctrl, key,
    register::Id,
    traits::{EventAction, EventHandler},
};

#[derive(Debug, Deserialize, Serialize)]
pub enum YankType {
    Cut,
    Copy,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Yanker {
    pub yanks: Vec<Yank<Id>>,
    pub source: Id,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Yank<T> {
    pub id: T,
    pub pos: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Edit {
    Yanked {
        yank_type: YankType,
        source: Id,
        yanks: Vec<Yank<Id>>,
    },
    Pasted {
        source: Id,
        yanks: Vec<Yank<Id>>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EditManager {
    pub yanker: Option<Yanker>,

    // TODO: vectors do not seem the best fit for this job. some kinda circular stack with fixed length (it's called a ring buffer i think) might be better
    // all ids stored in here should be valid. (non weak). and should get unregistered once the edits are removed
    // but Yanker::{yank_from, yank_to} are still weak ig
    pub edit_stack: Vec<Edit>,
    pub undo_stack: Vec<Edit>, // edits get popped off and get stored here after getting converted into their undo edit
}
impl Default for EditManager {
    fn default() -> Self {
        Self {
            yanker: None,
            edit_stack: Default::default(),
            undo_stack: Default::default(),
        }
    }
}

// pub enum EditAction {
//     None,
// }

pub struct EditContext<'a> {
    source_id: Id,
    source: &'a mut Content,
    item: Option<Yank<Id>>,
}

impl<'a> EventHandler<'a> for EditManager {
    type Action = EventAction<AppAction>;
    type Context = EditContext<'a>;

    fn handle_events(&mut self, event: &Event, ctx: Self::Context) -> Self::Action {
        let same_source = self
            .yanker
            .as_ref()
            .map(|y| y.source == ctx.source_id)
            .unwrap_or(false);

        match event {
            Event::Key(k) => match k {
                key!('y') => {
                    if ctx.item.is_some() {
                        if same_source {
                            self.yanker.as_mut().unwrap().yanks.push(ctx.item.unwrap())
                        } else {
                            self.yanker = Some(Yanker {
                                yanks: vec![ctx.item.unwrap()],
                                source: ctx.source_id,
                            });
                        }
                        return EventAction::Absorbed(AppAction::MoveDown);
                    }
                }
                ctrl!('x') => {
                    if same_source {
                        todo!()
                    }
                }
                ctrl!('c') => {
                    if same_source {
                        todo!()
                    }
                }
                ctrl!('z') => {
                    todo!()
                }
                ctrl!('y') => {
                    todo!()
                }
                ctrl!('v') => {
                    todo!()
                }
                _ => (),
            },
            _ => (),
        }
        EventAction::Unabsorbed(AppAction::None)
    }
}

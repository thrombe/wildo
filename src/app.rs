#[allow(unused_imports)]
use crate::{dbg, debug, error};

use anyhow::Result;
use crossterm::event::{Event, EventStream};
use derivative::Derivative;
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::select;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

use crate::{
    content::traits::Content,
    content::{
        main_provider::MainProvider,
        traits::{DisplayContext, DrawContext, WidgetOutput},
    },
    key,
    register::{ContentRegister, Id},
    service::{db::DBHandler, editors::EditManager},
    stack::ContentStack,
    traits::{Display, Drawable, EventAction, EventHandler, SelectedIndex, Widget},
};

pub struct AppActionContext<'a> {
    pub register: &'a mut ContentRegister<Content, Id>,
    pub editor: &'a mut EditManager,
    pub stack: &'a mut ContentStack,
}
impl<'a> From<&'a mut App> for AppActionContext<'a> {
    fn from(a: &'a mut App) -> Self {
        Self {
            register: &mut a.content_register,
            editor: &mut a.editor,
            stack: &mut a.stack,
        }
    }
}
impl<'a, 'b: 'a> From<&'a mut AppActionContext<'b>> for AppActionContext<'a> {
    fn from(a: &'a mut AppActionContext) -> Self {
        Self {
            register: a.register,
            editor: a.editor,
            stack: a.stack,
        }
    }
}
pub type AppActionCallback =
    Box<dyn FnOnce(AppActionContext) -> Result<AppAction> + Send + Sync + 'static>;

#[derive(Derivative)]
#[derivative(Debug)]
pub enum AppAction {
    Callback {
        #[derivative(Debug = "ignore")]
        call: AppActionCallback,
    },
    Actions {
        v: Vec<Self>,
    },
    MoveDown,
    MoveUp,
    MoveRight,
    MoveLeft,
    None,
}
impl AppAction {
    pub fn apply<'a>(self, ctx: &mut AppActionContext<'a>) -> Result<()> {
        dbg!(&self);
        match self {
            Self::Callback { call } => {
                call(ctx.into())?.apply(ctx)?;
            }
            Self::Actions { v } => {
                v.into_iter()
                    .map(|a| a.apply(ctx))
                    .collect::<Result<()>>()?;
            }
            Self::MoveDown => {
                todo!()
            }
            Self::MoveUp => {
                todo!()
            }
            Self::MoveRight => {
                let id = ctx.stack.last();
                let id = ctx
                    .register
                    .get(id)
                    .map(|e| e.as_provider().map(|e| e.get_selected()))
                    .flatten()
                    .flatten();
                if id
                    .map(|id| ctx.register.get(id))
                    .flatten()
                    .map(|e| e.as_widget().is_some())
                    .unwrap_or(false)
                {
                    ctx.stack.push(id.unwrap());
                }
            }
            Self::MoveLeft => {
                let _ = ctx.stack.pop();
            }
            Self::None => (),
        }
        Ok(())
    }

    pub fn chain<T: IntoIterator<Item = Self>>(mut self, other: T) -> Self {
        match &mut self {
            Self::Actions { v } => {
                other.into_iter().for_each(|a| {
                    // match a { // ignore Self::None and unroll Self::Actions
                    //     Self::Actions { v: w } => {
                    //         v.extend(w.into_iter());
                    //     }
                    //     Self::None => {}
                    //     a => {
                    //         v.push(a);
                    //     }
                    // }
                    v.push(a);
                });
            }
            _ => self = Self::Actions { v: vec![self] }.chain(other),
        }
        self
    }
}

pub struct App {
    pub stack: ContentStack,
    pub content_register: ContentRegister<Content, Id>,
    pub editor: EditManager,
    pub quit: bool,
}

impl App {
    pub fn new() -> Self {
        let t: Content = MainProvider::new("Wildo").into();
        let mut content_register = ContentRegister::new();
        let mp = content_register.alloc(t);
        let stack = ContentStack::new(mp);
        let a = Self {
            stack,
            content_register,
            editor: Default::default(),
            quit: false,
        };
        a
    }

    pub fn load() -> Result<Self> {
        let a = DBHandler::load()?
            .map(|db| Self {
                content_register: db.register,
                editor: db.editor,
                ..Self::new()
            })
            .unwrap_or(Self::new());
        Ok(a)
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut events = EventStream::new();
        loop {
            if self.quit {
                return Ok(());
            }
            terminal.draw(|f| self.render(f))?;
            let sleep = tokio::time::sleep(Duration::from_secs_f64(0.5));
            let event = events.next().fuse();
            select! {
                Some(e) = event => self.handle_event(&e.unwrap())?,
                _ = sleep => (),
            }
        }
    }

    pub fn save(self) -> Result<()> {
        DBHandler {
            register: self.content_register,
            editor: self.editor,
        }
        .save()
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        let rect = f.size();
        let id = self.stack.last();
        let ctx = (&(*self)).into();
        let lb = self
            .content_register
            .get(id)
            .unwrap()
            .as_widget()
            .map(|e| e.display(ctx));
        let mut i = SelectedIndex::none();
        let index = self
            .content_register
            .get_mut(id)
            .unwrap()
            .as_provider_mut()
            .map(|e| e.context_mut())
            .unwrap_or(&mut i);

        lb.map(|e| {
            e.draw(
                f,
                DrawContext {
                    area: rect,
                    selected_index: index,
                },
            )
        });
    }

    fn handle_event(&mut self, event: &Event) -> Result<()> {
        dbg!(event);
        let id = self.stack.last();
        let a = self
            .content_register
            .get_mut(id)
            .map(|e| e.as_event_handler())
            .flatten()
            .map(|e| e.handle_events(event, id))
            .unwrap_or(EventAction::Unabsorbed(AppAction::None));

        let a = match a {
            EventAction::Unabsorbed(a) => {
                match event {
                    Event::Key(k) => match k {
                        key!('q') => {
                            self.quit = true;
                        }
                        key!(Right) => {
                            AppAction::MoveRight.apply(&mut self.into())?;
                        }
                        key!(Left) => {
                            AppAction::MoveLeft.apply(&mut self.into())?;
                        }
                        _ => {}
                    },
                    _ => {}
                }
                a
            }
            EventAction::Absorbed(a) => a,
        };

        a.apply(&mut self.into())
    }
}

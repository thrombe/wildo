use crate::{
    app::{App, AppAction},
    display::{Item, ListBuilder},
    key,
    register::{ContentRegister, Id},
    service::editors::Yank,
    traits::{
        Display, Drawable, EventAction, EventHandler, Provider, SelectedIndex, Widget, YankDest,
    },
};
#[allow(unused_imports)]
use crate::{dbg, debug, error};

use anyhow::Result;
use crossterm::event::Event;
use serde::{Deserialize, Serialize};
use std::{any::Any, borrow::Cow, fmt::Debug};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content(Box<dyn ContentTrait>);
impl std::ops::Deref for Content {
    type Target = Box<dyn ContentTrait>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Content {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Box<dyn ContentTrait>> for Content {
    fn from(o: Box<dyn ContentTrait>) -> Self {
        Self(o)
    }
}
impl Content {
    pub fn new(t: Box<dyn ContentTrait>) -> Self {
        Self(t)
    }
}

pub trait BClone {
    fn bclone(&self) -> Box<dyn ContentTrait>;
}

impl<T> BClone for T
where
    T: 'static + Clone + Debug + ContentTrait,
{
    fn bclone(&self) -> Box<dyn ContentTrait> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn ContentTrait> {
    fn clone(&self) -> Self {
        self.bclone()
    }
}

impl<T> From<T> for Content
where
    T: ContentTrait + 'static,
{
    fn from(t: T) -> Self {
        Content::new(Box::new(t) as Box<dyn ContentTrait>)
    }
}

// ? this requirement is quite dangerous time waster. can it be enforced?
// the macro must be called on all the traits, else those implimentations will not be used
#[typetag::serde(tag = "type")]
pub trait ContentTrait
where
    Self: std::fmt::Debug + Send + Sync + BClone + Any,
{
    // for downcasting (the macro has implimentation for this)
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_widget(
        &self,
    ) -> Option<&dyn Widget<Context = DisplayContext, Output = WidgetOutput<'static>>> {
        None
    }
    fn as_event_handler(
        &mut self,
    ) -> Option<&mut dyn EventHandler<Action = EventAction<AppAction>, Context = Id>> {
        None
    }
    fn as_display(&self) -> &dyn Display<Output = Item<'static>>;
    fn as_display_mut(&mut self) -> &mut dyn Display<Output = Item<'static>>;
    fn as_yankdest(&mut self) -> Option<&mut dyn YankDest<Query = Yank<Id>>> {
        None
    }
    fn as_provider(
        &self,
    ) -> Option<&dyn Provider<Item = Option<Id>, Context = &mut SelectedIndex>> {
        None
    }
    fn as_provider_mut(
        &mut self,
    ) -> Option<&mut dyn Provider<Item = Option<Id>, Context = &mut SelectedIndex>> {
        None
    }
}

#[macro_export]
macro_rules! impliment_content {
    ($t:ident, IDK) => {};
    ($t:ident, Provider) => {
        fn as_provider(&self) -> Option<&dyn Provider<Item = Option<Id>, Context = &mut SelectedIndex>> {Some(self)}
        fn as_provider_mut(&mut self) -> Option<&mut dyn Provider<Item = Option<Id>, Context = &mut SelectedIndex>> {Some(self)}
    };
    ($t:ident, YankDest) => {
        fn as_yankdest(&mut self) -> Option<&mut dyn YankDest<Query = Yank<Id>>> {Some(self)}
    };
    ($t:ident, EventHandler) => {
        fn as_event_handler(&mut self) -> Option<&mut dyn EventHandler<Action = EventAction<AppAction>, Context = Id>> {Some(self)}
    };
    ($t:ident, Display) => {
        fn as_display(&self) -> &dyn Display<Output = Item<'static>> {self}
        fn as_display_mut(&mut self) -> &mut dyn Display<Output = Item<'static>> {self}
    };
    ($t:ident, Widget) => {
        fn as_widget(&self) -> Option<&dyn Widget<Context = DisplayContext, Output = WidgetOutput<'static>>> {Some(self)}
    };
    ($t:ident, $r:tt, $($e:tt), +) => {
        impliment_content!($t, $r);
        $(
            impliment_content!($t, $e);
        )+

        fn as_any(&self) -> &dyn std::any::Any {self}
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {self}
    };
}

// somehow forces the macro to be in this module.
pub(crate) use impliment_content;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Container<T> {
    pub items: Vec<T>,
    #[serde(skip_serializing, skip_deserializing, default = "Default::default")]
    pub selected_index: SelectedIndex,
}
impl<T> Default for Container<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            selected_index: Default::default(),
        }
    }
}
impl<'a, T> EventHandler<'a> for Container<T> {
    type Action = EventAction<AppAction>;
    type Context = ();
    fn handle_events(&mut self, event: &Event, _ctx: Self::Context) -> Self::Action {
        let unabsorbed = EventAction::Unabsorbed(AppAction::None);
        match event {
            Event::Key(k) => match k {
                key!(Up) => {
                    self.selected_index
                        .select(if self.selected_index.selected_index() > 0 {
                            self.selected_index.selected_index() - 1
                        } else {
                            0
                        });
                }
                key!(Down) => {
                    if self.items.len() > 0 {
                        self.selected_index.select(
                            (self.items.len() - 1).min(self.selected_index.selected_index() + 1),
                        );
                    }
                }
                key!(Home) => {
                    self.selected_index.select(0);
                }
                key!(End) => {
                    if self.items.len() > 0 {
                        self.selected_index.select(self.items.len() - 1);
                    }
                }
                _ => return unabsorbed,
            },
            _ => return unabsorbed,
        }
        EventAction::Absorbed(AppAction::None)
    }
}
impl<T: PartialEq + Eq + Copy> YankDest for Container<T> {
    type Query = Yank<T>;
    fn insert(&mut self, q: Self::Query) {
        self.items.insert(q.pos, q.id);
        if q.pos <= self.selected_index.selected_index() {
            self.selected_index
                .select((self.items.len() - 1).min(self.selected_index.selected_index() + 1))
        }
    }
    fn remove(&mut self, q: Self::Query) -> bool {
        if q.pos <= self.selected_index.selected_index()
            && self.items.get(q.pos).map(|&e| e == q.id).unwrap_or(false)
        {
            if self.selected_index.selected_index() > 0 {
                self.selected_index
                    .select(self.selected_index.selected_index() - 1);
            }
            self.items.remove(q.pos);
            true
        } else {
            false
        }
    }
}

pub struct DisplayContext<'a> {
    pub content_register: &'a ContentRegister<Content, Id>,
}
impl<'a> From<&'a App> for DisplayContext<'a> {
    fn from(a: &'a App) -> Self {
        Self {
            content_register: &a.content_register,
        }
    }
}

pub struct DrawContext<'a> {
    pub area: Rect,
    pub selected_index: &'a mut SelectedIndex,
}

pub enum WidgetOutput<'a> {
    TodoList {
        content: ListBuilder<'a>,
        date: ListBuilder<'a>,
    },
    MainProvider {
        content: ListBuilder<'a>,
    },
}

// pub enum DisplayOutput<'a> {
//     Todo {
//         content: Item<'a>,
//         date: Item<'a>,
//     },
//     Title {
//         title: Item<'a>,
//     },
// }

impl<'a> Drawable for WidgetOutput<'a> {
    type Context = DrawContext<'a>;

    fn draw<B: Backend>(&self, f: &mut Frame<B>, input: Self::Context) {
        match self {
            WidgetOutput::TodoList { content, date } => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
                    .split(input.area);
                let content_area = chunks[0];
                let date_area = chunks[1];

                let content_list =
                    content.list(content_area, input.selected_index.selected_index());
                let date_list = date.list(date_area, input.selected_index.selected_index());

                f.render_stateful_widget(content_list, content_area, input.selected_index.into());
                f.render_stateful_widget(date_list, date_area, input.selected_index.into());
            }
            WidgetOutput::MainProvider { content } => {
                let content_list = content.list(input.area, input.selected_index.selected_index());
                f.render_stateful_widget(content_list, input.area, input.selected_index.into());
            }
        }
    }
}

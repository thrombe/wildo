#[allow(unused_imports)]
use crate::{dbg, debug, error};

use anyhow::Result;
use crossterm::event::Event;
use std::borrow::Cow;
use tui::{backend::Backend, widgets::ListState, Frame};

use crate::{app::AppAction, content::traits::Content, register::Id};

pub enum EventAction<T> {
    Absorbed(T),
    Unabsorbed(T),
}

pub trait EventHandler<'a> {
    type Action;
    type Context;
    fn handle_events(&mut self, event: &Event, ctx: Self::Context) -> Self::Action;
}

pub trait Widget<'a> {
    // ? maybe GAT
    type Context;
    type Output: Drawable;

    fn display(&self, context: Self::Context) -> Self::Output;
}

pub trait Drawable {
    type Context;
    fn draw<B: Backend>(&self, f: &mut Frame<B>, input: Self::Context);
}

pub trait Display {
    type Output;
    fn text(&self) -> Cow<'static, str>;
    fn display(&self) -> Self::Output;
    fn set_text(&mut self, name: Cow<'static, str>);
}

pub trait YankDest {
    type Query;
    fn insert(&mut self, q: Self::Query);
    fn remove(&mut self, q: Self::Query) -> bool;
}

pub trait Provider<'a> {
    // ? matbe GAT
    type Item;
    type Context;
    fn get(&self, index: usize) -> Self::Item;
    fn context_mut(&'a mut self) -> Self::Context;
    fn get_selected(&self) -> Self::Item;
}

/// wrapping ListState to make sure not to call select(None) and to eliminate the use of unwrap() on selected_index()
/// currently, theres no way to access/suggest the offset
#[derive(Debug, Clone)]
pub struct SelectedIndex {
    index: ListState,
}
impl Default for SelectedIndex {
    fn default() -> Self {
        Self::new()
    }
}
impl Into<ListState> for SelectedIndex {
    fn into(self) -> ListState {
        self.index
    }
}
impl<'a> Into<&'a mut ListState> for &'a mut SelectedIndex {
    fn into(self) -> &'a mut ListState {
        &mut self.index
    }
}
impl SelectedIndex {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { index: state }
    }

    pub fn none() -> Self {
        Self {
            index: Default::default(),
        }
    }

    pub fn selected_index(&self) -> usize {
        self.index.selected().unwrap()
    }

    pub fn select(&mut self, index: usize) {
        self.index.select(Some(index));
    }
}

#[allow(unused_imports)]
use crate::{dbg, debug, error};

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tui::{
    style::{Color, Modifier, Style},
    text::Span,
};

use crate::{
    display::{Item, Line, SelectedText},
    impliment_content,
    traits::Display,
};

use super::traits::ContentTrait;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Todo {
    pub content: Cow<'static, str>,
    pub due_date: Option<Date>,
    pub due_time: Option<Time>,
    pub status: TodoStatus,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum TodoStatus {
    Pending,
    Done,
    Ignored,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Date {
    pub day: u8,
    pub month: u8,
    pub year: u16,
}
impl From<NaiveDate> for Date {
    fn from(d: NaiveDate) -> Self {
        Self {
            day: d.day().try_into().unwrap(),
            month: d.month().try_into().unwrap(),
            year: d.year().try_into().unwrap(),
        }
    }
}
impl From<Date> for NaiveDate {
    fn from(d: Date) -> Self {
        Self::from_ymd(d.year.into(), d.month.into(), d.day.into())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Time {
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
}

impl Todo {
    pub fn new<T: Into<Cow<'static, str>>>(content: T) -> Self {
        Self {
            content: content.into(),
            due_date: None,
            due_time: None,
            status: TodoStatus::Pending,
        }
    }
}

impl Display for Todo {
    // TODO: a todo should give multiple items, as TodoList expectes multiple items in its Widget implimentation. this trait is not good enough.
    type Output = Item<'static>;
    fn text(&self) -> Cow<'static, str> {
        self.content.clone()
    }
    fn display(&self) -> Self::Output {
        let mut text = Line::new(Span::raw(self.text()));
        let mut selected_text = text.clone();
        let st = match self.status {
            TodoStatus::Pending => Style::default().fg(Color::Rgb(200, 200, 100)),
            TodoStatus::Done => Style::default()
                .fg(Color::Rgb(90, 130, 90))
                .add_modifier(Modifier::CROSSED_OUT),
            TodoStatus::Ignored => Style::default().fg(Color::DarkGray),
        };
        text.text_style(st);
        selected_text.text_style(st.add_modifier(Modifier::BOLD));

        Item {
            text: vec![text.clone()],
            selected_text: SelectedText::Lines(vec![selected_text]),
        }
    }
    fn set_text(&mut self, name: Cow<'static, str>) {
        self.content = name;
    }
}

#[typetag::serde]
impl ContentTrait for Todo {
    impliment_content!(Todo, Display, IDK);
}

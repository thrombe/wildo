#[allow(unused_imports)]
use crate::{dbg, debug, error};

use chrono::{Datelike, NaiveDate};
use crossterm::event::{Event, KeyCode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tui::{
    layout::{Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders},
};

use crate::{
    app::{self, AppAction, AppActionCallback},
    content::todo::{Date, Todo, TodoStatus},
    ctrl,
    display::{Item, Line, ListBuilder, SelectedText},
    key,
    register::Id,
    service::{
        editors::{Edit, Yank},
        insert_mode::{InsertAction, InsertMode},
    },
    shift,
    traits::{Display, EventAction, EventHandler, Provider, SelectedIndex, Widget, YankDest},
};

use super::traits::{
    impliment_content, Container, Content, ContentTrait, DisplayContext, WidgetOutput,
};

#[derive(Debug, Clone, Copy)]
enum ListenTarget {
    ContentCreate,
    ContentEdit,
    DueDate,
    DueTime,
    None,
}
impl Default for ListenTarget {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TodoList {
    container: Container<Id>,
    title: Cow<'static, str>,
    #[serde(skip_serializing, skip_deserializing, default = "Default::default")]
    insert_mode: InsertMode,
    #[serde(skip_serializing, skip_deserializing, default = "Default::default")]
    listen_target: ListenTarget,
}

impl TodoList {
    pub fn new<T: Into<Cow<'static, str>>>(title: T) -> Self {
        Self {
            container: Default::default(),
            title: title.into(),
            insert_mode: Default::default(),
            listen_target: Default::default(),
        }
    }
}

impl<'a> EventHandler<'a> for TodoList {
    type Action = EventAction<AppAction>;
    type Context = Id;
    fn handle_events(&mut self, event: &Event, self_id: Self::Context) -> Self::Action {
        let rejected_creation = |id| AppAction::Callback {
            call: Box::new(move |ctx| {
                let me = ctx
                    .register
                    .get_mut(self_id)
                    .unwrap()
                    .as_any_mut()
                    .downcast_mut::<Self>()
                    .unwrap();
                me.remove(Yank {
                    id,
                    pos: me.container.selected_index.selected_index(),
                });
                ctx.register.unregister(id);
                Ok(AppAction::None)
            }),
        };

        let a = match self.insert_mode.handle_events(event, ()) {
            InsertAction::Action(a) => a,
            InsertAction::Accepted { action, text } => {
                let text = text
                    .trim_end_matches(' ')
                    .trim_start_matches(' ')
                    .to_owned();
                let id = self.container.items[self.container.selected_index.selected_index()];
                let a = if text.len() > 0 {
                    let a = match self.listen_target {
                        ListenTarget::ContentCreate | ListenTarget::ContentEdit => {
                            action.chain([AppAction::Callback {
                                call: Box::new(move |ctx| {
                                    ctx.register
                                        .get_mut(id)
                                        .unwrap()
                                        .as_display_mut()
                                        .set_text(Cow::Owned(text));
                                    Ok(AppAction::None)
                                }),
                            }])
                        }
                        ListenTarget::DueDate => {
                            action.chain([AppAction::Callback {
                                call: Box::new(move |ctx| {
                                    let t = ctx
                                        .register
                                        .get_mut(id)
                                        .unwrap()
                                        .as_any_mut()
                                        .downcast_mut::<Todo>()
                                        .unwrap();
                                    // let today = chrono::offset::Local::now();//.naive_local();
                                    t.due_date = NaiveDate::parse_from_str(&text, "%d-%m-%Y")
                                        .ok()
                                        .map(Date::from)
                                        .or(t.due_date);
                                    Ok(AppAction::None)
                                }),
                            }])
                        }
                        ListenTarget::DueTime => todo!(),
                        ListenTarget::None => unreachable!(),
                    };
                    a
                } else {
                    match self.listen_target {
                        ListenTarget::ContentCreate => action.chain([rejected_creation(id)]),
                        ListenTarget::DueDate => action.chain([AppAction::Callback {
                            call: Box::new(move |ctx| {
                                ctx.register
                                    .get_mut(id)
                                    .unwrap()
                                    .as_any_mut()
                                    .downcast_mut::<Todo>()
                                    .unwrap()
                                    .due_date = None;
                                Ok(AppAction::None)
                            }),
                        }]),
                        ListenTarget::DueTime => todo!(),
                        ListenTarget::ContentEdit => action,
                        ListenTarget::None => unreachable!(),
                    }
                };
                self.listen_target = ListenTarget::None;
                EventAction::Absorbed(a)
            }
            InsertAction::Rejected(action) => {
                let a = if let ListenTarget::ContentCreate = self.listen_target {
                    let id = self.container.items[self.container.selected_index.selected_index()];
                    action.chain([rejected_creation(id)])
                } else {
                    action
                };
                self.listen_target = ListenTarget::None;
                EventAction::Absorbed(a)
            }
        };
        let a = match a {
            EventAction::Absorbed(a) => return EventAction::Absorbed(a),
            EventAction::Unabsorbed(a) => a,
        };

        let a = match self.container.handle_events(event, ()) {
            EventAction::Absorbed(action) => return EventAction::Absorbed(action.chain([a])),
            EventAction::Unabsorbed(action) => a.chain([action]),
        };

        match event {
            Event::Key(k) => match k {
                key!('a') => {
                    let add_action = AppAction::Callback {
                        call: Box::new(move |ctx| {
                            let tl: Content = Todo::new("").into();
                            let id = ctx.register.alloc(tl);
                            let me = ctx
                                .register
                                .get_mut(self_id)
                                .unwrap()
                                .as_any_mut()
                                .downcast_mut::<Self>()
                                .unwrap();
                            me.listen_target = ListenTarget::ContentCreate;
                            let new_index = me
                                .container
                                .items
                                .len()
                                .min(me.container.selected_index.selected_index() + 1);
                            let y = Yank { id, pos: new_index };
                            me.insert_mode.listen();
                            me.insert(y); // ? maybe let the editor do this?
                            me.container.selected_index.select(new_index); // set index after inserting the element
                            ctx.editor.edit_stack.push(Edit::Pasted {
                                source: self_id,
                                yanks: vec![y],
                            });
                            Ok(AppAction::None)
                        }),
                    };
                    return EventAction::Absorbed(a.chain([add_action]));
                }
                key!('d') => {
                    let id = self.container.items[self.container.selected_index.selected_index()];
                    let action = AppAction::Callback {
                        call: Box::new(move |ctx| {
                            let text = ctx
                                .register
                                .get(id)
                                .unwrap()
                                .as_any()
                                .downcast_ref::<Todo>()
                                .unwrap()
                                .due_date
                                .map(NaiveDate::from)
                                .map(|d| d.format("%d-%m-%Y").to_string())
                                .unwrap_or("".to_owned());
                            let mut me = ctx
                                .register
                                .get_mut(self_id)
                                .unwrap()
                                .as_any_mut()
                                .downcast_mut::<Self>()
                                .unwrap();
                            me.listen_target = ListenTarget::DueDate;
                            me.insert_mode.listen();
                            me.insert_mode.replace_text(Cow::from(text));
                            Ok(AppAction::None)
                        }),
                    };
                    return EventAction::Absorbed(a.chain([action]));
                }
                shift!('D') => {
                    self.insert_mode.listen();
                    self.listen_target = ListenTarget::DueDate;
                    return EventAction::Absorbed(a);
                }
                // key!('t') => {
                //     todo!()
                // }
                // shift!('T') => {
                //     self.insert_mode.listen();
                //     self.listen_target = ListenTarget::DueTime;
                //     return EventAction::Absorbed(a);
                // }
                key!('i') => {
                    let id = self.container.items[self.container.selected_index.selected_index()];
                    let action = AppAction::Callback {
                        call: Box::new(move |ctx| {
                            let text = ctx.register.get(id).unwrap().as_display().text();
                            let me = ctx
                                .register
                                .get_mut(self_id)
                                .unwrap()
                                .as_any_mut()
                                .downcast_mut::<Self>()
                                .unwrap();
                            me.listen_target = ListenTarget::ContentEdit;
                            me.insert_mode.listen();
                            me.insert_mode.replace_text(text);
                            Ok(AppAction::None)
                        }),
                    };
                    return EventAction::Absorbed(a.chain([action]));
                }
                key!('c') => {
                    let id = self.container.items[self.container.selected_index.selected_index()];
                    let action = AppAction::Callback {
                        call: Box::new(move |ctx| {
                            let t = ctx
                                .register
                                .get_mut(id)
                                .unwrap()
                                .as_any_mut()
                                .downcast_mut::<Todo>()
                                .unwrap();
                            match t.status {
                                TodoStatus::Pending => {
                                    t.status = TodoStatus::Done;
                                }
                                TodoStatus::Done => {
                                    t.status = TodoStatus::Pending;
                                }
                                TodoStatus::Ignored => {
                                    t.status = TodoStatus::Done;
                                }
                            }
                            Ok(AppAction::None)
                        }),
                    };
                    return EventAction::Absorbed(a.chain([action]));
                }

                // TODO: temporary implimentation. till yanking is properly implimented
                ctrl!('j') => {
                    let i = self.container.selected_index.selected_index();
                    let items = &mut self.container.items;
                    if items.len() > 1 && i < items.len() - 1 {
                        let id1 = items[i];
                        let id2 = items[i + 1];
                        items[i] = id2;
                        items[i + 1] = id1;
                        self.container.selected_index.select(i + 1);
                    }
                    return EventAction::Absorbed(a);
                }
                ctrl!('k') => {
                    let i = self.container.selected_index.selected_index();
                    let items = &mut self.container.items;
                    if items.len() > 1 && i >= 1 {
                        let id1 = items[i];
                        let id2 = items[i - 1];
                        items[i] = id2;
                        items[i - 1] = id1;
                        self.container.selected_index.select(i - 1);
                    }
                    return EventAction::Absorbed(a);
                }
                _ => (),
            },
            _ => (),
        }
        EventAction::Unabsorbed(a)
    }
}

impl<'a> Widget<'a> for TodoList {
    type Context = DisplayContext<'a>;
    type Output = WidgetOutput<'static>; // ? maybe try use GAT here
    fn display(&self, context: Self::Context) -> Self::Output {
        let mut content = ListBuilder::default();
        content
            .title(Span::raw(format!("List Name: {}", self.title.clone())))
            .block(
                Block::default()
                    .border_style(Style::default().fg(Color::Rgb(150, 150, 150)))
                    // .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
                    .borders(Borders::all()),
            );
        content.items = self
            .container
            .items
            .iter()
            .cloned()
            .map(|id| {
                context
                    .content_register
                    .get(id)
                    .map(|e| e.as_display().display())
                    .unwrap()
            })
            .collect();

        let mut date = ListBuilder::default();
        date.block(
            Block::default()
                .title("Due Date")
                .border_style(Style::default().fg(Color::Rgb(150, 150, 150)))
                // .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
                .borders(Borders::all()),
        );
        let st = Style::default().fg(Color::Rgb(200, 200, 100));
        date.items = self
            .container
            .items
            .iter()
            .cloned()
            .map(|id| {
                context
                    .content_register
                    .get(id)
                    .map(|e| {
                        e.as_any()
                            .downcast_ref::<Todo>()
                            .unwrap()
                            .due_date
                            .map(NaiveDate::from)
                            .map(|d| d.format("%d-%m-%Y").to_string())
                    })
                    .flatten()
                    .map(Cow::from)
                    .unwrap_or(Cow::from(""))
            })
            .map(Span::raw)
            .map(Line::new)
            .map(|mut l| {
                l.text_style(st);
                Item {
                    text: vec![l],
                    selected_text: SelectedText::Style(st.add_modifier(Modifier::BOLD)),
                }
            })
            .collect();

        if self.insert_mode.is_listening() {
            let line = self.insert_mode.line();
            let item = Item {
                text: vec![line.clone()],
                selected_text: SelectedText::Lines(vec![line]),
            };
            match self.listen_target {
                ListenTarget::ContentCreate | ListenTarget::ContentEdit => {
                    content.items[self.container.selected_index.selected_index()] = item;
                }
                ListenTarget::DueDate => {
                    date.items[self.container.selected_index.selected_index()] = item;
                }
                ListenTarget::DueTime => todo!(),
                ListenTarget::None => unreachable!(),
            }
        }
        WidgetOutput::TodoList { content, date }
    }
}

impl Display for TodoList {
    type Output = Item<'static>;
    fn text(&self) -> Cow<'static, str> {
        self.title.clone()
    }
    fn display(&self) -> Self::Output {
        let mut text = Line::new(Span::raw(self.text()));
        let st = Style::default().fg(Color::Rgb(200, 200, 100));
        text.text_style(st);
        Item {
            text: vec![text],
            selected_text: SelectedText::Style(st.add_modifier(Modifier::BOLD)),
        }
    }
    fn set_text(&mut self, name: Cow<'static, str>) {
        self.title = name;
    }
}

impl YankDest for TodoList {
    type Query = Yank<Id>;
    fn insert(&mut self, q: Self::Query) {
        self.container.insert(q)
    }
    fn remove(&mut self, q: Self::Query) -> bool {
        self.container.remove(q)
    }
}

impl<'a> Provider<'a> for TodoList {
    type Context = &'a mut SelectedIndex;
    type Item = Option<Id>;
    fn get(&self, index: usize) -> Self::Item {
        self.container.items.get(index).map(|&e| e)
    }
    fn context_mut(&'a mut self) -> Self::Context {
        &mut self.container.selected_index
    }
    fn get_selected(&self) -> Self::Item {
        self.container
            .items
            .get(self.container.selected_index.selected_index())
            .map(|&e| e)
    }
}

#[typetag::serde]
impl ContentTrait for TodoList {
    impliment_content!(TodoList, Widget, EventHandler, Display, YankDest, Provider);
}

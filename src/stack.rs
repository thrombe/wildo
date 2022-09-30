#[allow(unused_imports)]
use crate::{dbg, debug, error};

use crate::{register::Id, traits::SelectedIndex};

#[derive(Clone, Debug)]
pub struct ContentStack {
    stack: Vec<Id>,
}
impl ContentStack {
    pub fn new<T>(main_provider: T) -> Self
    where
        T: Into<Id>,
    {
        Self {
            stack: vec![main_provider.into().into()],
        }
    }

    pub fn main_provider(&self) -> Id {
        *self.stack.first().unwrap()
    }

    pub fn push<T>(&mut self, id: T)
    where
        T: Into<Id>,
    {
        self.stack.push(id.into());
    }

    pub fn pop(&mut self) -> Option<Id> {
        dbg!(&self);
        debug!("popping");
        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }

    pub fn last(&self) -> Id {
        *self.stack.last().unwrap()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn get(&self, index: usize) -> Id {
        self.stack[index]
    }
}

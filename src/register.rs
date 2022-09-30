#[allow(unused_imports)]
use crate::{dbg, debug, error};

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use crate::content::traits::Content;

macro_rules! to_from_content_id {
    ($e:ident, $t:ident) => {
        impl std::convert::From<ContentID<$t>> for $e {
            fn from(id: ContentID<$t>) -> Self {
                Self::from_id(id)
            }
        }
        impl std::convert::Into<ContentID<$t>> for $e {
            fn into(self) -> ContentID<$t> {
                self.0
            }
        }
    };
}

#[derive(Derivative, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Id(ContentID<Content>);
impl Id {
    fn from_id(id: ContentID<Content>) -> Self {
        Self(id)
    }
}
to_from_content_id!(Id, Content);

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug)]
pub struct ContentRegister<T, P> {
    items: HashMap<ContentID<T>, ContentEntry<T>>,
    generation: u64,

    #[serde(skip_serializing, skip_deserializing, default = "Default::default")]
    #[derivative(Debug = "ignore")]
    _phantom: PhantomData<P>,
}

impl<T, P> ContentRegister<T, P>
where
    P: From<ContentID<T>> + Into<ContentID<T>>,
    T: Debug,
{
    pub fn new() -> Self {
        Self {
            items: Default::default(),
            generation: 0,
            _phantom: PhantomData,
        }
    }

    fn dealloc(&mut self, id: P) -> Option<T> {
        let id: ContentID<T> = id.into();
        self.items.remove(&id).map(|e| e.val)
    }

    pub fn get(&self, id: P) -> Option<&T> {
        let id: ContentID<T> = id.into();
        self.items.get(&id).map(|e| &e.val)
    }

    pub fn get_mut(&mut self, id: P) -> Option<&mut T> {
        let id: ContentID<T> = id.into();
        self.items.get_mut(&id).map(|e| &mut e.val)
    }

    pub fn alloc<I: Into<T>>(&mut self, item: I) -> P {
        let item = item.into();
        let id = self.set(item, self.generation);
        self.generation += 1;
        id
    }

    pub fn register(&mut self, id: P) {
        let id: ContentID<_> = id.into();
        self.items
            .get_mut(&id)
            .expect("can't register if it's not there")
            .id_counter += 1;
    }

    pub fn unregister(&mut self, id: P) -> Option<T> {
        let id: ContentID<_> = id.into();
        let entry = self
            .items
            .get_mut(&id)
            .expect("cant unregister if its not there");
        if entry.id_counter == 0 {
            self.dealloc(id.into())
        } else {
            entry.id_counter -= 1;
            None
        }
    }

    fn set(&mut self, item: T, id: u64) -> P {
        let entry = ContentEntry {
            val: item,
            generation: self.generation,
            id_counter: 1,
        };
        let id = ContentID {
            id,
            generation: self.generation,
            _phantom: PhantomData,
        };
        self.items.insert(id, entry);
        id.into()
    }

    // pub fn get_id_count(&self, index: usize) -> Option<(ContentID<T>, u32)> {
    //     self.items.get(index).map(|e| e.as_ref().map(|e| (
    //         ContentID {
    //             id: index,
    //             generation: e.generation,
    //             _phantom: PhantomData,
    //         },
    //         e.id_counter.into(),
    //     ))).flatten()
    // }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ContentEntry<T> {
    val: T,
    generation: u64,
    id_counter: u32,
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug, Hash)]
pub struct ContentID<T> {
    id: u64,
    generation: u64,

    #[serde(skip_serializing, skip_deserializing, default = "Default::default")]
    #[derivative(Debug = "ignore")]
    #[derivative(Hash = "ignore")]
    _phantom: PhantomData<T>,
}
impl<T> Clone for ContentID<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            generation: self.generation,
            _phantom: PhantomData,
        }
    }
}
impl<T> Copy for ContentID<T> {}
impl<T> PartialEq for ContentID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.generation == other.generation
    }
}
impl<T> Eq for ContentID<T> {}

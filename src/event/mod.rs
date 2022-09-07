#[macro_use]

pub mod client;

use std::{any::Any, mem};

pub use self::{
    client::*
};

pub trait Event: 'static {
    fn ty() -> EventType where Self: Sized;

    fn as_any(&self) -> &dyn Any;
}

#[derive(PartialEq)]
pub enum EventType {
    Ready
}

pub struct Listener {
    pub ty: EventType,
    pub call: fn(&dyn Event, fn(*const ())),
    pub i_call: fn(*const ())
}

impl Listener {
    pub fn new<E: Event>(ty: EventType, callback: fn(&E)) -> Self {
        Self {
            ty,
            call: unsafe {
                mem::transmute(Self::handle::<E> as fn(_,_))
            },
            i_call: unsafe {
                mem::transmute(callback)
            }
        }
    }

    fn handle<E: Event>(event: &dyn Event, call: fn(&E)) {
        let event: &E = event.as_any().downcast_ref().unwrap();
        call(event);
    }
}

macro_rules! impl_event {
    ($t: ty, $et: path) => {
        impl crate::event::Event for $t {
            fn ty() -> EventType {
                $et
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

pub(crate) use impl_event;
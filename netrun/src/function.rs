use std::{any::type_name, sync::Arc};

use log::error;
use parking_lot::Mutex;

type BoxedFn<In, Out> = Box<dyn FnMut(In) -> Out + Send>;

pub struct Function<In, Out> {
    fun: Arc<Mutex<BoxedFn<In, Out>>>,
}

impl<In, Out> Default for Function<In, Out> {
    fn default() -> Self {
        Self {
            fun: Arc::new(Mutex::new(Box::new(|_| {
                error!("Calling empty function of type: {}", type_name::<Self>());
                panic!("Calling empty function of type: {}", type_name::<Self>())
            }))),
        }
    }
}

impl<In, Out> Function<In, Out> {
    pub fn new(fun: impl FnMut(In) -> Out + Send + 'static) -> Self {
        Self {
            fun: Arc::new(Mutex::new(Box::new(fun))),
        }
    }

    pub fn replace(&self, fun: impl FnMut(In) -> Out + Send + 'static) {
        *self.fun.lock() = Box::new(fun);
    }

    pub fn call(&self, input: In) -> Out {
        (*self.fun.lock())(input)
    }
}

impl<In, Out> Clone for Function<In, Out> {
    fn clone(&self) -> Self {
        Self {
            fun: self.fun.clone(),
        }
    }
}

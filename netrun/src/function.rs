use std::{any::type_name, cell::RefCell};

use log::error;

pub struct Function<In, Out> {
    fun: RefCell<Box<dyn FnMut(In) -> Out>>,
}

impl<In, Out> Default for Function<In, Out> {
    fn default() -> Self {
        Self {
            fun: RefCell::new(Box::new(|_| {
                error!("Calling empty function of type: {}", type_name::<Self>());
                panic!("Calling empty function of type: {}", type_name::<Self>())
            })),
        }
    }
}

impl<In, Out> Function<In, Out> {
    pub fn new(fun: impl FnMut(In) -> Out + 'static) -> Self {
        Self {
            fun: RefCell::new(Box::new(fun)),
        }
    }

    pub fn replace(&self, fun: impl FnMut(In) -> Out + 'static) {
        *self.fun.borrow_mut() = Box::new(fun);
    }

    pub fn call(&self, input: In) -> Out {
        (*self.fun.borrow_mut())(input)
    }
}

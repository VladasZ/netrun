use std::{any::type_name, marker::PhantomData, sync::Arc};

use anyhow::Result;
use hreads::spawn;
use log::error;
use zmq::{Context, Message, Socket, SocketType::REP};

use crate::Function;

pub struct Rep<In, Out> {
    receive: Function<In, Out>,
    ctx: Context,
}

impl<In, Out> Rep<In, Out> {
    pub fn new(endpoint: &str) -> Result<Self> {
        let ctx = Context::new();

        let socket = ctx.socket(REP)?;
        socket.connect(endpoint)?;

        spawn(async move {
            loop {
                let bytes = match socket.recv_bytes(0) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        error!("Failed to receive in {}: {err}", type_name::<Self>());
                        continue;
                    }
                };

                if size_of::<In>() != bytes.len() {
                    error!(
                        "Invalid packet size for {}. Expected: {} got: {}",
                        type_name::<Self>(),
                        size_of::<In>(),
                        bytes.len()
                    );

                    let _ = socket.send("Error", 0).inspect_err(|err| {
                        error!("{err}");
                    });

                    continue;
                }
            }
        });

        Ok(Self {
            receive: Function::default(),
            ctx,
        })
    }

    pub fn on_receive(&self, action: impl FnMut(In) -> Out + 'static) {
        self.receive.replace(action);
    }
}

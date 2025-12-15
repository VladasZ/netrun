use std::marker::PhantomData;

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    spawn,
    sync::Mutex,
};

pub type Callback<T> = Box<dyn FnMut(T) + Send>;

pub struct Connection<In, Out> {
    callback: Mutex<Option<Callback<In>>>,
    stream:   Mutex<Option<TcpStream>>,
    write:    Mutex<Option<OwnedWriteHalf>>,
    _p:       PhantomData<Mutex<Out>>,
}

impl<In: DeserializeOwned + Send, Out: Serialize + Send> Connection<In, Out> {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            callback: Mutex::const_new(None),
            stream:   Mutex::new(Some(stream)),
            write:    Mutex::new(None),
            _p:       PhantomData,
        }
    }

    pub async fn start(&'static self) {
        let mut stream = self.stream.lock().await;

        if stream.is_none() {
            return;
        }

        let (read, write) = stream.take().unwrap().into_split();

        let mut wr = self.write.lock().await;

        assert!(wr.is_none(), "Writer already exits");

        wr.replace(write);

        spawn(async move { self.handle_read(read).await.unwrap() });
    }

    pub async fn handle_read(&self, mut reader: OwnedReadHalf) -> Result<()> {
        loop {
            let mut buf = vec![0u8; 4096];

            let n = reader.read(&mut buf).await?;

            if n == 0 {
                continue;
            }

            let json_str = std::str::from_utf8(&buf[..n])?;
            let msg: In = serde_json::from_str(json_str)?;
            self.callback.lock().await.as_mut().unwrap()(msg);
        }
    }

    pub async fn on_receive(&'static self, action: impl FnMut(In) + Send + 'static) -> &'static Self {
        let mut callback = self.callback.lock().await;

        assert!(callback.is_none(), "Already has callback");

        callback.replace(Box::new(action));

        self
    }

    pub async fn send(&'static self, msg: impl Into<Out>) -> Result<()> {
        let msg = msg.into();

        let json = serde_json::to_string(&msg)?;

        let mut writer = self.write.lock().await;
        let writer = writer.as_mut().expect("No writer. Did you start the connection?");

        writer.write_all(json.as_bytes()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tokio::sync::OnceCell;

    use crate::connection::Connection;

    static _CONNECTION: OnceCell<Connection<i32, i32>> = OnceCell::const_new();
}

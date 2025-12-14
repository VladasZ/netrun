use std::{
    fmt::Debug,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    net::TcpListener,
    spawn,
    sync::{Mutex, OnceCell},
};

use crate::connection::{Callback, Connection};

pub struct Server<T> {
    listener:   TcpListener,
    connection: OnceCell<Connection<T>>,
    started:    Mutex<bool>,
    callback:   Mutex<Option<Callback<T>>>,
}

impl<T: Serialize + DeserializeOwned + Send + Debug> Server<T> {
    pub async fn new(port: u16) -> Result<Self> {
        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)).await?;

        Ok(Self {
            listener,
            connection: OnceCell::default(),
            started: Mutex::new(false),
            callback: Mutex::new(None),
        })
    }

    pub async fn start(&'static self) {
        let mut started = self.started.lock().await;

        if *started {
            return;
        }

        spawn(async {
            loop {
                let (stream, addr) = self.listener.accept().await.unwrap();
                println!("Client connected: {addr}");

                assert!(self.connection.get().is_none(), "Connection already exists");

                self.connection
                    .get_or_init(|| async { Connection::new(stream) })
                    .await
                    .on_receive(self.callback.lock().await.take().expect("No callback set"))
                    .await
                    .start()
                    .await;
            }
        });

        *started = true;
    }

    pub async fn on_receive(&'static self, action: impl FnMut(T) + Send + 'static) {
        let mut callback = self.callback.lock().await;

        assert!(callback.is_none(), "Already has callback");

        callback.replace(Box::new(action));
    }

    pub async fn send(&'static self, msg: impl Into<T>) -> Result<()> {
        let Some(connection) = self.connection.get() else {
            dbg!("No connection");
            return Ok(());
        };

        connection.send(msg).await
    }
}

#[cfg(test)]
mod test {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        ops::Deref,
        time::Duration,
    };

    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};
    use tokio::{
        spawn,
        sync::{Mutex, OnceCell},
        time::sleep,
    };

    use crate::{client::Client, server::Server};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        a: i32,
        b: String,
    }

    static SERVER: OnceCell<Server<TestData>> = OnceCell::const_new();

    async fn server() -> &'static Server<TestData> {
        SERVER.get_or_init(|| async { Server::new(55443).await.unwrap() }).await
    }

    static CLIENT: OnceCell<Client<TestData>> = OnceCell::const_new();

    static DATA: Mutex<Vec<TestData>> = Mutex::const_new(Vec::new());

    async fn client() -> &'static Client<TestData> {
        CLIENT
            .get_or_init(|| async {
                Client::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 55443))
                    .await
                    .unwrap()
            })
            .await
    }

    #[tokio::test]
    async fn test_client_server() -> Result<()> {
        let sv = server().await;

        sv.on_receive(|msg| {
            spawn(async { DATA.lock().await.push(msg) });
        })
        .await;

        sv.start().await;

        server()
            .await
            .send(TestData {
                a: 555,
                b: "aaaa".to_string(),
            })
            .await?;

        let cl = client().await;

        cl.start().await;

        cl.on_receive(|msg| {
            spawn(async { DATA.lock().await.push(msg) });
        })
        .await;

        sleep(Duration::from_millis(100)).await;

        server()
            .await
            .send(TestData {
                a: 555,
                b: "aaaa".to_string(),
            })
            .await?;

        client()
            .await
            .send(TestData {
                a: 666,
                b: "aaaa".to_string(),
            })
            .await?;

        sleep(Duration::from_millis(10)).await;

        client()
            .await
            .send(TestData {
                a: 777,
                b: "aaaa".to_string(),
            })
            .await?;

        sleep(Duration::from_millis(200)).await;

        assert_eq!(
            &vec![
                TestData {
                    a: 555,
                    b: "aaaa".to_string(),
                },
                TestData {
                    a: 666,
                    b: "aaaa".to_string(),
                },
                TestData {
                    a: 777,
                    b: "aaaa".to_string(),
                }
            ],
            DATA.lock().await.deref()
        );

        Ok(())
    }
}

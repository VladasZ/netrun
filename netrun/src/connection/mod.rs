const BUFFER_SIZE: usize = 1024 * 8;

mod client;
mod serde;
mod server;

pub use client::*;
pub use server::*;

#[cfg(test)]
mod test {
    use std::{net::Ipv4Addr, sync::Arc, time::Duration};

    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use test_log::test;
    use tokio::{
        spawn,
        sync::{Mutex, OnceCell, mpsc::channel},
        time::sleep,
    };

    use super::*;

    async fn server() -> Result<&'static Server<u32, f32>> {
        static SERVER: OnceCell<Server<u32, f32>> = OnceCell::const_new();

        SERVER
            .get_or_try_init(|| async {
                let s = Server::new(57777).await?;
                Ok(s)
            })
            .await
    }

    #[test(tokio::test)]
    async fn test_connection_and_reconnection() -> Result<()> {
        let data: Arc<Mutex<Vec<u32>>> = Arc::default();

        let (s, mut server_received) = channel::<()>(1);

        assert!(server().await?.connections().await?.is_empty());

        let client: Client<f32, u32> = Client::connect((Ipv4Addr::LOCALHOST, 57777)).await?;

        assert_eq!(client.peer_addr().await?, (Ipv4Addr::LOCALHOST, 57777).into());

        let server_data = data.clone();
        spawn(async move {
            loop {
                let val = server().await.unwrap().receive().await;
                server_data.lock().await.push(val);
                s.send(()).await.unwrap();
            }
        });

        server().await?.wait_for_new_connection().await;

        assert_eq!(
            server().await?.connections().await?.first().unwrap().ip(),
            Ipv4Addr::LOCALHOST
        );

        server().await?.send(0.0042).await?;
        assert_eq!(Some(0.0042), client.receive().await);

        client.send(55u32).await?;

        server_received.recv().await.unwrap();

        assert_eq!(vec![55], **data.lock().await);

        drop(client);

        sleep(Duration::from_secs_f32(0.2)).await;

        server().await?.send(100.0).await?;

        sleep(Duration::from_secs_f32(0.2)).await;

        assert!(server().await?.connections().await?.is_empty());

        server().await?.send(100.0).await?;

        let client: Client<f32, u32> = Client::connect((Ipv4Addr::LOCALHOST, 57777)).await?;
        assert_eq!(client.peer_addr().await?, (Ipv4Addr::LOCALHOST, 57777).into());

        sleep(Duration::from_secs_f32(0.2)).await;

        assert_eq!(
            server().await?.connections().await?.first().unwrap().ip(),
            Ipv4Addr::LOCALHOST
        );

        server().await?.send(0.0042).await?;
        assert_eq!(Some(0.0042), client.receive().await);

        client.send(77u32).await?;
        server_received.recv().await.unwrap();
        assert_eq!(vec![55, 77], **data.lock().await);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_reset_on_double_connection() -> Result<()> {
        async fn double_server() -> Result<&'static Server<u32, f32>> {
            static SERVER: OnceCell<Server<u32, f32>> = OnceCell::const_new();

            SERVER
                .get_or_try_init(|| async {
                    let s = Server::new(57778).await?;
                    Ok(s)
                })
                .await
        }
        let (s, mut server_received) = channel::<()>(1);

        let data: Arc<Mutex<Vec<u32>>> = Arc::default();

        assert!(double_server().await?.connections().await?.is_empty());

        let client: Client<f32, u32> = Client::connect((Ipv4Addr::LOCALHOST, 57778)).await?;

        let server_data = data.clone();
        spawn(async move {
            loop {
                let val = double_server().await.unwrap().receive().await;
                server_data.lock().await.push(val);
                s.send(()).await.unwrap();
            }
        });

        double_server().await?.wait_for_new_connection().await;

        assert_eq!(client.peer_addr().await?, (Ipv4Addr::LOCALHOST, 57778).into());

        assert_eq!(
            *double_server().await?.connections().await?.first().unwrap(),
            client.local_addr().await?
        );

        let client2: Client<f32, u32> = Client::connect((Ipv4Addr::LOCALHOST, 57778)).await?;

        sleep(Duration::from_secs_f32(0.2)).await;

        double_server().await?.send(0.0042).await?;
        assert_eq!(Some(0.0042), client2.receive().await);

        client2.send(77u32).await?;
        server_received.recv().await.unwrap();
        assert_eq!(vec![77], **data.lock().await);

        Ok(())
    }
}

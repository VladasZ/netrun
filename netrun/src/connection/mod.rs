const BUFFER_SIZE: usize = 1024 * 8;

mod client;
mod serde;
mod server;

pub use client::*;
pub use server::*;

#[cfg(test)]
mod test {
    use std::{net::Ipv4Addr, time::Duration};

    use anyhow::Result;
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_connection() -> Result<()> {
        let server: Server<u32, f32> = Server::new(57777).await?;

        let client: Client<f32, u32> = Client::connect((Ipv4Addr::LOCALHOST, 57777)).await?;

        client.send(55u32).await?;

        assert_eq!(server.receive().await, Some(55));

        server.send(0.0042).await?;

        assert_eq!(client.receive().await, Some(0.0042));

        server.dump_connections().await?;

        let client: Client<f32, u32> = Client::connect((Ipv4Addr::LOCALHOST, 57777)).await?;

        sleep(Duration::from_secs_f32(0.1)).await;

        client.send(55u32).await?;

        assert_eq!(server.receive().await, Some(55));

        server.send(0.0042).await?;

        assert_eq!(client.receive().await, Some(0.0042));

        server.dump_connections().await?;

        Ok(())
    }
}

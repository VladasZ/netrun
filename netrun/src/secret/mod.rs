use std::{env::var, time::Duration};

use anyhow::{Context, Result};
use infisical::{AuthMethod, Client, secrets::GetSecretRequest};
use tokio::sync::{OnceCell, watch};

const EU_INFISICAL_URL: &str = "https://eu.infisical.com";

const REQUIRED_VARS: &str = "
  Required environment variables for Infisical:
    INFISICAL_CLIENT_ID      - Universal auth client ID
    INFISICAL_CLIENT_SECRET  - Universal auth client secret
    INFISICAL_PROJECT_ID     - Infisical project ID
    INFISICAL_ENVIRONMENT    - Environment name (e.g. dev, staging, prod)
";

fn require_var(name: &'static str) -> Result<String> {
    var(name).with_context(|| format!("Missing env var: {name}{REQUIRED_VARS}"))
}

static CLIENT: OnceCell<Client> = OnceCell::const_new();

async fn client() -> Result<&'static Client> {
    CLIENT
        .get_or_try_init(|| async {
            let client_id = require_var("INFISICAL_CLIENT_ID")?;
            let client_secret = require_var("INFISICAL_CLIENT_SECRET")?;

            let mut client = Client::builder().base_url(EU_INFISICAL_URL).build().await?;
            client.login(AuthMethod::new_universal_auth(client_id, client_secret)).await?;

            Ok(client)
        })
        .await
}

pub struct Secret {
    key: &'static str,
}

impl Secret {
    pub const fn new(key: &'static str) -> Self {
        Self { key }
    }

    pub async fn get(&self) -> Result<String> {
        let project_id = require_var("INFISICAL_PROJECT_ID")?;
        let environment = require_var("INFISICAL_ENVIRONMENT")?;

        let request = GetSecretRequest::builder(self.key, &project_id, &environment).build();
        let secret = client().await?.secrets().get(request).await?;

        Ok(secret.secret_value)
    }

    pub async fn watch(&'static self) -> Result<watch::Receiver<String>> {
        self.watch_interval(Duration::from_secs(300)).await
    }

    pub async fn watch_interval(&'static self, interval: Duration) -> Result<watch::Receiver<String>> {
        let initial = self.get().await?;
        let (tx, rx) = watch::channel(initial);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                match self.get().await {
                    Ok(value) => {
                        if *tx.borrow() != value {
                            let _ = tx.send(value);
                        }
                    }
                    Err(e) => log::error!("Failed to poll secret {}: {e}", self.key),
                }
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::secret::Secret;

    static TEST_SECRET: Secret = Secret::new("TEST_SECRET");

    #[tokio::test]
    async fn test_secret() -> Result<()> {
        dotenvy::dotenv().ok();

        assert_eq!(TEST_SECRET.get().await?, "plati");

        Ok(())
    }
}

mod method;
mod request;
mod response;
mod rest_api;

pub use method::Method;
pub use request::Request;
pub use response::Response;
pub use rest_api::RestAPI;

#[cfg(test)]
mod test {

    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use serde::Deserialize;

    use crate::rest::{Request, RestAPI};

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct User {
        id:       u32,
        username: String,
        email:    String,
    }

    static USERS: Request<(), Vec<User>> = Request::new("users");

    #[tokio::test]
    async fn test_rest() -> Result<()> {
        RestAPI::init("https://jsonplaceholder.typicode.com/");

        let users = USERS.await?;

        assert_eq!(users.len(), 10);

        Ok(())
    }
}

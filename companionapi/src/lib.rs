pub mod companion_api;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login() {
        let companion_api = companion_api::CompanionAPI::new("some@email.com", "somePassword");
        companion_api.login().await;
    }
}

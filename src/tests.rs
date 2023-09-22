// tests.rs

#[cfg(test)]
mod tests {
    use actix_rt::System;
    use actix_web::{test, App};
    use sqlx::MySqlPool;

    // Import your application state and handlers
    use crate::{AppState, get_aspnet_users}; // Update with your actual module path and handler function

    #[actix_rt::test]
    pub async fn test_get_aspnet_users_success() {
        // Initialize your application state (app_state) here
        let mysql_url: &'static str = "mysql://user:password@127.0.0.1:3306/consume_actix_api";
        let pool = MySqlPool::connect(mysql_url).await.unwrap(); // Use a test database URL
        let app_state = AppState { pool };

        // Create a test server with your app
        let mut app = test::init_service(App::new().data(app_state.clone())).await;

        // Make an HTTP request to your handler (get_aspnet_users)
        let req = test::TestRequest::get().uri("/get-aspnet-users").to_request();
        let resp = test::call_service(&mut app, req).await;

        // Assert that the response status code is 200 OK for the success scenario
        assert!(resp.status().is_success());

        // You can also assert the response body matches the expected success scenario JSON
        // let body_bytes = test::read_body(resp).await;
        // let expected_json = r#"{"users":[{"Id": ... }],"message":"Got all ASP.NET users."}"#;
        // assert_eq!(body_bytes, expected_json);
    }

    // #[actix_rt::test]
    // async fn test_get_aspnet_users_error() {
    //     // Similar structure as the success scenario test
    //     // Configure your app and make an HTTP request to your handler
    //     // Assert that the response status code is 500 Internal Server Error

    //     // For the error scenario, you may want to check the response body for specific error details.
    // }
}

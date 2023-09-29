pub mod models {
    use serde::{Deserialize, Serialize};

    // Define models here (all must be public to allow main to access)

    #[derive(Debug, Serialize, Deserialize)]
    #[allow(non_snake_case)]
    pub struct AspNetUser {
        pub Id: String,
        pub UserName: String,
        pub Email: String,
        pub PasswordHash: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct AspNetUsersResponse {
        pub users: Vec<AspNetUser>,
        pub message: String,
    }

    // Request model to accept user credentials for authentication.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AuthRequest {
        pub username_or_email: String,
        pub password: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AuthResponse {
        pub success: bool,
        pub user: Option<AspNetUser>,
        pub access_token: Option<String>,
        pub refresh_token: Option<String>,
    }

    impl AuthResponse {
        pub fn success(user: AspNetUser, access_token: String, refresh_token: String) -> Self {
            AuthResponse {
                success: true,
                user: Some(user),
                access_token: Some(access_token),
                refresh_token: Some(refresh_token),
            }
        }

        pub fn invalid_credentials() -> Self {
            AuthResponse {
                success: false,
                user: None,
                access_token: None,
                refresh_token: None,
            }
        }
    }
}

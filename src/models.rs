pub mod models {
    use serde::{ Serialize, Deserialize };

    // Define models here (all must be public ! in order for main to access)

    #[allow(non_snake_case)]
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AspNetUser {
        pub Id: String,
        pub UserName: String,
        pub Email: String,
        pub PasswordHash: String,
    }

    
    #[allow(non_snake_case)]
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
    pub enum AuthResult {
        Success(AspNetUser), // Authentication succeeded
        InvalidCredentials, // Invalid username/email or password
        DatabaseError(String), // Database error
    }
}
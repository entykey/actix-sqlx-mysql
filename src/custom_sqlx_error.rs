pub mod custom_sqlx_error {
    // use actix_web::HttpResponse;
    use ntex::web::{HttpResponse};
    use serde::Serialize;
    use sqlx::mysql::MySqlDatabaseError;
    use sqlx::Error as SqlxError;
    use std::fmt;

    // Custom struct for serializing SQLx errors
    #[derive(Debug, Serialize)] // Derive the Serialize trait for JSON serialization
    pub struct SqlxErrorResponse {
        pub code: Option<String>,
        pub message: String,
    }

    // Define a custom error type
    #[derive(Debug)]
    pub enum CustomError {
        Sqlx(SqlxError),
        Database(MySqlDatabaseError),
        NotFound,
    }

    // Implement From for SqlxError
    impl From<SqlxError> for CustomError {
        fn from(error: SqlxError) -> Self {
            CustomError::Sqlx(error)
        }
    }

    // Implement From for MySqlDatabaseError
    impl From<MySqlDatabaseError> for CustomError {
        fn from(error: MySqlDatabaseError) -> Self {
            CustomError::Database(error)
        }
    }

    impl fmt::Display for CustomError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                CustomError::Sqlx(error) => write!(f, "SQLx error: {}", error),
                CustomError::Database(db_error) => {
                    write!(f, "Database error: {}", db_error.message())
                }
                CustomError::NotFound => write!(f, "No account exists with the given credentials."),
            }
        }
    }

    impl CustomError {
        pub fn to_http_response(&self) -> HttpResponse {
            match self {
                CustomError::Sqlx(error) => {
                    // Extract the error code from the SqlxError, if available:
                    let (code, message) = match error {
                        SqlxError::Database(db_error) => (
                            db_error
                                .code()
                                .map_or_else(|| "Unknown".to_string(), |code| code.to_string()),
                            db_error.message().to_string(),
                        ),

                        // For other SQLx errors, set the code to "Unknown".
                        _ => ("Unknown".to_string(), error.to_string()),
                    };

                    // Custom Error Response
                    let sqlx_error_response: SqlxErrorResponse = SqlxErrorResponse {
                        code: Some(code),
                        message,
                    };

                    // Serialize the custom response as JSON
                    HttpResponse::InternalServerError().json(&sqlx_error_response)
                }
                CustomError::Database(db_error) => {
                    // Customize the response based on the database error.
                    HttpResponse::InternalServerError()
                        .json(&format!("Database error: {}", db_error.message()))
                }
                CustomError::NotFound => {
                    HttpResponse::NotFound().json(&"No account exists with the given credentials.")
                }
            }
        }
    }
}

pub mod custom_error {
    use std::fmt;
    use actix_web::HttpResponse;
    use serde::Serialize;
    use sqlx::mysql::MySqlDatabaseError;
    use sqlx::Error as SqlxError;

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
                CustomError::Database(db_error) => write!(f, "Database error: {}", db_error.message()),
                CustomError::NotFound => write!(f, "No account exists with the given credentials."),
            }
        }
    }

    impl CustomError {
        pub fn to_http_response(&self) -> HttpResponse {
            match self {
                CustomError::Sqlx(error) => {
                    // (Extracting Error Code: In your original code, you were trying to call the
                    // code method on the SqlxError type. However, it appears that code is not a method directly available on SqlxError.
                    // To extract the error code from a SqlxError, we need to do some pattern matching to check if
                    // the error is actually a SqlxError::Database variant, which provides access to the underlying
                    // database-specific error (in this case, a MySqlDatabaseError).)

                    // Extract the error code from the SqlxError, if available:
                    let code: Option<std::borrow::Cow<'_, str>> = match error {
                        SqlxError::Database(db_error) => db_error.code(),

                        // Here, we use a match statement to check the type of the error variable. If it's a SqlxError::Database, we extract the error code using db_error.code(). If it's not a database error, we set the code to None.
                        _ => None,
                    };

                    // Custom Error Response: Once we have extracted the error code (if available),
                    // we create a custom error response struct SqlxErrorResponse that includes both the error code and message.

                    // Serialize the SQLx error as an object:
                    let sqlx_error_response: SqlxErrorResponse = SqlxErrorResponse {
                        // Here, we use the map function to convert the optional code (which could be Some(code) or None) to a String. This allows us to include the error code in the response as a string.
                        code: code.map(|code: std::borrow::Cow<'_, str>| code.to_string()),
                        message: error.to_string(),
                    };

                    // Serialize the custom response as JSON
                    HttpResponse::InternalServerError().json(sqlx_error_response)
                }
                CustomError::Database(db_error) => {
                    // Customize the response based on the database error.
                    HttpResponse::InternalServerError()
                        .json(format!("Database error: {}", db_error.message(),))
                }
                CustomError::NotFound => {
                    HttpResponse::NotFound().json("No account exists with the given credentials.")
                }
            }
        }
    }
}

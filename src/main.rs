// total rust project size: 1.37GB on Windows 10;  1.36GB MacOS Monterey    |
// Intel(R) Core(TM) i5-4460  CPU @ 3.20GHz   3.20 GHz |
// Mem: 4.00 GB (3.88 GB usable)    |
// C: 99,6GB free of 165 GB    |   D: 299 GB (empty)

// Note: to fix the ERR of mysql: "Column count of mysql.proc is wrong. Expected 20, found 16. The table is probably corrupted"
// Run: $ sudo /Applications/XAMPP/xamppfiles/bin/mysql_upgrade

use actix_web::{web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySqlConnection, MySqlPool, MySqlPoolOptions, MySqlQueryResult, MySqlRow};
use sqlx::{Connection, FromRow, Row};
use sqlx::{Error as SqlxError, mysql::MySqlDatabaseError};
use uuid::Uuid;

// define modules & import
mod models; // Create a new module named "models" by convention
use models::models::{AspNetUser, AspNetUsersResponse,
    AuthRequest, AuthResult}; // Specify the correct module path

mod hasher;
use hasher::hasher::{verify_password_with_sha256_with_salt};

#[derive(Clone)]
struct AppState {
    pool: MySqlPool,
}

/*
//  NULLABLE email (NULL by default )
#[derive(Debug, Serialize, Deserialize)]
struct User {
    //id: i32,
    id: String,
    username: String,
    email: Option<String>,  // (MySQL col settings:  NULL = Yes, default = NULL) (accept NULL)
                            //  If non-NULLABLE (NULL = No) => error
}
*/
// Non-NULLABLE email, (empty string by default)
#[derive(Debug, Serialize, Deserialize)]
struct User {
    //id: i32,
    id: String,
    username: String,
    email: String, // (MySQL col settings:  NULL = No, default = None) (accept Non-NULL)
                   //  If non-NULLABLE (NULL = yes, default = NULL) => error
}

// Define User struct with lifetimes for references
// #[derive(Debug)]
// struct User<'a> {
//     id: &'a str,
//     username: &'a str,
//     email: &'a str, // Use Option<&str> for the email field
// }

#[derive(Serialize, Deserialize)]
struct Response {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct UserResponse {
    user: User,
    message: String,
}

// Define UserResponse struct with lifetimes for references
// #[derive(Debug)]
// struct UserResponse<'a> {
//     user: User<'a>,
//     message: &'static str,
// }

#[derive(Serialize, Deserialize)]
struct UsersResponse {
    users: Vec<User>,
    message: String,
}

// Define UserResponse struct with lifetimes for references
// #[derive(Debug)]
// struct UsersResponse<'a> {
//     users: Vec<User<'a>>,
//     message: &'static str,
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // let _database_url: String = env::var("DATABASE_URL").unwrap();
    const DATABASE_URL: &str = "mysql://user:password@127.0.0.1:3306/consume_actix_api"; // "mysql://user:password@127.0.0.1:3306/actix_sqlx"

    /* Connecting to a database
     * for single connection:
     * [MySql|Sqlite|PgConnection...]Connection::connect()
     *
     * for pool connection:
     * [MysqlPool|...]::connect()
     *
     * custom pool connection:
     * [MysqlPool|...]Options::new().connect()
     */
    let pool: MySqlPool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(DATABASE_URL)
        .await
        .unwrap();

    let app_state = AppState { pool };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(root))

            // AspNet Identity (other database):
            .route("/get-aspnet-users", web::get().to(get_aspnet_users))
            .route("/auth", web::post().to(authenticate_user))
    })
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}

async fn root() -> HttpResponse {
    HttpResponse::Ok().json(Response {
        message: "Server is up and running.".to_string(),
    })
}



/*
async fn get_user(path: web::Path<i32>, app_state: web::Data<AppState>) -> HttpResponse {
    let user_id: i32 = path.into_inner();
    /* Queries
     * prepared (parameterized):
     *   have their quey plan cached, use a
     *   binary mode of communication (lower
     *   bandwith and faster decoding), and
     *   utilize parameters to avoid SQL
     *   Injection
     * unprepared (simple):
     *   intended only for use case where
     *   prepared  statement will not work
     *
     * &str is treated as an unprepared query
     * Query or QueryAs struct is treated as
     * prepared query
     *
     *  conn.execute("BEGIN").await                            <- unprepared
     *  conn.execute(sqlx::query("DELETE FROM table")).await   <- prepared
     *
     * All methods accept one of &mut {connection type}, &mut Transaction or &Pool
     *
     * sqlx::query(""); // Query
     * sqlx::query_as(""); // QueryAs
     * sqlx::query("QUERY").fetch_one(&pool).await // <- sqlx::Result<MySqlRow>
     * sqlx::query_as("QUERY").fetch_one(&pool).await // <- sqlx::Result<User> which derives FromRow
     */

    /* sqlx::query Example:
     * let user: sqlx::Result<MySqlRow> = sqlx::query("").bind().fetch().await
     */

    /* sqlx::query_as Example:
     * #[derive(sqlx::FromRow)]
     * struct UserRow {
     *     id: i32,
     *     email: String,
     *     username: String,
     * }
     *
     * let user_0: sqlx::Result<UserRow> = sqlx::query_as("SELECT * FROM users WHERE id=?")
     *    .bind(user_id)
     *    .fetch_one(&app_state.pool)
     *    .await;
     */

    /*
    let updated: sqlx::Result<MySqlQueryResult> = sqlx::query!(
        "DROP TABLE users",
    ).execute(&app_state.pool).await;
    */

    let user: Result<User, sqlx::Error> = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id=?",
        user_id
    ).fetch_one(&app_state.pool).await;
    // fetch                   Stream                               call .try_next()
    // fetch_optional  .await  sqlx::Result<Option<T>>              extra rows are ignored
    // fetch_all       .await  sqlx::Result<Vec<T>>
    // fetch_one       .await  sqlx::Result<T>                      error if no rows, extra rows are ignored
    // execute         .await  sqlx::Result<Database::QueryResult>  for INSERT/UPDATE/DELETE without returning

    if user.is_err() {
        return HttpResponse::BadRequest().json(Response {
            message: "No user found with given id.".to_string()
        });
    }

    HttpResponse::Ok().json(UserResponse {
        user: user.unwrap(),
        message: "Got user.".to_string(),
    })
}
*/




// 3ms - 10ms (Postman)
// works fine for NON-NULLABLE email        (MySQL col settings:  NULL = No, default = None) (accept email='')
// panic on NULLABLE                        (MySQL col settings:  NULL = Yes, default = NULL) (accept email=NULL)
// async fn get_all_users(app_state: web::Data<AppState>) -> HttpResponse {
//     // timer
//     let time = std::time::Instant::now();

//     // Fetch users, including those with NULL email
//     let mut users: Vec<User> = sqlx::query_as!(
//         User,
//         "SELECT * FROM users",
//     )
//     .fetch_all(&app_state.pool)
//     .await
//     .unwrap();

//     // Modify the email field directly within the User struct
//     // for user in users.iter_mut() {
//     //     if user.email.is_empty() {
//     //         user.email = "Not Provided".to_string();
//     //     }
//     // }

//     // Modify the email field directly within the User struct using functional style
//     // More performance than for loop (tested with 5 duplications of looping code)
//     users.iter_mut().for_each(|user| {
//         if user.email.is_empty() {
//             user.email = "Not Provided".to_string();    // Map empty email to "null" string
//         }
//     });

//     // stop timer & print to terminal
//     let duration = time.elapsed();
//     let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
//     let elapsed_seconds = elapsed_ms / 1000.0;
//     println!("query time: {:?} ({:?} ms) ({:.8} s)", duration, elapsed_ms, elapsed_seconds);

//     // Response
//     HttpResponse::Ok().json(UsersResponse {
//         users,
//         message: "Got all users.".to_string(),
//     })
// }



// lifetime trial - failed, near sqlx::... expected &str, but found String
// async fn get_all_users(app_state: web::Data<AppState>) -> HttpResponse {
//     // timer
//     let time = std::time::Instant::now();

//     // Fetch users, including those with NULL email
//     let users: Vec<User> = sqlx::query_as!(
//         User,
//         "SELECT * FROM users",
//     )
//     .fetch_all(&app_state.pool)
//     .await
//     .unwrap();

//     // Modify the email field directly within the User struct using lifetimes
//     let users_with_updated_email: Vec<User> = users
//         .iter()
//         .map(|user| User {
//             id: user.id,
//             username: user.username,
//             email: if let Some(email) = user.email {
//                 if email.is_empty() {
//                     Some("Not Provided")
//                 } else {
//                     Some(email)
//                 }
//             } else {
//                 Some("Not Provided")
//             },
//         })
//         .collect();

//     // stop timer & print to terminal
//     let duration = time.elapsed();
//     let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
//     let elapsed_seconds = elapsed_ms / 1000.0;
//     println!("query time: {:?} ({:?} ms) ({:.8} s)", duration, elapsed_ms, elapsed_seconds);

//     // Response
//     HttpResponse::Ok().json(UsersResponse {
//         users: users_with_updated_email,
//         message: "Got all users.",
//     })
// }






// Custom struct for serializing SQLx errors
#[derive(Debug, Serialize)] // Derive the Serialize trait for JSON serialization
struct SqlxErrorResponse {
    code: Option<String>,
    message: String,
}
// Define a custom error type
#[derive(Debug)]
enum CustomError {
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

impl CustomError {
    fn to_http_response(&self) -> HttpResponse {
        match self {
            // CustomError::Sqlx(error) => {
            //     HttpResponse::InternalServerError().json(format!(
            //         "SQLx error: {}",
            //         error.clone()
            //     ))
            // }


            // CustomError::Sqlx(error) => {
            //     // Serialize the SQLx error as an object
            //     let sqlx_error_response = SqlxErrorResponse {
            //         //code: error.code().map(|code| code.to_string()),
            //         code: Some("trying it..".to_string(),
            //         message: error.to_string(),
            //     };

            //     // Serialize the custom response as JSON
            //     HttpResponse::InternalServerError().json(sqlx_error_response)
            // }

            CustomError::Sqlx(error) => {
                // (Extracting Error Code: In your original code, you were trying to call the
                // code method on the SqlxError type. However, it appears that code is not a method directly available on SqlxError.
                // To extract the error code from a SqlxError, we need to do some pattern matching to check if 
                // the error is actually a SqlxError::Database variant, which provides access to the underlying
                // database-specific error (in this case, a MySqlDatabaseError).)

                // Extract the error code from the SqlxError, if available:
                let code = match error {
                    SqlxError::Database(db_error) => db_error.code(),

                    // Here, we use a match statement to check the type of the error variable. If it's a SqlxError::Database, we extract the error code using db_error.code(). If it's not a database error, we set the code to None.
                    _ => None,
                };

                // Custom Error Response: Once we have extracted the error code (if available),
                // we create a custom error response struct SqlxErrorResponse that includes both the error code and message.
                
                // Serialize the SQLx error as an object:
                let sqlx_error_response = SqlxErrorResponse {
                    // Here, we use the map function to convert the optional code (which could be Some(code) or None) to a String. This allows us to include the error code in the response as a string.
                    code: code.map(|code| code.to_string()),
                    message: error.to_string(),
                };

                // Serialize the custom response as JSON
                HttpResponse::InternalServerError().json(sqlx_error_response)
            }
            CustomError::Database(db_error) => {
                // Customize the response based on the database error.
                HttpResponse::InternalServerError().json(format!(
                    "Database error: {}",
                    db_error.message(),
                ))
            }
            CustomError::NotFound => {
                HttpResponse::NotFound().json("No account exists with the given credentials.")
            }
        }
    }
}

// Define the private function to fetch ASP.NET users (Repository)
async fn fetch_aspnet_users(pool: &MySqlPool) -> Result<Vec<AspNetUser>, sqlx::Error> {
    let users: Vec<AspNetUser> =
        sqlx::query("SELECT u.Id, u.UserName, u.Email, u.PasswordHash FROM AspNetUsers u")
            .map(|user: sqlx::mysql::MySqlRow| {
                AspNetUser {
                    Id: user.get(0), // must add 'use sqlx::Row' !!
                    UserName: user.get(1),
                    Email: user.get(2),
                    PasswordHash: user.get(3),
                }
            })
            .fetch_all(pool)
            .await?;

    Ok(users)
}

// HTTP handler for getting al AspNetUser (controller)
async fn get_aspnet_users(app_state: web::Data<AppState>) -> HttpResponse {
    // timer
    let time = std::time::Instant::now();

    // Fetch ASP.NET users using the private function
    let users_result = fetch_aspnet_users(&app_state.pool).await;

    // Handle the result or return an error response
    match users_result {
        Ok(users) => {
            // stop timer & print to terminal
            let duration = time.elapsed();
            let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
            let elapsed_seconds = elapsed_ms / 1000.0;
            println!(
                "query time: {:?} ({:?} ms) ({:.8} s)",
                duration, elapsed_ms, elapsed_seconds
            );

            // Response
            HttpResponse::Ok().json(AspNetUsersResponse {
                users,
                message: "Got all ASP.NET users.".to_string(),
            })
        }
        // Err(err) => {
        //     // Handle the error and return an error response
        //     eprintln!("Error fetching ASP.NET users: {:?}", err);
        //     HttpResponse::InternalServerError().json(AspNetUsersResponse {
        //         users: Vec::new(),
        //         message: "Failed to fetch ASP.NET users.".to_string(),
        //     })
        // }
        Err(err) => {
            // Wrap the error and return an error response
            eprintln!("Error fetching ASP.NET users: {:?}", err);

            let custom_err: CustomError = err.into();
            custom_err.to_http_response()
        }
    }
}

/* test run: (MacOS Monterey, Intel i5 gen 7)

// success result:
{"users":[{"Id":"d60449d4-f1c2-43e9-a62f-ae087357fa05","UserName":"nguyentuan8a10ntk@gmail.com","Email":"nguyentuan8a10ntk@gmail.com","PasswordHash":"AQAAAAIAAYagAAAAEKxpBdIrGR6M67pLiiKJA1Jr9LRGHQ8/fln+oHWBvk96wsC4gatTOqyU6zyr76naZw=="}],"message":"Got all ASP.NET users."}

// server error result (wrong table name):
{"code":"42S02","message":"error returned from database: 1146 (42S02): Table 'consume_actix_api.aspnetuses' doesn't exist"}
*/





// Define a function to fetch one user by username or email.
async fn fetch_one_aspnet_user(
    app_state: web::Data<AppState>,
    username_or_email: &str,
) -> Result<Option<AspNetUser>, sqlx::Error> {
    let user: Result<Option<AspNetUser>, sqlx::Error> = match sqlx::query(
        "SELECT u.Id, u.UserName, u.Email, u.PasswordHash FROM AspNetUsers u WHERE u.UserName = ? OR u.Email = ?",
    )
    .bind(username_or_email)
    .bind(username_or_email)
    .fetch_one(&app_state.pool)
    .await
    {
        Ok(row) => Ok(Some(AspNetUser {
            Id: row.get(0),
            UserName: row.get(1),
            Email: row.get(2),
            PasswordHash: row.get(3),
        })),

        // if enter invalid email, it cause RowNotFound error, so we'll handle it from here
        Err(sqlx::Error::RowNotFound) => Ok(None), // Handle RowNotFound here

        // catching other errors here
        Err(e) => Err(e),
    };

    user
}

// Handler for user authentication.
async fn authenticate_user(
    app_state: web::Data<AppState>,
    auth_request: web::Json<AuthRequest>, // Use the request model.
) -> HttpResponse {
    // timer
    let time = std::time::Instant::now();

    match fetch_one_aspnet_user(app_state, &auth_request.username_or_email).await {
        // Ok(Some(user)) => {
        //     // stop timer & print to terminal
        //     let duration = time.elapsed();
        //     let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
        //     let elapsed_seconds = elapsed_ms / 1000.0;
        //     println!(
        //         "query time: {:?} ({:?} ms) ({:.8} s)",
        //         duration, elapsed_ms, elapsed_seconds
        //     );


        //     // User found, return the user.
        //     HttpResponse::Ok().json(AspNetUsersResponse {
        //         users: vec![user],
        //         message: "Found user.".to_string(),
        //     })
        // }

        Ok(Some(user)) => {
            // Verify the password
            if verify_password_with_sha256_with_salt(
                &auth_request.password,
                &user.PasswordHash,
            ) {
                // Stop timer & print to terminal
                let duration = time.elapsed();
                let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
                let elapsed_seconds = elapsed_ms / 1000.0;
                println!(
                    "query time: {:?} ({:?} ms) ({:.8} s)",
                    duration, elapsed_ms, elapsed_seconds
                );

                // User found and password is correct, return the user.
                HttpResponse::Ok().json(AuthResult::Success(user))
            } else {
                // Password is incorrect
                HttpResponse::Ok().json(AuthResult::InvalidCredentials)
            }
        }
        Ok(None) => {
            // stop timer & print to terminal
            let duration = time.elapsed();
            let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
            let elapsed_seconds = elapsed_ms / 1000.0;
            println!(
                "query time: {:?} ({:?} ms) ({:.8} s)",
                duration, elapsed_ms, elapsed_seconds
            );


            // User not found, return an empty list and appropriate message.
            HttpResponse::Ok().json(AspNetUsersResponse {
                users: vec![],
                message: "No account exists with the given credentials.".to_string(),
            })
        }
        // Err(err) => {
        //     // Handle the error, you can return an internal server error or customize it as needed.
        //     println!("Hey, Caught An Error: {:?}", err);
        //     HttpResponse::InternalServerError().finish()
        // }
        Err(err) => {
            // Wrap the error and return an error response
            eprintln!("Error fetching ASP.NET user: {:?}", err);

            let custom_err: CustomError = err.into();
            custom_err.to_http_response()
        }
    }
}





/*
#[derive(Serialize, Deserialize)]
struct CreateUserBody {
    username: String,
    email: String
}

// 6ms (Postman)
async fn create_user(body: web::Json<CreateUserBody>, app_state: web::Data<AppState>) -> HttpResponse {
    let new_uuid = Uuid::new_v4().to_string();

    let created: Result<MySqlQueryResult, sqlx::Error> = sqlx::query!(
        "INSERT INTO users(id, username, email) VALUES(?, ?, ?)",
        new_uuid,   // before converting to_string() -> error[E0277]: the trait `Type<MySql>` is not implemented for `Uuid`
        body.username,
        body.email,
    ).execute(&app_state.pool).await;

    if created.is_err() {
        println!("{}", created.unwrap_err());
        return HttpResponse::InternalServerError().json(Response {
            message: "Couldn't create a new user.".to_string(),
        });
    }

    HttpResponse::Ok().json(Response {
        message: "Created a user.".to_string(),
    })
}

#[derive(Serialize, Deserialize)]
struct PatchUserBody {
    id: i32,
    username: Option<String>,
    email: Option<String>,
}

async fn patch_user(body: web::Json<PatchUserBody>, app_state: web::Data<AppState>) -> HttpResponse {
    /* Patch username */
    if body.username.is_some() {
        let patch_username: Result<MySqlQueryResult, sqlx::Error> = sqlx::query!(
            "UPDATE users SET username = ? WHERE id = ?",
            body.username.as_ref().unwrap(),
            body.id,
        ).execute(&app_state.pool).await;

        if patch_username.is_err() {
            return HttpResponse::InternalServerError().json(Response {
                message: "Couldn't patch username.".to_string(),
            });
        }
    }

    /* Patch email */
    if body.email.is_some() {
        let patch_email: Result<MySqlQueryResult, sqlx::Error> = sqlx::query!(
            "UPDATE users SET email = ? WHERE id = ?",
            body.email.as_ref().unwrap(),
            body.id,
        ).execute(&app_state.pool).await;

        if patch_email.is_err() {
            return HttpResponse::InternalServerError().json(Response {
                message: "Couldn't patch email.".to_string(),
            });
        }
    }

    HttpResponse::Ok().json(Response {
        message: "Updated the user.".to_string(),
    })
}

#[derive(Serialize, Deserialize)]
struct DeleteUserBody {
    //id: i32,
    id: String,
}

// 10ms (Postman)
async fn delete_user(body: web::Json<DeleteUserBody>, app_state: web::Data<AppState>) -> HttpResponse {
    let deleted: Result<MySqlQueryResult, sqlx::Error> = sqlx::query!(
        "DELETE FROM users WHERE id=?",
        body.id,
    ).execute(&app_state.pool).await;

    if deleted.is_err() {
        println!("{}", deleted.unwrap_err());
        return HttpResponse::InternalServerError().json(Response {
            message: "Couldn't delete the user.".to_string(),
        });
    }

    HttpResponse::Ok().json(Response {
        message: "Deleted the user.".to_string(),
    })
}
*/

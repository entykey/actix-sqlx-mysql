// total rust project size: 1.37GB on Windows 10;  1.36GB on MacOS Monterey    |
// Intel(R) Core(TM) i5-4460  CPU @ 3.20GHz   3.20 GHz |
// Mem: 4.00 GB (3.88 GB usable)    |
// C: 99,6GB free of 165 GB    |   D: 299 GB (empty)

// Note: to fix the ERR of mysql: "Column count of mysql.proc is wrong. Expected 20, found 16. The table is probably corrupted"
// Run: $ sudo /Applications/XAMPP/xamppfiles/bin/mysql_upgrade

use ntex::util::HashMap;
// use actix_web::{web, App, HttpResponse, HttpServer};
use ntex::web::{self, App, HttpRequest, HttpResponse};
use futures::stream::StreamExt; // for using the .next() method
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::mysql::{MySqlConnection, MySqlPool, 
    MySqlPoolOptions, MySqlDatabaseError, MySqlQueryResult, MySqlRow};
use sqlx::Row;
use uuid::Uuid;
use rayon::prelude::*; // Import Rayon's prelude

// define modules & import
mod models; // Create a new module named "models" by convention
use models::models::{AspNetUser, AspNetUsersResponse,
    AuthRequest, AuthResponse, AspNetUserWithRoles}; // Specify the correct module path

mod hasher;
use hasher::hasher::{verify_password_with_sha256_with_salt};

mod custom_sqlx_error;
use custom_sqlx_error::custom_sqlx_error::{SqlxErrorResponse, CustomError};

use crate::models::models::AspNetUsersWithRolesResponse;


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
// #[derive(Debug, Serialize, Deserialize)]
// struct User {
//     //id: i32,
//     id: String,
//     username: String,
//     email: String, // (MySQL col settings:  NULL = No, default = None) (accept Non-NULL)
//                    //  If non-NULLABLE (NULL = yes, default = NULL) => error
// }

// Non-NULLABLE email
#[derive(Debug, Serialize, Deserialize)]
struct User<'a> {
    id: &'a str,
    username: &'a str,
    email: &'a str, // (MySQL col settings:  NULL = No, default = None) (accept Non-NULL)
                    //  If non-NULLABLE (NULL = yes, default = NULL) => error
}

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    message: String,
}

// Define UserResponse struct with lifetimes for references
#[derive(Debug)]
struct UserResponse<'a> {
    user: User<'a>,
    message: &'static str,
}

// Define UserResponse struct with lifetimes for references
#[derive(Debug)]
struct UsersResponse<'a> {
    users: Vec<User<'a>>,
    message: &'static str,
}

// #[actix_web::main]
#[ntex::main]
async fn main() -> std::io::Result<()> {

    // let _database_url: String = env::var("DATABASE_URL").unwrap();
    const DATABASE_URL: &str = "mysql://user:password@127.0.0.1:3306/consume_actix_api"; // "mysql://user:password@127.0.0.1:3306/actix_sqlx"
    const MAX_DB_RETRIES: u32 = 5; // Maximum number of connection retries
    const RETRY_INTERVAL_SECS: u64 = 5; // Interval between retries in seconds

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


    // Log that the API is starting to establish a database connection
    println!("⌛️ Starting Server, establishing database connection...");

    let mut retries: u32 = 0;

    while retries < MAX_DB_RETRIES {
        // create connection pool
        match MySqlPoolOptions::new()
            .max_connections(10)
            .connect(DATABASE_URL)
            .await
        {
            Ok(pool) => {
                let app_state: AppState = AppState { pool };

                // Start the Actix server with the established database connection
                println!("✅ Database connection established successful! Starting Server...");

                // actix_web server
                // let server = HttpServer::new(move || {
                //     App::new()
                //         // Allow all origins, methods, request headers and exposed headers allowed. Credentials supported. Max age 1 hour. Does not send wildcard.
                //         .wrap(Cors::permissive())

                //         .app_data(web::Data::new(app_state.clone()))

                //         .route("/", web::get().to(root))
                //         .route("/pool-info", web::get().to(get_pool_info))

                //         // AspNet Identity (other database):
                //         .route("/get-aspnet-users", web::get().to(get_aspnet_users))
                //         .route("/auth", web::post().to(authenticate_user))
                // })
                // .bind(("127.0.0.1", 4000));
                
                // ntex tokio server
                let server = web::server(move || {
                    App::new()
                        // Allow all origins, methods, request headers and exposed headers allowed. Credentials supported. Max age 1 hour. Does not send wildcard.
                        // NO cors yet (ntex)
                        // .wrap(Cors::permissive())

                        .state(app_state.clone())

                        .route("/", web::get().to(root))
                        // .service(web::resource("/").to(root))    // also works
                        // .route("/pool-info", web::get().to(get_pool_info))
                        .service(web::resource("/").to(get_pool_info))

                        // AspNet Identity (other database):
                        // .route("/get-aspnet-users", web::get().to(get_aspnet_users))
                        .service(web::resource("/get-aspnet-users").to(get_aspnet_users))
                        .route("/get-aspnet-users-with-roles", web::get().to(get_aspnet_users_with_roles))

                        // .route("/auth", web::post().to(authenticate_user))
                        .service(web::resource("/auth").to(authenticate_user))
                })
                .bind(("127.0.0.1", 4000));

                match server {
                    Ok(server) => {
                        // Print the success message after the server starts
                        println!("🚀 Server is up and listening at localhost:4000");

                        // Start the server
                        if let Err(e) = server.run().await {
                            println!("❌ Server error: {}", e);
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to bind server: {}", e);
                        return Err(e);
                    }
                }

                return Ok(());
                
            }
            Err(e) => {
                // Log the error and wait before retrying
                eprintln!("❌ Failed to connect to the database: {}", e);
                retries += 1;

                if retries < MAX_DB_RETRIES {
                    println!("⌛️ Retrying in {} seconds...", RETRY_INTERVAL_SECS);
                    std::thread::sleep(std::time::Duration::from_secs(RETRY_INTERVAL_SECS));
                } else {
                    eprintln!("❌ Max connection retries reached. Exiting...");
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "❌ Failed to connect to the database",
                    ));
                }
            }
        }
    }

    Ok(())
}


async fn root() -> HttpResponse {
    HttpResponse::Ok().json(&ApiResponse {
        message: "✅ Server is up and running.".to_string(),
    })
}

// async fn get_pool_info(app_context: web::Data<AppState>, req: actix_web::HttpRequest) -> HttpResponse {
async fn get_pool_info(app_context: web::types::State<AppState>, req: HttpRequest) -> HttpResponse {
    let app_context_address : String = format!("{:p}", app_context.get_ref());
    let pool_address: String = format!("{:p}", &app_context.pool);
    let request_address: String = format!("{:p}", &req);

    HttpResponse::Ok().json(&json!({
        "app_context_address": app_context_address,
        "pool_address": pool_address,
        "request_address": request_address,
    }))
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






// Define the private function to fetch ASP.NET users (Repository)
// async fn fetch_aspnet_users(pool: &MySqlPool) -> Result<Vec<AspNetUser>, sqlx::Error> {
//     let users: Vec<AspNetUser> =
//         sqlx::query("SELECT u.Id, u.UserName, u.Email, u.PasswordHash FROM AspNetUsers u")
//             .map(|user: sqlx::mysql::MySqlRow| {
//                 AspNetUser {
//                     Id: user.get(0), // must add 'use sqlx::Row' !!
//                     UserName: user.get(1),
//                     Email: user.get(2),
//                     PasswordHash: user.get(3),
//                 }
//             })
//             .fetch_all(pool)
//             .await?;

//     Ok(users)
// }

// Rayon version, reduced 2-4ms
async fn fetch_aspnet_users(pool: &MySqlPool) -> Result<Vec<AspNetUser>, sqlx::Error> {
    let users: Vec<AspNetUser> =
        sqlx::query("SELECT u.Id, u.UserName, u.Email, u.PasswordHash
                FROM AspNetUsers u"
        )
            .fetch_all(pool)
            .await?
            .into_par_iter() // Convert to parallel iterator
            .map(|user: sqlx::mysql::MySqlRow| {
                AspNetUser {
                    Id: user.get(0),
                    UserName: user.get(1),
                    Email: user.get(2),
                    PasswordHash: user.get(3),
                }
            })
            .collect(); // Collect results back into a Vec

    Ok(users)
}

// HTTP handler for getting al AspNetUser (controller)
async fn get_aspnet_users(
    // app_state: web::Data<AppState>       // actix_web
    app_state: web::types::State<AppState>  // ntex
) -> HttpResponse {

    // timer
    let time: std::time::Instant = std::time::Instant::now();

    // Fetch ASP.NET users using the private function
    let users_result: Result<Vec<AspNetUser>, sqlx::Error> = fetch_aspnet_users(&app_state.pool).await;

    // Handle the result or return an error response
    match users_result {
        Ok(users) => {  // : Vec<AspNetUser>
            // stop timer & print to terminal
            let duration: std::time::Duration = time.elapsed();
            let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
            let elapsed_seconds: f64 = elapsed_ms / 1000.0;
            println!(
                "query time: {:?} ({:?} ms) ({:.8} s)",
                duration, elapsed_ms, elapsed_seconds
            );

            // Response
            HttpResponse::Ok().json(&AspNetUsersResponse {
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

// err case: database error (wrong table name):
{"code":"42S02","message":"error returned from database: 1146 (42S02): Table 'consume_actix_api.aspnetuses' doesn't exist"}

// err case: database service terminated at runtime (server still running):
{"code":null,"message":"pool timed out while waiting for an open connection"}
*/


async fn fetch_aspnet_users_with_roles(pool: &MySqlPool) -> Result<Vec<AspNetUserWithRoles>, sqlx::Error> {
    let query =
        r#"
            SELECT u.Id, u.UserName, u.Email, u.PasswordHash,
            r.Name AS RoleName
            FROM AspNetUsers u
            LEFT JOIN AspNetUserRoles ur ON u.Id = ur.UserId
            LEFT JOIN AspNetRoles r ON ur.RoleId = r.Id
        "#;
    let mut aspnetusers_with_roles: std::collections::HashMap<String, AspNetUserWithRoles> = std::collections::HashMap::new();

    let mut result = sqlx::query(query)
        .fetch(pool);

        while let Some(row_result) = result.next().await {  // use futures::stream::StreamExt;  for .next()
            match row_result {
                Ok(row) => {
                    // Your existing code for processing a successful row
                    let user_id: String = row.get("Id");
                    let username: String = row.get("UserName");
                    let email: String = row.get("Email");
                    let passwordhash: String = row.get("PasswordHash");
                    let role_name: Option<String> = row.get("RoleName");
        
                    let entry: &mut AspNetUserWithRoles = aspnetusers_with_roles
                        .entry(user_id.clone())
                        .or_insert(AspNetUserWithRoles {
                            Id: user_id.clone(),
                            UserName: username,
                            Email: email,
                            PasswordHash: passwordhash,
                            Roles: vec![],
                        });
        
                    if let Some(role) = role_name {
                        entry.Roles.push(Some(role));
                    } else {
                        entry.Roles.push(None);
                    }
                }
                Err(err) => {
                    // Handle the error (e.g., print it)
                    eprintln!("Error fetching row: {:?}", err);
                }
            }
        }
        

    let aspnetusers_with_roles_list: Vec<AspNetUserWithRoles> = aspnetusers_with_roles
        .into_values()
        .collect();

    // for user in &aspnetusers_with_roles_list {
    //     println!("{:?}", user);
    // }

    Ok(aspnetusers_with_roles_list)
}

// HTTP handler for getting al AspNetUser (controller)
async fn get_aspnet_users_with_roles(
    // app_state: web::Data<AppState>       // actix_web
    app_state: web::types::State<AppState>  // ntex
) -> HttpResponse {

    // timer
    let time: std::time::Instant = std::time::Instant::now();

    // Fetch ASP.NET users using the private function
    let users_result: Result<Vec<AspNetUserWithRoles>, sqlx::Error> = fetch_aspnet_users_with_roles(&app_state.pool).await;

    // Handle the result or return an error response
    match users_result {
        Ok(users) => {  // : Vec<AspNetUser>
            // stop timer & print to terminal
            let duration: std::time::Duration = time.elapsed();
            let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
            let elapsed_seconds: f64 = elapsed_ms / 1000.0;
            println!(
                "query time: {:?} ({:?} ms) ({:.8} s)",
                duration, elapsed_ms, elapsed_seconds
            );

            // Response
            HttpResponse::Ok().json(&AspNetUsersWithRolesResponse {
                users,
                message: "Got all ASP.NET Users with joined Roles.".to_string(),
            })
        }
        Err(err) => {
            // Wrap the error and return an error response
            eprintln!("Error fetching ASP.NET Users with joined Roles: {:?}", err);

            let custom_err: CustomError = err.into();
            custom_err.to_http_response()
        }
    }
}





// Define a function to fetch one user by username or email.
async fn fetch_one_aspnet_user(
    // app_state: web::Data<AppState>           // actix_web
    app_state: web::types::State<AppState>,     // ntex
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
    // app_state: web::Data<AppState>           // actix_web
    app_state: web::types::State<AppState>,     // ntex
    // auth_request: web::Json<AuthRequest>,    // actix_web
    auth_request: ntex::web::types::Json<AuthRequest>   // ntex
) -> HttpResponse {
    
    // timer
    let time: std::time::Instant = std::time::Instant::now();

    match fetch_one_aspnet_user(app_state.clone(), &auth_request.username_or_email).await {
        Ok(Some(user)) => {
            // Verify the password
            if verify_password_with_sha256_with_salt(
                &auth_request.password,
                &user.PasswordHash,
            ) {
                // Stop timer & print to terminal
                let duration = time.elapsed();
                let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
                let elapsed_seconds: f64 = elapsed_ms / 1000.0;
                println!(
                    "query time: {:?} ({:?} ms) ({:.8} s)",
                    duration, elapsed_ms, elapsed_seconds
                );

                // Authentication succeeded
                let access_token: String = "your_generated_access_token".to_string(); // Replace with your logic
                let refresh_token: String = "your_generated_refresh_token".to_string(); // Replace with your logic

                let response: AuthResponse = AuthResponse::success(user, access_token, refresh_token);

                // Return the response
                HttpResponse::Ok().json(&response)
            } else {
                // Password is incorrect
                let response: AuthResponse = AuthResponse::invalid_credentials();

                // Return the response
                HttpResponse::Ok().json(&response)
            }
        }
        Ok(None) => {
            // stop timer & print to terminal
            let duration: std::time::Duration = time.elapsed();
            let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
            let elapsed_seconds: f64 = elapsed_ms / 1000.0;
            println!(
                "query time: {:?} ({:?} ms) ({:.8} s)",
                duration, elapsed_ms, elapsed_seconds
            );

            // User not found, return an empty list and appropriate message.
            HttpResponse::Ok().json(&AspNetUsersResponse {
                users: vec![],
                message: "No account exists with the given credentials.".to_string(),
            })
        }

        Err(err) => {
            // Wrap the error and return an error response
            eprintln!("[Task: authenticate_user()]: Error fetching ASP.NET user: {:?}", err);

            let custom_err: CustomError = err.into();
            custom_err.to_http_response()
        }
    }
}
/*  request input:
{
    "username_or_email": "xxxx@example.com",
    "password": "Abc@123"
}
response output:
Wrong credentials, returned {
    "success": false,
    "user": null,
    "access_token": null,
    "refresh_token": null
}
Valid credentials, it responded: {
    "success": true,
    "user": {
        "Id": "99c0f76b-87de-4819-91ea-631f65741ab2",
        "UserName": "xxxx@example.com",
        "Email": "xxxx@example.com",
        "PasswordHash": "f3egqjaYLSt0t1bKWzu1e7GpaYtHEZGGHLD+6PLJ7xkXKrK4qRWi/aVgPhj4Qibc"
    },
    "access_token": "your_generated_access_token",
    "refresh_token": "your_generated_refresh_token"
} */







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

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
use uuid::Uuid;

// define modules & import
mod models; // Create a new module named "models" by convention
use models::models::{AspNetUser, AspNetUsersResponse, AuthRequest}; // Specify the correct module path

#[derive(Clone)]
struct AppState {
    pool: MySqlPool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Thing {
    id: u64,
    i_8: i8,
    i_16: i16,
    i_32: i32,
    i_64: i64,
    f: f32,
    f_double: f64,
    string: String,
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
    const DATABASE_URL: &str = "mysql://user:password@127.0.0.1:3306/BlazorServerCrud"; // "mysql://user:password@127.0.0.1:3306/actix_sqlx"

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
            //.route("/get/{user_id}", web::get().to(get_user))
            // .route("/get-all", web::get().to(get_all_users))
            // .route("/create", web::post().to(create_user))
            // .route("/patch", web::patch().to(patch_user))
            // .route("/delete", web::delete().to(delete_user))
            // .route("/demo", web::get().to(demo))

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


// async fn demo(app_state: web::Data<AppState>) -> HttpResponse {
//     let things: Vec<Thing> = sqlx::query_as!(
//         Thing,
//         "SELECT * FROM things",
//     ).fetch_all(&app_state.pool).await.unwrap();

//     HttpResponse::Ok().json(things)
// }



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

/* test run: (windows 10, Intel i5 gen 10)
query time: 1.6042ms (1.6042 ms) (0.00160420 s)
query time: 1.2359ms (1.2359 ms) (0.00123590 s)
query time: 995.5µs (0.9955 ms) (0.00099550 s)
query time: 1.0672ms (1.0672 ms) (0.00106720 s)
query time: 977.1µs (0.9771000000000001 ms) (0.00097710 s)
query time: 949.9µs (0.9499000000000001 ms) (0.00094990 s)
query time: 1.0364ms (1.0364 ms) (0.00103640 s)
query time: 1.1089ms (1.1089 ms) (0.00110890 s)
query time: 1.2713ms (1.2712999999999999 ms) (0.00127130 s)
query time: 1.0145ms (1.0145 ms) (0.00101450 s)
query time: 897.7µs (0.8976999999999999 ms) (0.00089770 s)
query time: 1.377ms (1.377 ms) (0.00137700 s)
query time: 1.3461ms (1.3461 ms) (0.00134610 s)
query time: 1.0744ms (1.0744 ms) (0.00107440 s)
query time: 1.4353ms (1.4353 ms) (0.00143530 s)
query time: 1.053ms (1.053 ms) (0.00105300 s)
query time: 986.3µs (0.9863000000000001 ms) (0.00098630 s)
query time: 924.8µs (0.9248000000000001 ms) (0.00092480 s)
query time: 1.2397ms (1.2397 ms) (0.00123970 s)
query time: 1.0165ms (1.0165 ms) (0.00101650 s)
query time: 885.2µs (0.8852000000000001 ms) (0.00088520 s)
query time: 1.1683ms (1.1683 ms) (0.00116830 s)
query time: 962.8µs (0.9628 ms) (0.00096280 s)
query time: 991.4µs (0.9914 ms) (0.00099140 s)
query time: 991.9µs (0.9919 ms) (0.00099190 s)
query time: 1.2514ms (1.2513999999999998 ms) (0.00125140 s)
*/

/* test run : (MacOS Monterey, Intel i5 gen 7)

(Chrome)
query time: 13.354221ms (13.354220999999999 ms) (0.01335422 s)
query time: 1.134404ms (1.1344040000000002 ms) (0.00113440 s)
query time: 1.913823ms (1.913823 ms) (0.00191382 s)
query time: 1.098089ms (1.098089 ms) (0.00109809 s)
query time: 1.141108ms (1.141108 ms) (0.00114111 s)
query time: 1.235515ms (1.235515 ms) (0.00123551 s)
query time: 917.211µs (0.917211 ms) (0.00091721 s)
query time: 1.224723ms (1.224723 ms) (0.00122472 s)
query time: 1.023464ms (1.023464 ms) (0.00102346 s)
query time: 995.447µs (0.995447 ms) (0.00099545 s)

(Postman)
query time: 6.629898ms (6.629898000000001 ms) (0.00662990 s)
query time: 3.20108ms (3.2010799999999997 ms) (0.00320108 s)
query time: 1.436492ms (1.4364919999999999 ms) (0.00143649 s)
query time: 1.543356ms (1.5433560000000002 ms) (0.00154336 s)
query time: 918.798µs (0.918798 ms) (0.00091880 s)
query time: 16.498128ms (16.498128 ms) (0.01649813 s)
query time: 1.844631ms (1.844631 ms) (0.00184463 s)
query time: 915.22µs (0.91522 ms) (0.00091522 s)
query time: 989.985µs (0.9899850000000001 ms) (0.00098999 s)
query time: 876.679µs (0.876679 ms) (0.00087668 s)
query time: 1.162814ms (1.162814 ms) (0.00116281 s)
*/





// Define the private function to fetch ASP.NET users (Repository)
async fn fetch_aspnet_users(pool: &MySqlPool) -> Result<Vec<AspNetUser>, sqlx::Error> {
    let users: Vec<AspNetUser> =
        sqlx::query("SELECT u.Id, u.UserName, u.Email, u.PasswordHash FROM Users u")
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
        Err(err) => {
            // Handle the error and return an error response
            eprintln!("Error fetching ASP.NET users: {:?}", err);
            HttpResponse::InternalServerError().json(AspNetUsersResponse {
                users: Vec::new(),
                message: "Failed to fetch ASP.NET users.".to_string(),
            })
        }
    }
}

/* test run: (MacOS Monterey, Intel i5 gen 7)

(browser)
query time: 4.565839ms (4.565839 ms) (0.00456584 s)
query time: 937.131µs (0.937131 ms) (0.00093713 s)
query time: 756.638µs (0.756638 ms) (0.00075664 s)
query time: 1.009978ms (1.0099779999999998 ms) (0.00100998 s)
query time: 670.841µs (0.670841 ms) (0.00067084 s)
query time: 653.499µs (0.6534989999999999 ms) (0.00065350 s)
query time: 1.612772ms (1.6127719999999999 ms) (0.00161277 s)
query time: 793.523µs (0.793523 ms) (0.00079352 s)
query time: 1.056399ms (1.0563989999999999 ms) (0.00105640 s)
query time: 1.012197ms (1.012197 ms) (0.00101220 s)
query time: 640.147µs (0.640147 ms) (0.00064015 s)
query time: 675.241µs (0.675241 ms) (0.00067524 s)
query time: 1.62289ms (1.6228900000000002 ms) (0.00162289 s)
query time: 1.563034ms (1.563034 ms) (0.00156303 s)
query time: 1.082387ms (1.082387 ms) (0.00108239 s)
query time: 1.423153ms (1.423153 ms) (0.00142315 s)
query time: 768.672µs (0.768672 ms) (0.00076867 s)
query time: 1.565801ms (1.565801 ms) (0.00156580 s)

// result:
{"users":[{"Id":"d60449d4-f1c2-43e9-a62f-ae087357fa05","UserName":"nguyentuan8a10ntk@gmail.com","Email":"nguyentuan8a10ntk@gmail.com","PasswordHash":"AQAAAAIAAYagAAAAEKxpBdIrGR6M67pLiiKJA1Jr9LRGHQ8/fln+oHWBvk96wsC4gatTOqyU6zyr76naZw=="}],"message":"Got all ASP.NET users."}

*/




// Define a function to fetch one user by username or email.
async fn fetch_one_aspnet_user(
    app_state: web::Data<AppState>,
    username_or_email: &str,
) -> Result<Option<AspNetUser>, sqlx::Error> {
    let user: Result<Option<AspNetUser>, sqlx::Error> = match sqlx::query(
        "SELECT u.Id, u.UserName, u.Email, u.PasswordHash FROM Users u WHERE u.UserName = ? OR u.Email = ?",
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
        Ok(Some(user)) => {
            // stop timer & print to terminal
            let duration = time.elapsed();
            let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
            let elapsed_seconds = elapsed_ms / 1000.0;
            println!(
                "query time: {:?} ({:?} ms) ({:.8} s)",
                duration, elapsed_ms, elapsed_seconds
            );


            // User found, return the user.
            HttpResponse::Ok().json(AspNetUsersResponse {
                users: vec![user],
                message: "Found user.".to_string(),
            })
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
        Err(err) => {
            // Handle the error, you can return an internal server error or customize it as needed.
            println!("Hey, Caught An Error: {:?}", err);
            HttpResponse::InternalServerError().finish()
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

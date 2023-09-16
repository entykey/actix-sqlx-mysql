// total rust project size: 1.37GB on Windows 10;  1.36GB MacOS Monterey    |
// Intel(R) Core(TM) i5-4460  CPU @ 3.20GHz   3.20 GHz |
// Mem: 4.00 GB (3.88 GB usable)    |
// C: 99,6GB free of 165 GB    |   D: 299 GB (empty)

// Note: to fix the ERR of mysql: "Column count of mysql.proc is wrong. Expected 20, found 16. The table is probably corrupted"
// Run: $ sudo /Applications/XAMPP/xamppfiles/bin/mysql_upgrade

use actix_web::{ HttpServer,
                 App,
                 HttpResponse,
                 web };
use serde::{ Serialize, Deserialize };
use sqlx::mysql::{ MySqlConnection, MySqlPool, MySqlPoolOptions, MySqlQueryResult, MySqlRow };
use sqlx::{FromRow, Connection};
use uuid::Uuid;


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

#[derive(Debug, Serialize, Deserialize)]
struct User {
    //id: i32,
    id: String,
    username: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct Response {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct UserResponse {
    user: User,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct UsersResponse {
    users: Vec<User>,
    message: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // let _database_url: String = env::var("DATABASE_URL").unwrap();
    const DATABASE_URL: &str = "mysql://user:password@127.0.0.1:3306/actix_sqlx";

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
            .route("/get/{user_id}", web::get().to(get_user))
            .route("/get-all", web::get().to(get_all_users))
            .route("/create", web::post().to(create_user))
            .route("/patch", web::patch().to(patch_user))
            .route("/delete", web::delete().to(delete_user))
            .route("/demo", web::get().to(demo))
    }).bind(("127.0.0.1", 4000))?
        .run()
        .await
}

async fn root() -> HttpResponse {
    HttpResponse::Ok().json(Response {
        message: "Server is up and running.".to_string(),
    })
}

async fn demo(app_state: web::Data<AppState>) -> HttpResponse {
    let things: Vec<Thing> = sqlx::query_as!(
        Thing,
        "SELECT * FROM things",
    ).fetch_all(&app_state.pool).await.unwrap();

    HttpResponse::Ok().json(things)
}

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
    let updated: sqlx::Result<MySqlQueryResult> = sqlx::query!(
        "DROP TABLE users",
    ).execute(&app_state.pool).await;

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


// 4ms (Postman)
async fn get_all_users(app_state: web::Data<AppState>) -> HttpResponse {
    // timer
    let time = std::time::Instant::now();

    let users: Vec<User> = sqlx::query_as!(
        User,
        "SELECT * FROM users",
    ).fetch_all(&app_state.pool).await.unwrap();

    // stop timer & print to terminal
    let duration = time.elapsed();
    let elapsed_ms: f64 = duration.as_secs_f64() * 1000.0;
    let elaped_seconds = elapsed_ms / 1000.0;
    println!("query time: {:?} ({:?} ms) ({:.8} s)", duration, elapsed_ms, elaped_seconds);

    // response
    HttpResponse::Ok().json(UsersResponse {
        users,
        message: "Got all users.".to_string(),
    })
}
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

/* test run :(MacOS Monterey, Intel i5 gen 7)

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

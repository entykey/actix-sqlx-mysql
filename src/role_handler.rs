use ntex::web::{self, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::{models::models::{AspNetRole, NewAspNetRole}, AppState};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/roles")
            .route("", web::get().to(load_roles))
            .route("", web::post().to(add_role))
            .route("/{id}", web::put().to(update_role))
            .route("/{id}", web::delete().to(delete_role)),
    );
}


// async fn load_roles(pool: web::Data<MySqlPool>) -> Result<HttpResponse> {
//     let roles_result = sqlx::query("SELECT * FROM AspNetRoles")
//         .fetch_all(pool.get_ref())
//         .await;

//     match roles_result {
//         Ok(roles_list) => {
//             Ok(HttpResponse::Ok().json(roles_list))
//         }

//         // Handle RowNotFound error
//         Err(sqlx::Error::RowNotFound) => {
//             Ok(HttpResponse::NotFound().body("Roles not found"))
//         }

//         // Catching other errors
//         Err(e) => {
//             eprintln!("Error fetching roles: {:?}", e);
//             Ok(HttpResponse::InternalServerError().body("Internal Server Error"))
//         }
//     }
// }

async fn load_roles(
    app_state: web::types::State<AppState>, // ntex
) -> HttpResponse {
    let roles_result: Result<Vec<AspNetRole>, sqlx::Error> = sqlx::query_as::<_, AspNetRole>("SELECT * FROM AspNetRoles")
        .fetch_all(&app_state.pool)
        .await;

    match roles_result {
        Ok(roles_list) => {
            HttpResponse::Ok().json(&roles_list)
        }
        // Handle RowNotFound error
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().body("Roles not found")
        }
        // Catching other errors
        Err(e) => {
            eprintln!("Error fetching roles: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}


async fn add_role(
    app_state: web::types::State<AppState>,
    new_role: web::types::Json<NewAspNetRole>,
) -> HttpResponse {
    let name = &new_role.name;
    if name.is_empty() {
        return HttpResponse::BadRequest().body("Role name cannot be empty");
    }

    let insert_result = sqlx::query!(
        "INSERT INTO AspNetRoles (Id, Name, NormalizedName) VALUES (?, ?, ?)",
        uuid::Uuid::new_v4().to_string(),
        name.clone(),
        name.to_uppercase()
    )
    .execute(&app_state.pool)
    .await;

    match insert_result {
        Ok(_) => {
            HttpResponse::Created().body("Role added successfully")
        }
        Err(e) => {
            eprintln!("Error adding role: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

async fn update_role(
    app_state: web::types::State<AppState>,
    path: web::types::Path<String>,
    new_role: web::types::Json<NewAspNetRole>,
) -> HttpResponse {
    let role_id = &*path;
    let name = &new_role.name;

    let update_result = sqlx::query!(
        "UPDATE AspNetRoles SET Name = ?, NormalizedName = ? WHERE Id = ?",
        name,
        name.to_uppercase(),
        role_id
    )
    .execute(&app_state.pool)
    .await;

    match update_result {
        Ok(_) => {
            HttpResponse::Ok().body("Role updated successfully")
        }
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().body("Role not found")
        }
        Err(e) => {
            eprintln!("Error updating role: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

async fn delete_role(
    app_state: web::types::State<AppState>,
    path: web::types::Path<String>,
) -> HttpResponse {
    let role_id = &*path;

    let delete_result = sqlx::query!("DELETE FROM AspNetRoles WHERE Id = ?", role_id)
        .execute(&app_state.pool)
        .await;

    match delete_result {
        Ok(_) => {
            HttpResponse::Ok().body("Role deleted successfully")
        }
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().body("Role not found")
        }
        Err(e) => {
            eprintln!("Error deleting role: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}
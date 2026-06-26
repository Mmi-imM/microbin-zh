use crate::args::{Args, ARGS};
use crate::endpoints::errors::ErrorTemplate;
use crate::util::misc::remove_expired;
use crate::util::share_code::find_pasta_index_by_code;
use crate::AppState;
use actix_web::{get, web, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "auth_upload.html")]
struct AuthPasta<'a> {
    args: &'a Args,
    id: String,
    status: String,
    encrypted_key: String,
    encrypt_client: bool,
    path: String,
}

fn auth_response(
    data: web::Data<AppState>,
    id: String,
    status: String,
    path: &'static str,
) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    if let Some(index) = find_pasta_index_by_code(&pastas, &id) {
        return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
            AuthPasta {
                args: &ARGS,
                id: pastas[index].id_as_animals(),
                status,
                encrypted_key: pastas[index].encrypted_key.to_owned().unwrap_or_default(),
                encrypt_client: pastas[index].encrypt_client,
                path: String::from(path),
            }
            .render()
            .unwrap(),
        );
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth/{id}")]
pub async fn auth_upload(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    auth_response(data, id.into_inner(), String::from(""), "upload")
}

#[get("/auth/{id}/{status}")]
pub async fn auth_upload_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    let (id, status) = param.into_inner();
    auth_response(data, id, status, "upload")
}

#[get("/auth_raw/{id}")]
pub async fn auth_raw_pasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    auth_response(data, id.into_inner(), String::from(""), "raw")
}

#[get("/auth_raw/{id}/{status}")]
pub async fn auth_raw_pasta_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    let (id, status) = param.into_inner();
    auth_response(data, id, status, "raw")
}

#[get("/auth_edit_private/{id}")]
pub async fn auth_edit_private(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    auth_response(data, id.into_inner(), String::from(""), "edit_private")
}

#[get("/auth_edit_private/{id}/{status}")]
pub async fn auth_edit_private_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    let (id, status) = param.into_inner();
    auth_response(data, id, status, "edit_private")
}

#[get("/auth_file/{id}")]
pub async fn auth_file(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    auth_response(data, id.into_inner(), String::from(""), "secure_file")
}

#[get("/auth_file/{id}/{status}")]
pub async fn auth_file_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    let (id, status) = param.into_inner();
    auth_response(data, id, status, "secure_file")
}

#[get("/auth_remove_private/{id}")]
pub async fn auth_remove_private(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    auth_response(data, id.into_inner(), String::from(""), "remove")
}

#[get("/auth_remove_private/{id}/{status}")]
pub async fn auth_remove_private_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    let (id, status) = param.into_inner();
    auth_response(data, id, status, "remove")
}

#[get("/auth_url/{id}")]
pub async fn auth_url(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    auth_response(data, id.into_inner(), String::from(""), "url")
}

#[get("/auth_url/{id}/{status}")]
pub async fn auth_url_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    let (id, status) = param.into_inner();
    auth_response(data, id, status, "url")
}

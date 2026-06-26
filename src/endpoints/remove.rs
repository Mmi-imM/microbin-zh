use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpResponse};

use crate::args::ARGS;
use crate::endpoints::errors::ErrorTemplate;

use crate::util::auth;
use crate::util::db::delete;
use crate::util::misc::{decrypt, remove_expired};
use crate::util::share_code::find_pasta_index_by_code;
use crate::AppState;
use askama::Template;
use std::fs;

#[get("/remove/{id}")]
pub async fn remove(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();

    let id = id.into_inner();
    remove_expired(&mut pastas);

    if let Some(i) = find_pasta_index_by_code(&pastas, &id) {
        let pasta_id = pastas[i].id;
        let display_id = pastas[i].id_as_animals();
        let storage_id = pastas[i].storage_id_as_animals();

        // if it's encrypted or read-only, it needs password to be deleted
        // OR if it is not editable (public immutable), it needs admin password to be deleted
        if pastas[i].encrypt_server || pastas[i].readonly || !pastas[i].editable {
            return HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("{}/auth_remove_private/{}", ARGS.public_path_as_str(), display_id),
                ))
                .finish();
        }

        // remove the directory and all its contents
        if fs::remove_dir_all(format!("{}/attachments/{}/", ARGS.data_dir, storage_id)).is_err() {
            log::error!("Failed to delete directory for {}!", display_id)
        }

        // remove it from in-memory pasta list
        pastas.remove(i);

        delete(Some(&pastas), Some(pasta_id));

        return HttpResponse::Found()
            .append_header(("Location", format!("{}/list", ARGS.public_path_as_str())))
            .finish();
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[post("/remove/{id}")]
pub async fn post_remove(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let id = id.into_inner();

    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let password = auth::password_from_multipart(payload).await?;

    if let Some(i) = find_pasta_index_by_code(&pastas, &id) {
        let pasta_id = pastas[i].id;
        let display_id = pastas[i].id_as_animals();
        let storage_id = pastas[i].storage_id_as_animals();

        if pastas[i].readonly || pastas[i].encrypt_server || !pastas[i].editable {
            if password != *"" {
                let mut is_password_correct = false;

                if password == *ARGS.auth_admin_password {
                    is_password_correct = true;
                }

                // if it is read-only, the content is not encrypted, but the key is
                if !is_password_correct && pastas[i].readonly {
                    if let Some(ref encrypted_key) = pastas[i].encrypted_key {
                        let res = decrypt(encrypted_key, &password);
                        if let Ok(decrypted_key) = res {
                            if decrypted_key == pasta_id.to_string() {
                                is_password_correct = true;
                            }
                        }
                    }
                } else if !is_password_correct && pastas[i].encrypt_server {
                    // if it is not read-only, the content is encrypted
                    let res = decrypt(pastas[i].content.to_owned().as_str(), &password);
                    if res.is_ok() {
                        is_password_correct = true;
                    }
                }

                if is_password_correct {
                // remove the directory and all its contents
                    if fs::remove_dir_all(format!("{}/attachments/{}/", ARGS.data_dir, storage_id))
                        .is_err()
                    {
                        log::error!("Failed to delete directory for {}!", display_id)
                    }

                    // remove it from in-memory pasta list
                    pastas.remove(i);

                    delete(Some(&pastas), Some(pasta_id));

                    return Ok(HttpResponse::Found()
                        .append_header(("Location", format!("{}/list", ARGS.public_path_as_str())))
                        .finish());
                } else {
                    return Ok(HttpResponse::Found()
                        .append_header((
                            "Location",
                            format!(
                                "{}/auth_remove_private/{}/incorrect",
                                ARGS.public_path_as_str(),
                                display_id
                            ),
                        ))
                        .finish());
                }
            } else {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!(
                            "{}/auth_remove_private/{}/incorrect",
                            ARGS.public_path_as_str(),
                            display_id
                        ),
                    ))
                    .finish());
            }
        }

        return Ok(HttpResponse::Found()
            .append_header((
                "Location",
                format!("{}/upload/{}", ARGS.public_path_as_str(), display_id),
            ))
            .finish());
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap()))
}

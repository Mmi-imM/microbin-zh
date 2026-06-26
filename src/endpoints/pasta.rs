use crate::args::{Args, ARGS};
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::Pasta;
use crate::util::auth;
use crate::util::db::update;
use crate::util::misc::remove_expired;
use crate::util::share_code::find_pasta_index_by_code;
use crate::AppState;
use actix_multipart::Multipart;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse};
use askama::Template;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Template)]
#[template(path = "upload.html", escape = "none")]
struct PastaTemplate<'a> {
    pasta: &'a Pasta,
    args: &'a Args,
}

fn pastaresponse(
    data: web::Data<AppState>,
    id: String,
    password: String,
    skip_increment: bool,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    if let Some(index) = find_pasta_index_by_code(&pastas, &id) {
        if pastas[index].encrypt_server && password == *"" {
            return HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("{}/auth/{}", ARGS.public_path_as_str(), pastas[index].id_as_animals()),
                ))
                .finish();
        }

        if !skip_increment {
            // increment read count
            pastas[index].read_count += 1;

            // save the updated read count
            update(Some(&pastas), Some(&pastas[index]));
        }

        let original_content = pastas[index].content.to_owned();

        // decrypt content temporarily
        if password != *"" && !original_content.is_empty() {
            let res = decrypt(&original_content, &password);
            if let Ok(..) = res {
                pastas[index]
                    .content
                    .replace_range(.., res.unwrap().as_str());
            } else {
                return HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("{}/auth/{}/incorrect", ARGS.public_path_as_str(), pastas[index].id_as_animals()),
                    ))
                    .finish();
            }
        }

        // serve pasta in template
        let response = HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
            PastaTemplate {
                pasta: &pastas[index],
                args: &ARGS,
            }
            .render()
            .unwrap(),
        );

        if pastas[index].content != original_content {
            pastas[index].content = original_content;
        }

        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // update last read time
        pastas[index].last_read = timenow;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        return response;
    }

    // otherwise send pasta not found error
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[post("/upload/{id}")]
pub async fn postpasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let password = auth::password_from_multipart(payload).await?;
    Ok(pastaresponse(data, id.into_inner(), password, false))
}

#[post("/p/{id}")]
pub async fn postshortpasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let password = auth::password_from_multipart(payload).await?;
    Ok(pastaresponse(data, id.into_inner(), password, false))
}

#[get("/upload/{id}")]
pub async fn getpasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let id = id.into_inner();
    let mut skip_increment = false;

    // the user attached an owner_token. likely they're the same user that created the pasta
    // but let's verify it just in case
    if let Some(cookie) = req.cookie("owner_token") {
        let mut pastas = data.pastas.lock().unwrap();
        remove_expired(&mut pastas);
        if let Some(index) = find_pasta_index_by_code(&pastas, &id) {
            if verify_owner_token(cookie.value(), pastas[index].id) {
                // yay, it really is the same user and their cookie isn't expired
                // so let's skip incrementing the read count
                skip_increment = true;
            }
        }
    }

    pastaresponse(data, id, String::from(""), skip_increment)
}

// when creating a pasta, the owner is issued a token with a 15-second expiration
// this token is used to avoid incrementing the read count of the pasta when the owner views it
fn verify_owner_token(token: &str, expected_id: u64) -> bool {
    // decode the token
    if let Ok(numbers) = crate::util::hashids::HARSH.decode(token) {
        if numbers.len() == 2 {
            let expiry = numbers[0];
            let token_id = numbers[1];
            let timenow = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

            // verify the token is valid
            if token_id == expected_id && expiry > timenow {
                // yay, it's valid
                return true;
            }
        }
    }
    false
}

#[get("/p/{id}")]
pub async fn getshortpasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    pastaresponse(data, id.into_inner(), String::from(""), false)
}

fn urlresponse(data: web::Data<AppState>, id: String, password: String) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    if let Some(index) = find_pasta_index_by_code(&pastas, &id) {
        if pastas[index].encrypt_server && password == *"" {
            return HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("{}/auth_url/{}", ARGS.public_path_as_str(), pastas[index].id_as_animals()),
                ))
                .finish();
        }

        // send redirect if it's a url pasta
        if pastas[index].pasta_type == "url" {
            let target_url = if password != *"" {
                let res = decrypt(&pastas[index].content, &password);
                if let Ok(url) = res {
                    url
                } else {
                    return HttpResponse::Found()
                        .append_header((
                            "Location",
                            format!(
                                "{}/auth_url/{}/incorrect",
                                ARGS.public_path_as_str(),
                                pastas[index].id_as_animals()
                            ),
                        ))
                        .finish();
                }
            } else {
                pastas[index].content.to_owned()
            };

            // increment read count
            pastas[index].read_count += 1;

            let response = HttpResponse::Found()
                .append_header(("Location", target_url))
                .finish();

            // get current unix time in seconds
            let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => n.as_secs(),
                Err(_) => {
                    log::error!("SystemTime before UNIX EPOCH!");
                    0
                }
            } as i64;

            // update last read time
            pastas[index].last_read = timenow;

            // save the updated read count
            update(Some(&pastas), Some(&pastas[index]));

            return response;
        // send error if we're trying to open a non-url pasta as a redirect
        } else {
            return HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(ErrorTemplate { args: &ARGS }.render().unwrap());
        }
    }

    // otherwise send pasta not found error
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/url/{id}")]
pub async fn redirecturl(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    urlresponse(data, id.into_inner(), String::from(""))
}

#[post("/url/{id}")]
pub async fn postredirecturl(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let password = auth::password_from_multipart(payload).await?;
    Ok(urlresponse(data, id.into_inner(), password))
}

#[get("/u/{id}")]
pub async fn shortredirecturl(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    urlresponse(data, id.into_inner(), String::from(""))
}

#[get("/raw/{id}")]
pub async fn getrawpasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = id.into_inner();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    if let Some(index) = find_pasta_index_by_code(&pastas, &id) {
        if pastas[index].encrypt_server {
            return Ok(HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("{}/auth_raw/{}", ARGS.public_path_as_str(), pastas[index].id_as_animals()),
                ))
                .finish());
        }

        // increment read count
        pastas[index].read_count += 1;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // update last read time
        pastas[index].last_read = timenow;

        // send raw content of pasta
        let response = Ok(HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .body(pastas[index].content.to_owned()));

        return response;
    }

    // otherwise send pasta not found error as raw text
    Ok(HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(String::from("Upload not found! :-(")))
}

#[post("/raw/{id}")]
pub async fn postrawpasta(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let password = auth::password_from_multipart(payload).await?;

    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = id.into_inner();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    if let Some(index) = find_pasta_index_by_code(&pastas, &id) {
        if pastas[index].encrypt_server && password == *"" {
            return Ok(HttpResponse::Found()
                .append_header((
                    "Location",
                    format!("{}/auth/{}", ARGS.public_path_as_str(), pastas[index].id_as_animals()),
                ))
                .finish());
        }

        // increment read count
        pastas[index].read_count += 1;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        let original_content = pastas[index].content.to_owned();

        // decrypt content temporarily
        if password != *"" {
            let res = decrypt(&original_content, &password);
            if res.is_ok() {
                pastas[index]
                    .content
                    .replace_range(.., res.unwrap().as_str());
            } else {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("{}/auth/{}/incorrect", ARGS.public_path_as_str(), pastas[index].id_as_animals()),
                    ))
                    .finish());
            }
        }

        // get current unix time in seconds
        let timenow: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => {
                log::error!("SystemTime before UNIX EPOCH!");
                0
            }
        } as i64;

        // update last read time
        pastas[index].last_read = timenow;

        // save the updated read count
        update(Some(&pastas), Some(&pastas[index]));

        // send raw content of pasta
        let response = Ok(HttpResponse::Ok()
            .content_type("text/plain; charset=utf-8")
            .body(pastas[index].content.to_owned()));

        if pastas[index].content != original_content {
            pastas[index].content = original_content;
        }

        return response;
    }

    // otherwise send pasta not found error as raw text
    Ok(HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(String::from("Upload not found! :-(")))
}

fn decrypt(text_str: &str, key_str: &str) -> Result<String, magic_crypt::MagicCryptError> {
    let mc = new_magic_crypt!(key_str, 256);

    mc.decrypt_base64_to_string(text_str)
}

#[get("/{code}")]
pub async fn root_lookup(data: web::Data<AppState>, code: web::Path<String>) -> HttpResponse {
    let mut pastas = data.pastas.lock().unwrap();
    let code = code.into_inner();

    remove_expired(&mut pastas);

    if let Some(index) = find_pasta_index_by_code(&pastas, &code) {
        let pasta = &pastas[index];
        let slug = pasta.id_as_animals();
        let path = if pasta.encrypt_server {
            if pasta.pasta_type == "url" {
                "auth_url"
            } else {
                "auth"
            }
        } else if pasta.pasta_type == "url" {
            "url"
        } else {
            "upload"
        };

        return HttpResponse::Found()
            .append_header((
                "Location",
                format!("{}/{}/{}", ARGS.public_path_as_str(), path, slug),
            ))
            .finish();
    }

    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_token_must_match_expected_pasta_id() {
        let expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60;
        let token = crate::util::hashids::HARSH.encode(&[expiry, 42]);

        assert!(verify_owner_token(&token, 42));
        assert!(!verify_owner_token(&token, 7));
    }
}

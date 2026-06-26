use std::fs::File;
use std::path::PathBuf;

use crate::args::ARGS;
use crate::util::auth;
use crate::util::misc::remove_expired;
use crate::util::misc::decrypt_file;
use crate::util::share_code::find_pasta_index_by_code;
use crate::AppState;
use actix_multipart::Multipart;
use actix_web::http::header;
use actix_web::{get, post, web, Error, HttpResponse};

use std::collections::HashMap;

#[post("/secure_file/{id}")]
pub async fn post_secure_file(
    data: web::Data<AppState>,
    id: web::Path<String>,
    payload: Multipart,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = id.into_inner();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    let password = auth::password_from_multipart(payload).await?;

    if let Some(index) = find_pasta_index_by_code(&pastas, &id) {
        let mut target_filename = None;
        if let Some(fname) = query.get("fname") {
             // sanitize fname? It should match one of the attachments or file.
             // Security check: ensure fname is in the list of files for this pasta
             if let Some(file) = &pastas[index].file {
                 if file.name() == *fname {
                     target_filename = Some(file.name());
                 }
             }
             if target_filename.is_none() {
                 if let Some(attachments) = &pastas[index].attachments {
                     for att in attachments {
                         if att.name() == *fname {
                             target_filename = Some(att.name());
                             break;
                         }
                     }
                 }
             }
        }

        // Fallback to primary file if no fname or not found (and fname wasn't provided)
        if target_filename.is_none() && query.get("fname").is_none() {
             if let Some(file) = &pastas[index].file {
                 target_filename = Some(file.name());
             }
        }

        if let Some(filename) = target_filename {
            // Try new naming scheme {filename}.enc first, then fallback to data.enc (legacy/primary)
            let mut enc_path = format!(
                "{}/attachments/{}/{}.enc",
                ARGS.data_dir,
                pastas[index].storage_id_as_animals(),
                filename
            );
            
            if !std::path::Path::new(&enc_path).exists() {
                 // Fallback for legacy primary file
                 enc_path = format!(
                    "{}/attachments/{}/data.enc",
                    ARGS.data_dir,
                    pastas[index].storage_id_as_animals()
                );
            }

            if let Ok(file) = File::open(&enc_path) {
                // Not compatible with NamedFile from actix_files (it needs a File
                // to work therefore secure files do not support streaming
                let decrypted_data: Vec<u8> = decrypt_file(&password, &file)?;

                // Set the content type based on the file extension
                let content_type = mime_guess::from_path(&filename)
                    .first_or_octet_stream()
                    .to_string();

                // Create a response with the decrypted data
                let response = HttpResponse::Ok()
                    .content_type(content_type)
                    .append_header((
                        "Content-Disposition",
                        format!("attachment; filename=\"{}\"", filename),
                    ))
                    // TODO: make streaming <21-10-24, dvdsk>
                    .body(decrypted_data);
                return Ok(response);
            }
        }
    }
    Ok(HttpResponse::NotFound().finish())
}

#[get("/file/{id}")]
pub async fn get_file(
    request: actix_web::HttpRequest,
    id: web::Path<String>,
    data: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = id.into_inner();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    if let Some(index) = find_pasta_index_by_code(&pastas, &id) {
        // Determine which file to serve
        let mut target_file = None;
        if let Some(fname) = query.get("fname") {
            if let Some(file) = &pastas[index].file {
                if file.name() == *fname {
                    target_file = Some(file);
                }
            }
            if target_file.is_none() {
                if let Some(attachments) = &pastas[index].attachments {
                    for att in attachments {
                        if att.name() == *fname {
                            target_file = Some(att);
                            break;
                        }
                    }
                }
            }
        } else {
            // Default to primary file
             if let Some(file) = &pastas[index].file {
                target_file = Some(file);
            }
        }

        if let Some(pasta_file) = target_file {
            if pastas[index].encrypt_server {
                return Ok(HttpResponse::Found()
                    .append_header((
                        "Location",
                        format!("{}/auth_file/{}", ARGS.public_path_as_str(), pastas[index].id_as_animals()),
                    ))
                    .finish());
            }

            // Construct the path to the file
            let file_path = format!(
                "{}/attachments/{}/{}",
                ARGS.data_dir,
                pastas[index].storage_id_as_animals(),
                pasta_file.name()
            );
            let file_path = PathBuf::from(file_path);

            // This will stream the file and set the content type based on the
            // file path
            let file_reponse = actix_files::NamedFile::open(file_path)?;
            
            let disposition = if query.get("preview").map(|s| s == "true").unwrap_or(false) {
                header::DispositionType::Inline
            } else {
                header::DispositionType::Attachment
            };

            let file_reponse = file_reponse.set_content_disposition(header::ContentDisposition {
                disposition,
                parameters: vec![header::DispositionParam::Filename(
                    pasta_file.name().to_string(),
                )],
            });
            // This takes care of streaming/seeking using the Range
            // header in the request.
            return Ok(file_reponse.into_response(&request));
        }
    }

    Ok(HttpResponse::NotFound().finish())
}

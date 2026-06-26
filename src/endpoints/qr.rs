use crate::args::{Args, ARGS};
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::Pasta;
use crate::util::misc::{self, remove_expired};
use crate::util::share_code::find_pasta_index_by_code;
use crate::AppState;
use actix_web::{get, web, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "qr.html", escape = "none")]
struct QRTemplate<'a> {
    qr: &'a String,
    pasta: &'a Pasta,
    args: &'a Args,
}

#[get("/qr/{id}")]
pub async fn getqr(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = id.into_inner();

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    if let Some(index) = find_pasta_index_by_code(&pastas, &id) {
        let slug = pastas[index].id_as_animals();
        // generate the QR code as an SVG - if its a file or text pastas, this will point to the /upload endpoint, otherwise to the /url endpoint, essentially directly taking the user to the url stored in the pasta
        let svg = misc::string_to_qr_svg(&match pastas[index].pasta_type.as_str() {
            "url" => match ARGS.short_path.as_ref() {
                Some(short) => format!("{short}/u/{slug}"),
                _ => format!("{}/url/{}", &ARGS.public_path_as_str(), &slug),
            },
            _ => match ARGS.short_path.as_ref() {
                Some(short) => format!("{short}/p/{slug}"),
                _ => format!("{}/upload/{}", &ARGS.public_path_as_str(), &slug),
            },
        });

        // serve qr code in template
        return HttpResponse::Ok().content_type("text/html; charset=utf-8").body(
            QRTemplate {
                qr: &svg,
                pasta: &pastas[index],
                args: &ARGS,
            }
            .render()
            .unwrap(),
        );
    }

    // otherwise
    // send pasta not found error
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

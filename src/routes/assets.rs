use crate::model::state::path;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

#[get("/assets/{file:.*}")]
pub async fn assets(req: HttpRequest, file: web::Path<String>) -> impl Responder {
  let file = file.into_inner();
  if file.contains('/') {
    return HttpResponse::BadRequest().finish();
  }

  let path = format!("{}/assets/{}", path(), file);
  let file = actix_files::NamedFile::open_async(path).await;

  match file {
    Ok(file) => file.into_response(&req),
    Err(_) => HttpResponse::NotFound().finish()
  }
}

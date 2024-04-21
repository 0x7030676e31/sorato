use actix_web::Scope;

mod actor;
mod audio;
mod stream;

pub fn routes() -> Scope {
  Scope::new("/v1actor")
    .service(actor::routes())
    .service(audio::routes())
    .service(stream::stream)
}
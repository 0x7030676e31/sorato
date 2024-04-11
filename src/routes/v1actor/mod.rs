use actix_web::Scope;

mod actor;
mod stream;

pub fn routes() -> Scope {
  Scope::new("/v1actor")
    .service(actor::routes())
    .service(stream::stream)
}
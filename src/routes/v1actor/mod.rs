use actix_web::Scope;

mod actor;

pub fn routes() -> Scope {
  Scope::new("/v1actor")
  .service(actor::routes())
}
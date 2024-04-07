use actix_web::Scope;

pub fn routes() -> Scope {
  Scope::new("/v1client")
}
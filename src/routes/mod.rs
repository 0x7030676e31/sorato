use actix_web::Scope;

mod v1actor;
mod v1client;
mod assets;

pub fn routes() -> Scope {
  Scope::new("/")
    .service(assets::assets)
    .service(api())
}

fn api() -> Scope {
  Scope::new("/api")
    .service(v1actor::routes())
    .service(v1client::routes())
}

use crate::{AppState, Auth, Secure};

use actix_governor::{GovernorConfig, Governor};
use actix_web::{web, HttpResponse, Scope};

// #[post("/authorize")]
async fn authorize(state: web::Data<AppState>, code: web::Json<String>) -> HttpResponse {
  let mut state = state.write().await;
  match state.exchange_code(&code).await {
    Some(name) => HttpResponse::Ok().body(name),
    None => HttpResponse::BadRequest().body("Invalid code"),
  }
}

// #[post("/code")]
async fn gen_code(state: web::Data<AppState>, name: web::Json<String>) -> HttpResponse {
  if name.is_empty() {
    return HttpResponse::BadRequest().body("Name is required");
  }

  if name.len() > 64 {
    return HttpResponse::BadRequest().body("Name is too long");
  }

  let mut state = state.write().await;
  let code = state.create_code(name.into_inner());

  HttpResponse::Ok().body(code)
}

// #[post("/{actor_id}/rename")]
async fn rename(state: web::Data<AppState>, actor_id: web::Path<u32>, name: web::Json<String>) -> HttpResponse {
  if name.is_empty() {
    return HttpResponse::BadRequest().body("Name is required");
  }

  if name.len() > 64 {
    return HttpResponse::BadRequest().body("Name is too long");
  }

  let mut state = state.write().await;
  match state.rename_actor(*actor_id, name.into_inner()).await {
    Some(_) => HttpResponse::Ok().finish(),
    None => HttpResponse::BadRequest().body("Invalid actor ID"),
  }
}

// #[delete("/{actor_id}")]
async fn revoke(state: web::Data<AppState>, actor_id: web::Path<u32>) -> HttpResponse {
  let mut state = state.write().await;
  match state.revoke_actor_access(*actor_id).await {
    Some(_) => HttpResponse::Ok().finish(),
    None => HttpResponse::BadRequest().body("Invalid actor ID"),
  }
}

// #[post("/{actor_id}/access")]
async fn set_access(state: web::Data<AppState>, actor_id: web::Path<u32>, access: web::Json<bool>) -> HttpResponse {
  let mut state = state.write().await;
  match state.set_actor_access(*actor_id, access.into_inner()).await {
    Some(_) => HttpResponse::Ok().finish(),
    None => HttpResponse::BadRequest().body("Invalid actor ID"),
  }
}


pub fn routes() -> Scope {
  let conf: Secure = GovernorConfig::secure();

  Scope::new("/actor")
    .service(
      web::resource("/authorize")
        .wrap(Governor::new(&conf))
        .route(web::post().to(authorize))
    )
    .service(
      web::resource("/code")
        .wrap(Auth::head())
        .route(web::post().to(gen_code))
    )
    .service(
      web::resource("/{actor_id}/rename")
        .wrap(Auth::head())
        .route(web::post().to(rename))
    )
    .service(
      web::resource("/{actor_id}")
        .wrap(Auth::head())
        .route(web::delete().to(revoke))
    )
    .service(
      web::resource("/{actor_id}/access")
        .wrap(Auth::head())
        .route(web::post().to(set_access))
    )
}


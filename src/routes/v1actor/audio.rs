use crate::routes::extractor::Token;
use crate::model::stream::SsePayload;
use crate::{AppState, Auth};

use std::io::Write;

use actix_web::{HttpResponse, Scope, web};
use futures::StreamExt;
use serde::Deserialize;

#[derive(Deserialize)]
struct AudioQuery {
  title: String,
}

async fn upload_audio(state: web::Data<AppState>, query: web::Query<AudioQuery>, mut payload: web::Payload, token: Token) -> HttpResponse {
  let mut state_ = state.write().await;
  let AudioQuery { title } = query.into_inner();
  log::info!("Uploading audio {}...", title);

  let actor_id = state_.token_to_id(&token.0);
  let (id, mut writer) = match state_.get_audio_writer() {
    Ok((id, writer)) => (id, writer),
    Err(e) => {
      log::error!("Failed to get audio writer: {}", e);
      return HttpResponse::InternalServerError().finish();
    }
  };

  drop(state_);
  while let Some(chunk) = payload.next().await {
    let chunk = match chunk {
      Ok(chunk) => chunk,
      Err(e) => {
        log::error!("Failed to get chunk: {}", e);
        return HttpResponse::InternalServerError().finish();
      }
    };

    if let Err(e) = writer.write_all(&chunk) {
      log::error!("Failed to write chunk: {}", e);
      return HttpResponse::InternalServerError().finish();
    }
  }

  let mut state = state.write().await;
  let length = match state.finalize_audio_upload(id, title.clone(), actor_id) {
    Ok(Some(length)) => length,
    Ok(None) => {
      if let Err(err) = state.remove_audio_file(id) {
        log::error!("Failed to remove audio file: {}", err);
      }
      
      log::error!("Failed to finalize audio upload: Invalid file format");
      return HttpResponse::UnprocessableEntity().finish();
    }
    Err(e) => {
      if let Err(err) = state.remove_audio_file(id) {
        log::error!("Failed to remove audio file: {}", err);
      }

      log::error!("Failed to finalize audio upload: {}", e);
      return HttpResponse::InternalServerError().finish();
    }
  };

  let sse_payload = SsePayload::AudioCreated { id, title: &title, length, author: actor_id };
  state.broadcast_to_all(sse_payload, None).await;

  HttpResponse::Created().finish()
}

pub fn routes() -> Scope {
  Scope::new("/audio")
    .service(
      web::resource("/upload")
        .wrap(Auth::both())
        .route(web::post().to(upload_audio)
    )
  )
}
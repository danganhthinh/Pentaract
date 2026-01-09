use std::sync::Arc;

use axum::{
    body::Full,
    extract::{Path as RoutePath, Query, State},
    http::StatusCode,
    response::{AppendHeaders, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use reqwest::header;
use std::path::Path;
use tokio_util::bytes::Bytes;
use uuid::Uuid;

use crate::{
    common::routing::app_state::AppState,
    schemas::files::SearchQuery,
    services::files::FilesService,
};

pub struct PublicFilesRouter;

impl PublicFilesRouter {
    pub fn get_router(state: Arc<AppState>) -> Router {
        Router::new()
            .route("/:storage_id/*path", get(Self::dynamic_get))
            .with_state(state)
    }

    async fn dynamic_get(
        State(state): State<Arc<AppState>>,
        RoutePath((storage_id, path)): RoutePath<(Uuid, String)>,
        query: Query<SearchQuery>,
    ) -> impl IntoResponse {
        let (root_path, path) = path.split_once("/").unwrap_or((&path, ""));
        match root_path {
            "tree" => Self::tree(state, storage_id, path).await,
            "download" => Self::download(state, storage_id, path).await,
            "info" => Self::fetch_info(state, storage_id, path).await,
            "search" => {
                if let Some(search_path) = query.0.search_path {
                    Self::search(state, storage_id, path, &search_path).await
                } else {
                    Err((
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "search_path query parameter is required".to_owned(),
                    ))
                }
            }
            _ => Err((StatusCode::NOT_FOUND, "Not found".to_owned())),
        }
    }

    async fn tree(
        state: Arc<AppState>,
        storage_id: Uuid,
        path: &str,
    ) -> Result<Response, (StatusCode, String)> {
        let fs_layer = FilesService::new(&state.db, state.tx.clone())
            .list_dir_public(storage_id, path)
            .await?;
        Ok(Json(fs_layer).into_response())
    }

    async fn fetch_info(
        state: Arc<AppState>,
        storage_id: Uuid,
        path: &str,
    ) -> Result<Response, (StatusCode, String)> {
        let file = FilesService::new(&state.db, state.tx.clone())
            .get_file_info_public(path, storage_id)
            .await?;
        Ok(Json(file).into_response())
    }

    async fn download(
        state: Arc<AppState>,
        storage_id: Uuid,
        path: &str,
    ) -> Result<Response, (StatusCode, String)> {
        FilesService::new(&state.db, state.tx.clone())
            .download_public(path, storage_id)
            .await
            .map(|data| {
                let filename = Path::new(&path)
                    .file_name()
                    .map(|name| name.to_str().unwrap_or_default())
                    .unwrap_or("unnamed.bin");
                let content_type = mime_guess::from_path(filename)
                    .first_or_octet_stream()
                    .to_string();
                let bytes = Bytes::from(data);
                let body = Full::new(bytes);

                let headers = AppendHeaders([
                    (header::CONTENT_TYPE, content_type),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{filename}\""),
                    ),
                ]);

                (headers, body).into_response()
            })
            .map_err(|e| <(StatusCode, String)>::from(e))
    }

    async fn search(
        state: Arc<AppState>,
        storage_id: Uuid,
        path: &str,
        search_path: &str,
    ) -> Result<Response, (StatusCode, String)> {
        FilesService::new(&state.db, state.tx.clone())
            .search_public(storage_id, path, search_path)
            .await
            .map(|files| Json(files).into_response())
            .map_err(|e| <(StatusCode, String)>::from(e))
    }
}

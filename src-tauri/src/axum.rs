use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use log::info;
use std::{net::SocketAddr, path::PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn run_server() {
    let app = Router::new().route("/:name", get(serve_jpg));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    info!("Axum server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn serve_jpg(Path(name): Path<String>) -> Response {
    let mut path = PathBuf::from("../static/images");
    let filename = format!("{}.png", name);
    path.push(&filename);

    if !path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    match File::open(&path).await {
        Ok(mut file) => {
            let mut buf = Vec::new();
            if file.read_to_end(&mut buf).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            (
                [
                    ("Content-Type", "image/png"),
                    (
                        "Content-Disposition",
                        &format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                buf,
            )
                .into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

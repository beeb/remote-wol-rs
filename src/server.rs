use std::{env, path::Path, sync::Arc};

use anyhow::Result;
use axum::{
    body::{boxed, Full},
    extract::Extension,
    handler::HandlerWithoutStateExt,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, handle_server_fns, LeptosRoutes};
use rust_embed::RustEmbed;

use crate::app::*;

pub async fn server_start() -> Result<()> {
    let config_file = Path::new("Cargo.toml").exists().then_some("Cargo.toml");
    if config_file.is_none() {
        env::set_var("LEPTOS_OUTPUT_NAME", "remote_wol"); // required for constructing the config
    }
    let conf = get_configuration(config_file).await?;
    let addr = conf.leptos_options.site_address;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> }).await;
    let leptos_options = conf.leptos_options;

    let app = Router::new()
        .route("/api/*fn_name", post(handle_server_fns))
        .route_service("/pkg/*file", static_handler.into_service())
        .leptos_routes(leptos_options.clone(), routes, |cx| view! { cx, <App/> })
        .fallback_service(get(not_found))
        .layer(Extension(Arc::new(leptos_options)));

    log!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| e.into())
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.ends_with("_bg.wasm") {
        path = path.replace("_bg.wasm", ".wasm");
    }

    StaticFile(path)
}

async fn not_found() -> Html<&'static str> {
    Html("<h1>404</h1><p>Not Found</p>")
}

/// Embed assets into binary

#[derive(RustEmbed)]
#[folder = "target/site/pkg/"]
#[prefix = "pkg/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}

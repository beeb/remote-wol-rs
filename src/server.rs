use std::{env, net::IpAddr, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use axum::{
    body::{boxed, Full},
    error_handling::HandleErrorLayer,
    extract::Extension,
    handler::HandlerWithoutStateExt,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::post,
    BoxError, Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, handle_server_fns, LeptosRoutes};
use once_cell::sync::OnceCell;
use rust_embed::RustEmbed;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};
use wol::MacAddr;

use crate::{app::*, cli::Args};

pub struct Settings {
    pub mac_address: MacAddr,
    pub passphrase: String,
    pub ip_address: Option<IpAddr>,
}

pub static SETTINGS: OnceCell<Settings> = OnceCell::new();

fn parse_args(args: Args) -> Result<Settings> {
    let port_number = env::var("WOL_PORT")
        .ok()
        .map(|p| p.parse().ok())
        .flatten()
        .unwrap_or(args.port);
    let host = env::var("WOL_HOST").ok().unwrap_or_else(|| {
        args.host
            .then_some("0.0.0.0")
            .unwrap_or("127.0.0.1")
            .to_string()
    });
    env::set_var("LEPTOS_SITE_ADDR", format!("{host}:{port_number}"));

    let ip_address = args.ip_address.or_else(|| env::var("WOL_IP_ADDRESS").ok());
    let ip_address: Option<IpAddr> = match ip_address {
        Some(ip) => Some(ip.parse()?),
        None => None,
    };

    let mac_address: MacAddr = args
        .mac_address
        .or_else(|| env::var("WOL_MAC_ADDRESS").ok())
        .ok_or_else(|| anyhow!("MAC address not set"))?
        .parse()
        .map_err(|_| anyhow!("Invalid MAC address syntax"))?;

    let passphrase = args
        .passphrase
        .or_else(|| env::var("WOL_PASSPHRASE").ok())
        .ok_or_else(|| anyhow!("passphrase not set"))?;

    if passphrase.len() < 8 {
        return Err(anyhow!("Passphrase must be at least 8 characters long"));
    }

    Ok(Settings {
        mac_address,
        passphrase,
        ip_address,
    })
}

pub async fn server_start(args: Args) -> Result<()> {
    let settings = parse_args(args)?;
    SETTINGS
        .set(settings)
        .map_err(|_| anyhow!("Could not set global Settings"))?;

    env::set_var("LEPTOS_OUTPUT_NAME", "remote_wol"); // required for constructing the config
    let conf = get_configuration(None).await?;
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> }).await;
    let leptos_options = conf.leptos_options;

    register_server_functions();

    let app = Router::new()
        .route("/api/*fn_name", post(handle_server_fns))
        .leptos_routes(leptos_options.clone(), routes, |cx| view! { cx, <App/> })
        .fallback_service(static_handler.into_service()) // static files
        .layer(Extension(Arc::new(leptos_options)))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {err}"),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(100, Duration::from_secs(60))),
        );

    log!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| e.into())
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();
    StaticFile(path)
}

/// Embed assets into binary
#[derive(RustEmbed)]
#[folder = "target/site/"]
struct Asset;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct PublicAsset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()).or_else(|| PublicAsset::get(path.as_str())) {
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

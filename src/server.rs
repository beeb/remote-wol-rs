use std::{env, net::IpAddr, path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use axum::{
    body::{boxed, Full},
    extract::Extension,
    handler::HandlerWithoutStateExt,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, handle_server_fns, LeptosRoutes};
use rust_embed::RustEmbed;
use serde::Serialize;
use wol::MacAddr;

use crate::{app::*, cli::Args, ping::Pinger};

struct Settings {
    mac_address: MacAddr,
    passphrase: String,
    ip_address: Option<IpAddr>,
}

fn parse_args(args: Args) -> Result<Settings> {
    match args
        .site_addr
        .or_else(|| env::var("LEPTOS_SITE_ADDRESS").ok())
    {
        Some(site_addr) => env::set_var("LEPTOS_SITE_ADDRESS", site_addr),
        None => {}
    };

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

    Ok(Settings {
        mac_address,
        passphrase,
        ip_address,
    })
}

pub async fn server_start(args: Args) -> Result<()> {
    let settings = Arc::new(parse_args(args)?);

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
        .route("/api/ping", get(ping_handler))
        .route("/api/*fn_name", post(handle_server_fns))
        .route_service("/favicon.ico", static_handler.into_service())
        .route_service("/pkg/*file", static_handler.into_service())
        .leptos_routes(leptos_options.clone(), routes, |cx| view! { cx, <App/> })
        .fallback(fallback)
        .layer(Extension(Arc::new(leptos_options)))
        .layer(Extension(settings));

    log!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| e.into())
}

#[derive(Serialize)]
struct PingResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn ping_handler(Extension(settings): Extension<Arc<Settings>>) -> impl IntoResponse {
    let Some(ip_address) = settings.ip_address else {
        return Json(PingResponse { success: false, error: None });
    };
    let Ok(pinger) = Pinger::new() else {
        // probably due to lack of permissions
        // ping needs root or CAP_NET_RAW capability set on the binary
        return Json(PingResponse { success: false, error: Some("Operation not permitted".to_string()) });
    };
    let success = match pinger.ping(ip_address, None).await {
        Ok(_) => true,
        Err(_) => false,
    };
    return Json(PingResponse {
        success,
        error: None,
    });
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();
    StaticFile(path)
}

async fn fallback() -> (StatusCode, Html<&'static str>) {
    (StatusCode::NOT_FOUND, Html("<h1>404</h1><p>Not Found</p>"))
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

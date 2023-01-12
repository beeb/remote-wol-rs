use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::sync::Arc;

        use anyhow::Result;
        use leptos::*;
        use remote_wol::app::*;
        use axum::{body::{boxed, Full}, handler::HandlerWithoutStateExt, routing::{get, post}, Router, extract::Extension, response::{Html, IntoResponse, Response}, http::{header, StatusCode, Uri}};
        use rust_embed::RustEmbed;
        use leptos_axum::{LeptosRoutes, generate_route_list, handle_server_fns};


        #[tokio::main]
        async fn main() -> Result<()> {
            let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
            let addr = conf.leptos_options.site_address.clone();
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
                .await.map_err(|e| e.into())
        }


        async fn static_handler(uri: Uri) -> impl IntoResponse {
            log!("static_handler: {:?}", uri);
            let mut path = uri.path().trim_start_matches('/').to_string();
            log!("path: {:?}", path);

            if path.starts_with("pkg/") {
                path = path.replace("pkg/", "");
            }

            StaticFile(path)
        }

        async fn not_found() -> Html<&'static str> {
            Html("<h1>404</h1><p>Not Found</p>")
        }

        #[derive(RustEmbed)]
        #[folder = "target/site/pkg/"]
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
                    Response::builder().header(header::CONTENT_TYPE, mime.as_ref()).body(body).unwrap()
                }
                None => Response::builder().status(StatusCode::NOT_FOUND).body(boxed(Full::from("404"))).unwrap(),
                }
            }
        }
    }
    else {
        pub fn main() {
            // no client-side main function
            // unless we want this to work with e.g., Trunk for pure client-side testing
            // see lib.rs for hydration function instead
        }
    }
}

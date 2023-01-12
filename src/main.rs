use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::sync::Arc;

        use anyhow::Result;
        use leptos::*;
        use remote_wol::app::*;
        use axum::{routing::post, Router, extract::Extension};
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
                .leptos_routes(leptos_options.clone(), routes, |cx| view! { cx, <App/> })
                .layer(Extension(Arc::new(leptos_options)));

            log!("listening on {}", addr);
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await.map_err(|e| e.into())
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

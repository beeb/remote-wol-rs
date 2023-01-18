use std::time::Duration;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::{ping::Pinger, server::SETTINGS};
#[cfg(feature = "ssr")]
use axum::http::StatusCode;

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    _ = WakeUp::register();
    _ = Ping::register();
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,

        <Stylesheet id="leptos" href="/pkg/remote_wol.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Title text="Remote Wake-on-LAN" />
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <MainView/> }/>
                </Routes>
            </main>
        </Router>
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WakeUpResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[server(WakeUp, "/api")]
pub async fn wake_up(cx: Scope, passphrase: String) -> Result<WakeUpResponse, ServerFnError> {
    let response = use_context::<leptos_axum::ResponseOptions>(cx)
        .expect("to have leptos_axum::ResponseOptions provided");
    let Some(settings) = SETTINGS.get() else {
        response.set_status(StatusCode::INTERNAL_SERVER_ERROR).await;
        return Ok(WakeUpResponse {
            success: false,
            error: Some("Settings not initialized".to_string()),
        });
    };
    if settings.passphrase != passphrase {
        response.set_status(StatusCode::BAD_REQUEST).await;
        return Ok(WakeUpResponse {
            success: false,
            error: Some("Wrong passphrase".to_string()),
        });
    }
    // TDOD: wake up
    Ok(WakeUpResponse {
        success: true,
        error: None,
    })
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PingResponse {
    pub success: bool,
    pub warning: Option<String>,
    pub error: Option<String>,
}

#[server(Ping, "/api")]
async fn ping(cx: Scope) -> Result<PingResponse, ServerFnError> {
    let response = use_context::<leptos_axum::ResponseOptions>(cx)
        .expect("to have leptos_axum::ResponseOptions provided");
    let Some(settings) = SETTINGS.get() else {
        response.set_status(StatusCode::INTERNAL_SERVER_ERROR).await;
        return Ok(PingResponse {
            success: false,
            error: Some("Settings not initialized".to_string()),
            ..Default::default()
        });
    };
    let Some(ip_address) = settings.ip_address else {
        return Ok(PingResponse {
            success: false,
            warning: Some("IP Address was not indicated".to_string()),
            ..Default::default()
        });
    };
    let Ok(pinger) = Pinger::new() else {
        // probably due to lack of permissions
        // ping needs root or CAP_NET_RAW capability set on the binary
        response.set_status(StatusCode::INTERNAL_SERVER_ERROR).await;
        return Ok(PingResponse {
            success: false,
            error: Some("Operation not permitted".to_string()),
            ..Default::default()
        });
    };
    let success = match pinger.ping(ip_address, None).await {
        Ok(_) => true,
        Err(_) => false,
    };
    return Ok(PingResponse {
        success,
        ..Default::default()
    });
}

#[component]
fn MainView(cx: Scope) -> impl IntoView {
    let wake_up = create_server_action::<WakeUp>(cx);
    let ping = create_server_action::<Ping>(cx);
    let (passphrase, set_passphrase) = create_signal(cx, String::new());
    let on_input = move |ev| set_passphrase(event_target_value(&ev));
    let submit_disabled = move || passphrase().len() < 8;
    let ping_result = ping.value();
    let online = move || {
        ping_result()
            .and_then(|r| r.ok())
            .map(|r| r.success)
            .unwrap_or(false)
    };

    // this doesn't work
    /* let _ = set_interval(
        move || {
            ping.dispatch(Ping {});
        },
        Duration::from_millis(2500),
    ); */

    view! { cx,
    <div class="flex justify-center items-center min-h-screen min-w-screen p-8">
        <div class="card flex flex-col w-full max-w-md p-6 gap-4">
            <h1 class="text-2xl font-bold text-center">"Remote Wake-on-LAN"</h1>
            <div class="flex items-center gap-4 text-lg">
                <div>"Device status:"</div>
                    {move || if online() {
                        view! {
                            cx,
                            <div class="text-success flex items-center gap-2">
                                "online"
                            </div>
                        }
                    } else {
                        view! {
                            cx,
                            <div class="text-error flex items-center gap-2">
                                "offline"
                            </div>
                        }
                    }}
            </div>
            <ActionForm action=wake_up>
                <div class="flex flex-col gap-4">
                    <div class="flex items-center gap-4 flex-wrap">
                        <label class="text-lg" for="passphrase">"Passphrase:"</label>
                        <input
                            class="grow rounded-lg text-lg p-2 border border-slate-300"
                            type="password"
                            id="passphrase"
                            name="passphrase"
                            prop:value=passphrase
                            on:input=on_input
                        />
                    </div>
                    <div class="flex justify-center">
                        <button
                            class="primary p-3 rounded-lg text-lg uppercase disabled:opacity-50 disabled:cursor-not-allowed"
                            type="submit"
                            prop:disabled=submit_disabled
                        >
                            "Wake up!"
                        </button>
                    </div>
                </div>
            </ActionForm>
        </div>
    </div>
    }
}

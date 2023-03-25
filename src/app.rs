#![allow(unused_variables)] // fixes a glitch warning about `cx` not being used in server functions
#![allow(clippy::let_with_type_underscore)]
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
use wol::send_wol;

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
    pub success: Option<bool>,
    pub error: Option<String>,
}

#[server(WakeUp, "/api")]
pub async fn wake_up(cx: Scope, passphrase: String) -> Result<WakeUpResponse, ServerFnError> {
    let response = use_context::<leptos_axum::ResponseOptions>(cx)
        .expect("to have leptos_axum::ResponseOptions provided");
    let Some(settings) = SETTINGS.get() else {
        response.set_status(StatusCode::FAILED_DEPENDENCY);
        return Ok(WakeUpResponse {
            success: Some(false),
            error: Some("Settings not initialized".to_string()),
        });
    };
    if settings.passphrase != passphrase {
        response.set_status(StatusCode::BAD_REQUEST);
        return Ok(WakeUpResponse {
            success: None,
            error: Some("Wrong passphrase".to_string()),
        });
    }
    match send_wol(settings.mac_address, None, None) {
        Ok(_) => Ok(WakeUpResponse {
            success: Some(true),
            error: None,
        }),
        Err(e) => {
            // ideally this would be a 5xx but leptos loses the response body on 5xx and returns a ServerFnError instead
            response.set_status(StatusCode::BAD_REQUEST);
            Ok(WakeUpResponse {
                success: Some(false),
                error: Some(format!("Error sending WOL packet: {e}")),
            })
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
        response.set_status(StatusCode::FAILED_DEPENDENCY);
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
        // ideally this would be a 5xx but leptos loses the response body on 5xx and returns a ServerFnError instead
        // Error probably due to lack of permissions
        // ping needs root or CAP_NET_RAW capability set on the binary
        response.set_status(StatusCode::FORBIDDEN);
        return Ok(PingResponse {
            success: false,
            error: Some("Operation not permitted".to_string()),
            ..Default::default()
        });
    };
    let success = pinger.ping(ip_address, None).await.is_ok();
    Ok(PingResponse {
        success,
        ..Default::default()
    })
}

#[component]
fn MainView(cx: Scope) -> impl IntoView {
    let wake_up = create_server_action::<WakeUp>(cx);
    let ping = create_server_action::<Ping>(cx);
    let (passphrase, set_passphrase) = create_signal(cx, String::new());
    let on_input = move |ev| set_passphrase(event_target_value(&ev));
    let online = move || {
        let ping_result = ping.value();
        match ping_result.get().and_then(|r| r.ok()) {
            Some(r) => {
                if r.error.is_some() || r.warning.is_some() {
                    return None;
                }
                Some(r.success)
            }
            None => None,
        }
    };
    let submit_disabled = move || passphrase().len() < 8 || Some(true) == online();
    let online_error = move || {
        let ping_result = ping.value();
        ping_result.get().and_then(|r| r.ok()).and_then(|r| r.error)
    };
    let online_warning = move || {
        let ping_result = ping.value();
        ping_result
            .get()
            .and_then(|r| r.ok())
            .and_then(|r| r.warning)
    };
    let wakeup_status = move || {
        let wakeup_result = wake_up.value();
        let res = wakeup_result
            .get()
            .and_then(|r| r.ok())
            .and_then(|r| r.success);
        if Some(true) == res {
            set_passphrase(String::new());
        }
        res
    };
    let wakeup_error = move || {
        let wakeup_result = wake_up.value();
        wakeup_result
            .get()
            .and_then(|r| r.ok())
            .and_then(|r| r.error)
    };

    if cfg!(not(feature = "ssr")) {
        ping.dispatch(Ping {});
        let _ = set_interval_with_handle(
            move || {
                ping.dispatch(Ping {});
            },
            Duration::from_millis(2500),
        );
    }

    view! { cx,
    <div class="container flex justify-center items-center min-w-full p-8">
        <div class="card flex flex-col w-full max-w-md p-6 gap-4">
            <h1 class="text-2xl font-bold text-center">"Remote Wake-on-LAN"</h1>
            {
                move || {wakeup_status().map(|success| {
                    if success && Some(true) != online() {
                        Some(view! {
                            cx,
                            <div class="text-lg success p-2 rounded-lg text-center">
                                "Magic packet was sent. Please wait for the computer to wake up."
                            </div>
                        })
                    } else if !success {
                        Some(view! {
                            cx,
                            <div class="text-lg error p-2 rounded-lg text-center">
                                "Failed to send magic packet. "
                                {wakeup_error}
                            </div>
                        })
                    } else {
                        None
                    }
                })}
            }
            <div class="flex items-center gap-4 text-lg">
                <div>"Device status:"</div>
                    {move || if let Some(ol) = online() {
                        if ol {
                            view! {
                                cx,
                                <div class="flex items-center gap-2">
                                    <div class="badge badge-success">"online"</div>
                                </div>
                            }
                        } else {
                            view! {
                                cx,
                                <div class="flex items-center gap-2">
                                    <div class="badge badge-danger">
                                        "offline"
                                    </div>
                                    {move || match wakeup_status() {
                                        Some(true) => Some(view! {
                                            cx,
                                            <div class="loader" />
                                        }),
                                        _ => None
                                    }}
                                </div>
                            }
                        }
                    } else {
                        view! {
                            cx,
                            <div class="flex items-center gap-2">
                                <div class="badge badge-primary">"unknown"</div>
                                <span class="text-sm">
                                    {online_error().unwrap_or_default()}
                                    {online_warning().unwrap_or_default()}
                                </span>
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
                            prop:disabled=move || Some(true) == online()
                            on:input=on_input
                        />
                        {move || {
                            match wakeup_status() {
                                None => {
                                    wakeup_error().map(|err| view! {
                                        cx,
                                        <div class="text-error grow text-center">{err}</div>
                                    })
                                },
                                Some(_) => None
                            }
                        }}
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

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[cfg(feature = "ssr")]
use crate::server::SETTINGS;

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    _ = WakeUp::register();
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

#[server(WakeUp, "/api")]
pub async fn wake_up(passphrase: String) -> Result<(), ServerFnError> {
    let Some(settings) = SETTINGS.get() else {
        return Err(ServerFnError::ServerError("Settings not initialized".to_string()));
    };
    if settings.passphrase != passphrase {
        return Err(ServerFnError::ServerError("Wrong passphrase".to_string()));
    }
    // TDOD: wake up
    Ok(())
}

#[component]
fn MainView(cx: Scope) -> impl IntoView {
    let wake_up = create_server_action::<WakeUp>(cx);
    let (passphrase, set_passphrase) = create_signal(cx, String::new());
    let on_input = move |ev| set_passphrase(event_target_value(&ev));
    let submit_disabled = move || passphrase().len() < 8;

    view! { cx,
    <div class="flex justify-center items-center min-h-screen min-w-screen p-8">
        <div class="card flex flex-col w-full max-w-md p-6 gap-4">
            <h1 class="text-2xl font-bold text-center">"Remote Wake-on-LAN"</h1>
            <div class="flex items-center gap-4 text-lg">
                <div>"Device status:"</div>
                    <div class="text-success flex items-center gap-2">
                        "online"
                    </div>
                    <div class="text-error flex items-center gap-2">
                        "offline"
                    </div>
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

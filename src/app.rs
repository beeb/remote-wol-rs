use leptos::*;
use leptos_meta::*;
use leptos_router::*;

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

#[component]
fn MainView(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <div class="flex flex-col w-full items-center p-6">
            <div class="text-center">
                <h1 class="text-3xl mb-4 w-auto">"Welcome to Leptos!"</h1>
                <button on:click=on_click class="bg-gray-300 border border-gray-500 p-2 rounded">
                    "Click Me: " {count}
                </button>
            </div>
        </div>
    }
}

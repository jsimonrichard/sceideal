use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/sceideal.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                    <Route path="/login" view=|cx| view! { cx, <Login/> }/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="p-2 center-vertically">
            <div class="panel small-panel is-primary">
                <p class="panel-heading">"Schedule a CPT session"</p>
                <div class="panel-block">
                    "Appointment Type:"
                    <div class="select pl-3">
                        <select class="is-disabled">
                        </select>
                    </div>
                </div>
                <div class="panel-block is-justify-content-end">
                    <a class="button is-primary">"Next"</a>
                </div>
            </div>
        </div>
        <a class="button m-2" style="position: absolute; right: 0px; bottom: 0px" href="/login">"Provider Login"</a>
    }
}

#[component]
fn Login(cx: Scope) -> impl IntoView {
    view! {cx,
        <div class="p-2 center-vertically">
            <div class="panel small-panel is-primary">
                <p class="panel-header">"Login"</p>
                <div class="panel-block">

                </div>
            </div>
        </div>
    }
}

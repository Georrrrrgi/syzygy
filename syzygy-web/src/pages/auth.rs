use leptos::*;
use leptos_router::*;
use crate::api;

#[component]
pub fn Auth() -> impl IntoView {
    let location = use_location();
    let is_login = move || location.pathname.get().contains("login");

    let username = create_rw_signal(String::new());
    let display_name = create_rw_signal(String::new());
    let bio = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let error = create_rw_signal(String::new());
    let is_loading = create_rw_signal(false);

    let submit = move |_| {
        let u = username();
        let p = password();
        let d = display_name();
        let b = bio();

        if u.is_empty() || p.is_empty() {
            error.set("Username and password are required.".into());
            return;
        }

        is_loading.set(true);
        error.set(String::new());

        wasm_bindgen_futures::spawn_local({
            let is_login = is_login();
            let u = u.clone();
            let p = p.clone();
            let d = d.clone();
            let b = b.clone();
            let error = error;
            let is_loading = is_loading;
            async move {
                let result = if is_login {
                    api::login(&u, &p).await
                } else {
                    api::register(&u, &d, &b, &p).await
                };

                match result {
                    Ok(resp) => {
                        let doc = web_sys::window().unwrap().document().unwrap();
                        let cookie = format!("syzygy_token={}; path=/; max-age={}", resp.token, 604800);
                        doc.set_cookie(&cookie).ok();
                        let nav = leptos_router::use_navigate();
                        nav("/", Default::default());
                    }
                    Err(e) => error.set(e),
                }
                is_loading.set(false);
            }
        });
    };

    view! {
        <div class="auth-page">
            <div class="auth-card">
                <div class="auth-title">"SYZYGY"</div>
                <div class="auth-subtitle">
                    {move || if is_login() { "Welcome back, celestial being." } else { "Join the alignment." }}
                </div>
                <form class="auth-form" on:submit=|ev| ev.prevent_default()>
                    <input
                        class="input"
                        type="text"
                        placeholder="Username"
                        prop:value=username
                        on:input=move |ev| username.set(event_target_value(&ev))
                    />
                    {move || if !is_login() {
                        view! {
                            <input
                                class="input"
                                type="text"
                                placeholder="Display Name"
                                prop:value=display_name
                                on:input=move |ev| display_name.set(event_target_value(&ev))
                            />
                            <textarea
                                class="input textarea"
                                placeholder="Bio (optional)"
                                prop:value=bio
                                on:input=move |ev| bio.set(event_target_value(&ev))
                            />
                        }
                    } else {
                        view! {}.into_view()
                    }}
                    <input
                        class="input"
                        type="password"
                        placeholder="Password"
                        prop:value=password
                        on:input=move |ev| password.set(event_target_value(&ev))
                    />
                    {move || if !error().is_empty() {
                        view! { <p style="color: var(--danger); font-size: 13px;">{error()}</p> }
                    } else {
                        view! {}.into_view()
                    }}
                    <button
                        class="btn btn-primary btn-block"
                        disabled=move || is_loading()
                        on:click=submit
                    >
                        {move || if is_loading() { "..." } else if is_login() { "Enter" } else { "Align" }}
                    </button>
                </form>
                <div class="auth-link">
                    {move || if is_login() {
                        view! { <A href="/register">"No account? Join the syzygy."</A> }
                    } else {
                        view! { <A href="/login">"Already aligned? Enter."</A> }
                    }}
                </div>
            </div>
        </div>
    }
}

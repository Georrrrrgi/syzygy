use leptos::*;
use leptos_router::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="nav">
            <div class="nav-brand">S Y Z Y G Y</div>
            <A href="/" class="nav-item">"Home"</A>
            <A href="/explore" class="nav-item">"Explore"</A>
            <A href="/profile/me" class="nav-item">"Profile"</A>
            <div style="margin-top: 24px;">
                <a href="/login" class="btn btn-primary btn-block">"Sign In"</a>
            </div>
        </nav>
    }
}

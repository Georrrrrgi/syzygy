use leptos::*;
use leptos_router::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="empty-state" style="margin-top: 64px;">
            <h3>"404 — This page is not in alignment."</h3>
            <p>"The celestial bodies have not converged here."</p>
            <div style="margin-top: 16px;">
                <A href="/" class="btn btn-primary">"Return to the Syzygy"</A>
            </div>
        </div>
    }
}

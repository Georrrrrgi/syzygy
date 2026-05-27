use leptos::*;
use crate::api;
use crate::models::PostEnriched;
use crate::components::feed::Feed;

#[component]
pub fn Explore() -> impl IntoView {
    let posts = create_rw_signal(Vec::<PostEnriched>::new());
    let is_loading = create_rw_signal(true);

    wasm_bindgen_futures::spawn_local({
        let posts = posts;
        let is_loading = is_loading;
        async move {
            match api::get_explore(50, 0).await {
                Ok(p) => posts.set(p),
                Err(_) => {}
            }
            is_loading.set(false);
        }
    });

    view! {
        <div>
            <h2 style="font-size: 20px; font-weight: 700; margin-bottom: 16px;">"Explore"</h2>
            <Feed posts=posts() is_loading=is_loading() />
        </div>
    }
}

use leptos::*;
use leptos_router::*;
use crate::api;
use crate::models::PostEnriched;
use crate::components::post_card::PostCard;

#[component]
pub fn PostPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params().get("id").cloned().unwrap_or_default();

    let post = create_rw_signal(None::<PostEnriched>);
    let is_loading = create_rw_signal(true);

    wasm_bindgen_futures::spawn_local({
        let id = id();
        let post = post;
        let is_loading = is_loading;
        async move {
            if let Ok(p) = api::get_post(&id).await {
                post.set(Some(p));
            }
            is_loading.set(false);
        }
    });

    view! {
        <div>
            {move || {
                if is_loading() {
                    return view! { <div class="empty-state"><h3>"Contemplating the syzygy..."</h3></div> }.into_view();
                }
                match post() {
                    Some(p) => view! { <PostCard post=p on_like=None/> }.into_view(),
                    None => view! { <div class="empty-state"><h3>"This syzygy does not exist."</h3></div> }.into_view(),
                }
            }}
        </div>
    }
}

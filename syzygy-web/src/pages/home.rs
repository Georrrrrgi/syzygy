use leptos::*;
use crate::api;
use crate::models::PostEnriched;
use crate::components::{compose::ComposeBox, feed::Feed};

#[component]
pub fn Home() -> impl IntoView {
    let posts = create_rw_signal(Vec::<PostEnriched>::new());
    let is_loading = create_rw_signal(true);

    let fetch = {
        let posts = posts;
        let is_loading = is_loading;
        move || {
            is_loading.set(true);
            wasm_bindgen_futures::spawn_local({
                let posts = posts;
                let is_loading = is_loading;
                async move {
                    match api::get_feed(50, 0).await {
                        Ok(p) => posts.set(p),
                        Err(_) => {}
                    }
                    is_loading.set(false);
                }
            });
        }
    };

    fetch();

    view! {
        <div>
            <ComposeBox on_post=Callback::new(move |_| fetch()) />
            <Feed posts=posts() is_loading=is_loading() />
        </div>
    }
}

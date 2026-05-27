use leptos::*;
use leptos_router::*;
use crate::api;
use crate::models::{UserProfile, PostEnriched};
use crate::components::feed::Feed;

#[component]
pub fn Profile() -> impl IntoView {
    let params = use_params_map();
    let id = move || params().get("id").cloned().unwrap_or_default();

    let profile = create_rw_signal(None::<UserProfile>);
    let posts = create_rw_signal(Vec::<PostEnriched>::new());
    let is_loading = create_rw_signal(true);
    let is_following = create_rw_signal(false);

    wasm_bindgen_futures::spawn_local({
        let id = id();
        let profile = profile;
        let posts = posts;
        let is_loading = is_loading;
        async move {
            if let Ok(p) = api::get_user(&id).await {
                profile.set(Some(p));
            }
            if let Ok(p) = api::get_user_posts(&id, 50, 0).await {
                posts.set(p);
            }
            is_loading.set(false);
        }
    });

    view! {
        <div>
            {move || profile().map(|p| {
                view! {
                    <div class="profile-header">
                        <div class="profile-avatar">
                            {p.user.username.chars().next().unwrap_or('S').to_uppercase()}
                        </div>
                        <div>
                            <div class="profile-name">{p.user.display_name}</div>
                            <div class="profile-bio">"@" {p.user.username}</div>
                            <div class="profile-bio">{p.user.bio}</div>
                            <div class="profile-stats">
                                <span class="profile-stat">
                                    <strong>{p.post_count}</strong> " Posts"
                                </span>
                                <span class="profile-stat">
                                    <strong>{p.follower_count}</strong> " Followers"
                                </span>
                                <span class="profile-stat">
                                    <strong>{p.following_count}</strong> " Following"
                                </span>
                            </div>
                        </div>
                    </div>
                }
            })}
            <h3 style="font-size: 16px; font-weight: 600; margin-bottom: 12px;">"Syzygies"</h3>
            <Feed posts=posts() is_loading=is_loading() />
        </div>
    }
}

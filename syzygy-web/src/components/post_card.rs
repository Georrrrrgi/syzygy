use leptos::*;
use leptos_router::*;
use crate::models::PostEnriched;
use crate::api;

#[component]
pub fn PostCard(post: PostEnriched, on_like: Option<Callback<(String, bool)>>) -> impl IntoView {
    let is_liked = create_rw_signal(post.is_liked);
    let like_count = create_rw_signal(post.like_count);
    let post_id = post.id.to_string();
    let initial_liked = post.is_liked;

    let time_ago = timeago(post.created_at);

    view! {
        <div class="post-card">
            <div class="post-header">
                <div class="post-avatar">
                    {post.author.username.chars().next().unwrap_or('S').to_uppercase()}
                </div>
                <div>
                    <div class="post-author">{post.author.display_name}</div>
                    <div class="post-username">"@" {post.author.username}</div>
                </div>
                <div style="margin-left: auto;">
                    <div class="post-time">{time_ago}</div>
                </div>
            </div>
            <div class="post-content">
                <A href=format!("/post/{}", post.id)>{post.content}</A>
            </div>
            <div class="post-actions">
                <button
                    class=move || format!("post-action {}", if is_liked() { "liked" } else { "" })
                    on:click=move |_| {
                        let id = post_id.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            if initial_liked {
                                api::unlike_post(&id).await.ok();
                            } else {
                                api::like_post(&id).await.ok();
                            }
                        });
                        if initial_liked {
                            is_liked.set(false);
                            like_count.set(like_count() - 1);
                        } else {
                            is_liked.set(true);
                            like_count.set(like_count() + 1);
                        }
                    }
                >
                    {move || if is_liked() { "♥" } else { "♡" }}
                    <span>{move || like_count()}</span>
                </button>
                <span class="post-action">{post.syzygy_count} " ⟳"</span>
            </div>
        </div>
    }
}

fn timeago(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now - dt;
    let secs = diff.num_seconds();
    if secs < 60 { return format!("{}s", secs); }
    let mins = diff.num_minutes();
    if mins < 60 { return format!("{}m", mins); }
    let hours = diff.num_hours();
    if hours < 24 { return format!("{}h", hours); }
    let days = diff.num_days();
    format!("{}d", days)
}

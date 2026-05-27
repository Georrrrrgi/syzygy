use leptos::*;
use crate::models::PostEnriched;
use crate::components::post_card::PostCard;

#[component]
pub fn Feed(posts: Vec<PostEnriched>, is_loading: bool) -> impl IntoView {
    if is_loading {
        return view! {
            <div class="empty-state">
                <h3>"Aligning the celestial bodies..."</h3>
            </div>
        }.into_view();
    }

    if posts.is_empty() {
        return view! {
            <div class="empty-state">
                <h3>"The void stares back"</h3>
                <p>"Nothing has been syzygied yet. Be the first to align."</p>
            </div>
        }.into_view();
    }

    view! {
        <For
            each=move || posts.clone()
            key=|p| p.id
            children=move |post| {
                view! { <PostCard post=post on_like=None/> }
            }
        />
    }.into_view()
}

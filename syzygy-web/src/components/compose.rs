use leptos::*;
use crate::api;

#[component]
pub fn ComposeBox(on_post: Callback<String>) -> impl IntoView {
    let content = create_rw_signal(String::new());
    let is_loading = create_rw_signal(false);
    let error = create_rw_signal(String::new());
    let max_chars: i64 = 777;

    let chars_left = move || max_chars - content().len() as i64;

    let can_submit = move || {
        let c = content();
        !c.trim().is_empty() && c.len() <= max_chars as usize && !is_loading()
    };

    view! {
        <div class="compose-box">
            <textarea
                class="compose-textarea"
                placeholder="What is the nature of your alignment?"
                prop:value=content
                on:input=move |ev| {
                    content.set(event_target_value(&ev));
                }
            />
            <div class="compose-footer">
                <div class=move || format!("compose-char {}", if chars_left() < 20 { "warning" } else { "" })>
                    {chars_left}
                </div>
                <button
                    class="btn btn-primary"
                    disabled=move || !can_submit()
                    on:click=move |_| {
                        let text = content();
                        if text.trim().is_empty() { return; }
                        is_loading.set(true);
                        error.set(String::new());

                        let text_clone = text.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match api::create_post(&text_clone).await {
                                Ok(_) => {
                                    on_post.call(text_clone);
                                    content.set(String::new());
                                }
                                Err(e) => error.set(e),
                            }
                            is_loading.set(false);
                        });
                    }
                >
                    {move || if is_loading() { "Aligning..." } else { "Syzygy" }}
                </button>
            </div>
            {move || if !error().is_empty() {
                view! { <p style="color: var(--danger); font-size: 13px; margin-top: 8px;">{error()}</p> }
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

use leptos::*;
use leptos_router::*;
use crate::components::nav::Nav;
use crate::pages::{home::Home, explore::Explore, profile::Profile, post::PostPage, auth::Auth, not_found::NotFound};

#[component]
pub fn App() -> impl IntoView {
    let is_auth = create_rw_signal(false);

    provide_context(is_auth);

    view! {
        <Router>
            <Routes>
                <Route path="/login" view=Auth/>
                <Route path="/register" view=Auth/>
                <Route path="/" view=move || {
                    view! {
                        <div class="sidebar-layout">
                            <Nav/>
                            <main class="main-content">
                                <Outlet/>
                            </main>
                        </div>
                    }
                }>
                    <Route path="" view=Home/>
                    <Route path="explore" view=Explore/>
                    <Route path="profile/:id" view=Profile/>
                    <Route path="post/:id" view=PostPage/>
                </Route>
                <Route path="/*" view=NotFound/>
            </Routes>
        </Router>
    }
}

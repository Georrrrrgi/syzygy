use crate::models::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::wasm_bindgen;
use wasm_bindgen::JsCast;

const API_BASE: &str = "http://127.0.0.1:8080/api";

fn get_token() -> Option<String> {
    let doc = web_sys::window()?.document()?;
    let cookie = doc.cookie().ok()?;
    cookie.split(';')
        .find_map(|c| {
            let parts: Vec<&str> = c.trim().split('=').collect();
            if parts.len() == 2 && parts[0] == "syzygy_token" {
                Some(parts[1].to_string())
            } else {
                None
            }
        })
}

async fn api_get<T: for<'de> serde::Deserialize<'de>>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, path);
    let mut req = Request::get(&url);
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    resp.json().await.map_err(|e| e.to_string())
}

async fn api_post<T: for<'de> serde::Deserialize<'de>, B: serde::Serialize>(path: &str, body: &B) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, path);
    let mut req = Request::post(&url).header("Content-Type", "application/json");
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let body_str = serde_json::to_string(body).map_err(|e| e.to_string())?;
    let resp = req.body(body_str).send().await.map_err(|e| e.to_string())?;
    resp.json().await.map_err(|e| e.to_string())
}

async fn api_delete(path: &str) -> Result<(), String> {
    let url = format!("{}{}", API_BASE, path);
    let mut req = Request::delete(&url);
    if let Some(token) = get_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    req.send().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn login(username: &str, password: &str) -> Result<AuthResponse, String> {
    api_post("/auth/login", &LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    }).await
}

pub async fn register(username: &str, display_name: &str, bio: &str, password: &str) -> Result<AuthResponse, String> {
    api_post("/auth/register", &RegisterRequest {
        username: username.to_string(),
        display_name: display_name.to_string(),
        bio: bio.to_string(),
        password: password.to_string(),
    }).await
}

pub async fn get_me() -> Result<User, String> {
    api_get("/auth/me").await
}

pub async fn get_feed(limit: i64, offset: i64) -> Result<Vec<PostEnriched>, String> {
    api_get(&format!("/feed?limit={}&offset={}", limit, offset)).await
}

pub async fn get_explore(limit: i64, offset: i64) -> Result<Vec<PostEnriched>, String> {
    api_get(&format!("/explore?limit={}&offset={}", limit, offset)).await
}

pub async fn create_post(content: &str) -> Result<Post, String> {
    api_post("/posts", &serde_json::json!({ "content": content })).await
}

pub async fn get_post(id: &str) -> Result<PostEnriched, String> {
    api_get(&format!("/posts/{}", id)).await
}

pub async fn delete_post(id: &str) -> Result<(), String> {
    api_delete(&format!("/posts/{}", id)).await
}

pub async fn like_post(id: &str) -> Result<(), String> {
    api_post(&format!("/posts/{}/like", id), &serde_json::json!({})).await
}

pub async fn unlike_post(id: &str) -> Result<(), String> {
    api_delete(&format!("/posts/{}/like", id)).await
}

pub async fn get_user(id: &str) -> Result<UserProfile, String> {
    api_get(&format!("/users/{}", id)).await
}

pub async fn get_user_posts(id: &str, limit: i64, offset: i64) -> Result<Vec<PostEnriched>, String> {
    api_get(&format!("/users/{}/posts?limit={}&offset={}", id, limit, offset)).await
}

pub async fn follow(id: &str) -> Result<(), String> {
    api_post(&format!("/users/{}/follow", id), &serde_json::json!({})).await
}

pub async fn unfollow(id: &str) -> Result<(), String> {
    api_delete(&format!("/users/{}/follow", id)).await
}

pub async fn get_followers(id: &str) -> Result<Vec<User>, String> {
    api_get(&format!("/users/{}/followers", id)).await
}

pub async fn get_following(id: &str) -> Result<Vec<User>, String> {
    api_get(&format!("/users/{}/following", id)).await
}

pub async fn search_users(q: &str) -> Result<Vec<User>, String> {
    api_get(&format!("/users/search?q={}", q)).await
}

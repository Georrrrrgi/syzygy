use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

mod config;
mod state;
mod errors;
mod middleware;
mod routes;
mod handlers;
mod infrastructure;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "syzygy_server=debug,tower_http=debug".into()))
        .init();

    let config = config::Config::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    run_migrations(&pool).await;

    let state = state::AppState::new(pool);
    let app = routes::build_router(state);

    let addr = config.bind_address();
    tracing::info!("SYZYGY server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}

async fn run_migrations(pool: &sqlx::PgPool) {
    tracing::info!("running database migrations");

    sqlx::query(include_str!("../../migrations/001_create_users.sql"))
        .execute(pool)
        .await
        .ok();

    sqlx::query(include_str!("../../migrations/002_create_posts.sql"))
        .execute(pool)
        .await
        .ok();

    sqlx::query(include_str!("../../migrations/003_create_follows.sql"))
        .execute(pool)
        .await
        .ok();

    sqlx::query(include_str!("../../migrations/004_create_likes.sql"))
        .execute(pool)
        .await
        .ok();

    sqlx::query(include_str!("../../migrations/005_create_syzygies.sql"))
        .execute(pool)
        .await
        .ok();

    tracing::info!("migrations complete");
}

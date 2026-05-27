                                                                                
                      ███████  ██    ██ ███████  ██████  ██    ██  ██████  
                        ██    ██    ██    ██    ██       ██    ██ ██       
                        ██    ██    ██    ██    ██       ██    ██ ██       
                        ██    ██    ██    ██    ██       ██    ██ ██       
                        ██     ██████   ██     ██████    ██████   ██████  

    "The alignment of celestial bodies. The convergence of social atoms.
     A platform for those who understand that communication is architecture."

## Stack

| Layer       | Technology                                |
|-------------|-------------------------------------------|
| Domain      | Pure Rust (zero framework dependencies)   |
| API         | Axum 0.8 - Tokio - SQLx                   |
| Frontend    | Leptos 0.7 - WASM - Fine-grained signals  |
| Database    | PostgreSQL 15+                            |
| Cache       | Redis 7+ (optional)                       |
| Auth        | JWT (bcrypt + jsonwebtoken)               |
| Real-time   | WebSockets (tokio-tungstenite)            |
| Build       | Cargo workspace + Trunk                   |

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    syzygy-web                        │
│              Leptos CSR SPA → WASM                   │
│       HTTP (gloo-net) → Axum REST API               │
├─────────────────────────────────────────────────────┤
│                    syzygy-server                     │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Routes  │→ │  Handlers    │→ │Infrastructure│  │
│  │  (Axum)  │  │  (Use Cases) │  │  (Adapters)  │  │
│  └──────────┘  └──────┬───────┘  └──────────────┘  │
│                       │                             │
│                 ┌──────▼───────┐                    │
│                 │  syzygy-core  │                    │
│                 │  (Ports)     │                    │
│                 │  (Domain)    │                    │
│                 └──────────────┘                    │
├─────────────────────────────────────────────────────┤
│  PostgreSQL   │   Redis (opt)   │   WebSocket conns │
└─────────────────────────────────────────────────────┘
```

## Domain-Driven Hexagonal Architecture

The core domain (`syzygy-core`) has **zero** framework dependencies.
It defines:

- **Entities** — `User`, `Post`, `Follow`, `Like`
- **Ports** — `UserRepository` trait, `PostRepository` trait
- **Errors** — typed domain errors with `thiserror`

The server (`syzygy-server`) implements these ports as adapters.
Swap Postgres for SQLite? Change one file. Swap JWT for OAuth?
Change one module. The domain remains untouched.

This is not simple. Simplicity is for those who cannot handle complexity.

## Getting Started

### Prerequisites

```bash
# Rust (nightly for Leptos)
rustup default nightly

# PostgreSQL 15+
# Create database:
createdb syzygy

# Trunk (WASM bundler)
cargo install trunk

# WASM target
rustup target add wasm32-unknown-unknown
```

### Environment

```bash
cp .env.example .env
# Edit DATABASE_URL and JWT_SECRET
```

### Run the Server

```bash
cargo run -p syzygy-server
```

### Run the Frontend (separate terminal)

```bash
cd syzygy-web
trunk serve
```

Open http://127.0.0.1:8080 for the API.
Open http://127.0.0.1:3000 for the frontend.

## API Endpoints

### Public
- `POST /api/auth/register` — Create account
- `POST /api/auth/login` — Authenticate
- `GET /api/auth/me` — Current user
- `GET /api/ws` — WebSocket (real-time events)

### Protected (JWT required)
- `GET /api/feed` — Timeline from followed users
- `GET /api/explore` — Global post stream
- `POST /api/posts` — Create a syzygy (max 777 chars)
- `GET /api/posts/:id` — Get a post
- `DELETE /api/posts/:id` — Delete your post
- `POST /api/posts/:id/like` — Like
- `DELETE /api/posts/:id/like` — Unlike
- `GET /api/users/:id` — User profile
- `PUT /api/users/:id` — Update profile
- `POST /api/users/:id/follow` — Follow
- `DELETE /api/users/:id/follow` — Unfollow
- `GET /api/users/:id/followers` — Followers list
- `GET /api/users/:id/following` — Following list
- `GET /api/users/:id/posts` — User's posts
- `GET /api/users/search?q=` — Search users

## License

MIT — do what you want, but don't blame us when the syzygy collapses.

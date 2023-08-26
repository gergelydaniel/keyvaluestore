use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use axum::{Router, routing::{get, post}, TypedHeader};
use axum::extract::{Path, State};
use axum::headers::{Authorization, Date};
use axum::headers::authorization::Bearer;
use axum::http::StatusCode;
use axum::response::Response;
use httpdate::fmt_http_date;
use ini::ini;

struct AppConfig {
    port: u16,
    read_token: String,
    write_token: String,
}

#[derive(Clone)]
struct EntryState {
    value: String,
    modified_date: Date,
}

#[derive(Clone)]
struct AppState {
    read_token: String,
    write_token: String,
    values: Arc<Mutex<HashMap<String, EntryState>>>,
}

fn read_app_config() -> AppConfig {
    let ini = ini!("keyvaluestore.ini");
    let port = ini["keyvaluestore"]["port"].clone().unwrap().parse::<u16>().unwrap();
    let read_token = ini["keyvaluestore"]["read_token"].clone().unwrap();
    let write_token = ini["keyvaluestore"]["write_token"].clone().unwrap();

    AppConfig { port, read_token, write_token }
}

#[tokio::main]
async fn main() {
    let config = read_app_config();

    let app_state = AppState {
        read_token: config.read_token,
        write_token: config.write_token,
        values: Arc::new(Mutex::new(HashMap::new())),
    };

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/:key", get(read))
        .route("/:key", post(write))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    println!("Server started listening on port {}", config.port);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn read(
    Path(key): Path<String>,
    auth: TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
) -> Result<Response<String>, StatusCode> {
    let auth_token = auth.0.0.token();

    if auth_token == app_state.read_token {
        match app_state.values.lock().unwrap().get(&key) {
            None => Err(StatusCode::NO_CONTENT),
            Some(value) => {
                let cloned = value.clone();
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("Last-Modified", fmt_http_date(SystemTime::from(cloned.modified_date)))
                    .body(cloned.value)
                    .unwrap();

                Ok(response)
            }
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn write(
    Path(key): Path<String>,
    auth: TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    body: String,
) -> Result<(), StatusCode> {
    let auth_token = auth.0.0.token();

    if auth_token == app_state.write_token {
        let mut map = app_state.values.lock().unwrap();
        let new_entry = EntryState {
            value: body,
            modified_date: Date::from(SystemTime::now()),
        };
        map.insert(key, new_entry);
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

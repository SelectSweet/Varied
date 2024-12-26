use rand::{thread_rng, Rng};
use sea_orm::{
    prelude::DateTime, ActiveModelTrait, ActiveValue, ColumnTrait, ConnectOptions, Database,
    DatabaseConnection, DbBackend, EntityTrait, ModelTrait, QueryFilter, QuerySelect, Set,
    Statement,
};

use base64::{engine::general_purpose, *};
use chrono::{NaiveDateTime, Utc};
use sea_orm::{Condition, FromQueryResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqids::{Options, Sqids};
use std::sync::{Arc, OnceLock};
use std::{
    collections::HashMap,
    fmt,
    fmt::Display,
    fs::File,
    io::Read,
    process::{Command, Stdio},
};
use url::Url;

use axum::{
    body::Body,
    debug_handler,
    extract::{DefaultBodyLimit, Path as APath, Query, State},
    http::{header, header::*, Method, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};

use moka::future::Cache;

use tower_cookies::{Cookie, CookieManagerLayer, Cookies, Key};

// use axum_extra::extract::cookie::Cookie as CookieJ;
// use axum_extra::extract::CookieJar;
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use tempfile::NamedTempFile;

use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use uuid::Uuid;

use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegProgress};

use std::path::Path;

use migration::{Migrator, MigratorTrait};

use opendal::{
    services::{Http, S3},
    Operator,
};
use std::fs;

use crate::Image::ProcessImages;

use entity::{
    v_account, v_biscuitkey, v_collection, v_follow, v_media, v_session, v_task, v_towersession,
};

use biscuit_auth::{
    macros::{authorizer, biscuit},
    Authorizer, Biscuit, KeyPair, PrivateKey, PublicKey,
};
use sea_orm::Insert;

//static SessionKey: Cache<String, Vec<u8>> = Cache::new(1);

pub struct SessionCache {
    Key: Cache<String, Vec<u8>>,
}

mod Account;
mod Audio;
mod Auth;
mod Collection;
mod Feed;
mod Follow;
mod Image;
mod Login;
mod Media;
mod Settings;
mod Task;
mod Text;
mod Video;

use Auth::BiscuitToken::{
    create_key, create_token, get_key, AllMediaVerify, FeedVerify, VerifyAllMedia,
};
use Collection::{add_to_collection, CollectionValues};
use Settings::{
    encode_base64_id, establish_connection, get_core_config, get_dal_op, get_object_config,
    get_session, make_sqid, random_nums,
};
use Task::{Create_Progress, Update_Progress};

#[tokio::main]
async fn main() {
    // Returns Core Settings as a tuple values are front_end_url, main_url, file_size in that order
    let core = get_core_config();

    let feed_cors_url = core.0.join("Feed").unwrap();

    let SessionKey = Arc::new(SessionCache { Key: Cache::new(1) });

    // Cors Settings
    let origins = [
        core.0.as_str().parse::<HeaderValue>().unwrap(),
        feed_cors_url.as_str().parse::<HeaderValue>().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::HEAD])
        .allow_origin(origins)
        .allow_headers([
            ORIGIN,
            ACCESS_CONTROL_REQUEST_HEADERS,
            CONTENT_TYPE,
            COOKIE,
            ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_ALLOW_HEADERS,
            ACCESS_CONTROL_ALLOW_CREDENTIALS,
        ])
        .allow_credentials(true)
        .expose_headers([
            CONTENT_TYPE,
            COOKIE,
            ACCESS_CONTROL_REQUEST_HEADERS,
            ACCESS_CONTROL_ALLOW_HEADERS,
        ]);

    // Main Application Router that Routes each Request to its matching function
    let app = Router::new()
        .route("/api/account", post(Account::create_account))
        .route("/api/account", get(Account::view_account))
        .route("/api/account", patch(Account::update_account))
        .route("/api/account/avatar", patch(Account::update_avatar))
        .route("/api/account/card", post(Account::account_card))
        .route("/api/login", post(Login::login))
        .route("/api/logout", post(Login::logout))
        .route("/api/follow", post(Follow::follow))
        .route("/api/follow/list", post(Follow::list))
        .route("/api/isfollowing", get(Follow::isfollowing))
        .route("/api/unfollow", post(Follow::unfollow))
        .route("/api/media", patch(Media::UpdateDetails))
        .route("/api/media", get(Media::ViewMedia))
        .route("/api/media/file/:folder/:file", get(Media::get_media_file))
        .route("/api/text", post(Text::Create_Text))
        .route("/api/text", patch(Text::UpdateText))
        .route("/api/video", post(Video::UploadVideo))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(core.2))
        .route("/api/audio", post(Audio::UploadAudio))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(core.2))
        .route("/api/image", post(Image::UploadImage))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(core.2))
        .route("/api/collection", post(Collection::Create_Collection))
        .route("/api/collection/add", post(Collection::Update_Collection))
        .route("/api/feed", get(Feed::feed))
        .route("/api/tasks", get(Task::list_tasks))
        .layer(cors)
        .layer(CookieManagerLayer::new())
        .with_state(SessionKey);

    println!("listening on {}", core.1);

    let listener = tokio::net::TcpListener::bind(core.1.as_str())
        .await
        .unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}

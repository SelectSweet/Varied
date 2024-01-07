use rand::{thread_rng, Rng};
use sea_orm::{
    DatabaseConnection, 
    Database, 
    ConnectOptions, 
    ActiveModelTrait, 
    prelude::DateTime, 
    EntityTrait, 
    ColumnTrait, 
    QueryFilter, 
    QuerySelect, 
    Set, 
    ActiveValue,
    ModelTrait,
    DbBackend, Statement
};

use sea_orm::{Condition, FromQueryResult};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqids::{Sqids, Options};
use std::os::unix::process;
use std::{fs::File, io::{Read, Write}, net::SocketAddr, fmt, collections::HashMap, process::{Command, Stdio}};
use url::Url;
use base64::{*, engine::general_purpose};
use chrono::{NaiveDateTime, Utc};


use axum::{
    async_trait,
    body::Bytes,
    extract::{multipart::*, ConnectInfo, Multipart, State, BodyStream, DefaultBodyLimit, Query},
    headers::{authorization::Bearer, Authorization, HeaderMap},
    headers::Cookie,
    TypedHeader,
    http::{header::*, Method, Request, StatusCode, request::Parts as RequestParts},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, patch, post, options},
    body::Body,
    Json, Router, debug_handler,
};


use axum_extra::extract::cookie::Cookie as CookieJ;
use axum_extra::extract::CookieJar;
use axum_extra::extract::Form;
use axum_extra::either::Either;


use tower_http::{
    cors::{Any, CorsLayer, AllowOrigin}, 
    limit::RequestBodyLimitLayer,
    ServiceBuilderExt,
    set_header::SetResponseHeaderLayer
};

use argon2::{password_hash::{SaltString, rand_core::OsRng}, Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use uuid::Uuid;

use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegProgress};


use migration::{Migrator, MigratorTrait};

use entity::{v_account, v_follow, v_media, v_session, v_task};



#[derive(Deserialize)]
pub struct Config {
    database: database,
    RClone: RClone,
    Core: Core
}

#[derive(Deserialize)]
pub struct RClone {
    name: String,
    endpoint: Url,
    process: String
}

#[derive(Deserialize)]
pub struct Core {
    file_size_limit: String,
    front_end_url: Url
}


#[derive(Deserialize)]
pub struct database {
    url: String,
}

mod Account;
mod Feed;
mod Follow;
mod Login;
mod Media;
mod Text;
mod Task;
mod Video;
//mod Image;
mod Audio;

use Task::{Create_Progress, Update_Progress};


pub async fn establish_connection() -> DatabaseConnection {

    // Reads config file
    let database_url = File::open("varied.toml");

    // Gets Database url from read config file then adds it to empty string
    let mut dstring = String::new();
    database_url.unwrap().read_to_string(&mut dstring).unwrap();

    // Converts Database url string to Config Struct then get the url from the struct
    let read_url: Config = toml::from_str(&dstring).unwrap();
    let url = read_url.database.url;

    // Connection Options
    let mut options = ConnectOptions::new(url);

    // Connection to Database
    let connection = Database::connect(options).await.unwrap();

    return connection;

}

const CUSTOM_ENGINE: engine::GeneralPurpose = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

pub fn encode_base64_id(id: String) -> String {
    let baseid = CUSTOM_ENGINE.encode(id);
    return baseid
}

pub fn make_sqid(nums: Vec<u64>) -> String {
    let sqid = Sqids::new(Some(Options::new(
        Some("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".to_string()), 
        Some(5), 
        None
    ))).unwrap();

    let id = sqid.encode(&nums).unwrap();

    return id;


}

pub async fn random_nums(range: u8) -> Vec<u64> {
    const CHARSET: &[u8] = b"0123456789";
    let mut rng = thread_rng();

    let rand_nums: Vec<u64> = (0..range).map(
        |_| {
            let rand_s = rng.gen_range(0..CHARSET.len());
            CHARSET[rand_s] as u64
        }
    ).collect();

    return rand_nums
}


pub async fn random_alpha(range: u8) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = thread_rng();

    let rand_string: String = (0..range).map(
        |_| {
            let rand_s = rng.gen_range(0..CHARSET.len());
            CHARSET[rand_s] as char
        }
    ).collect();

    return rand_string
}



pub async fn get_session(cookies: CookieJar) -> String {

    let connection = establish_connection().await;
    let session_id = cookies.get("id").unwrap().to_string().replace("id=", "");

    let session = v_session::Entity::find_by_id(session_id.clone())
    .into_json()
    .one(&connection).await.unwrap().unwrap()
    ;

    let Username = session["username"].to_string();    

    return Username;
}

pub fn get_rclone_config() -> HashMap<String, String> {
    // Reads config file
    let RClone = File::open("varied.toml");

    // Gets RClone Details from read config file then adds it to empty string
    let mut Rstring = String::new();
    RClone.unwrap().read_to_string(&mut Rstring).unwrap();

    // Converts Rstring to Config Struct then get name, endpoint and process from the struct
    let read_config: Config = toml::from_str(&Rstring).unwrap();
    let name = read_config.RClone.name;
    let endpoint = read_config.RClone.endpoint;
    let process: String = read_config.RClone.process;

    // Insert Key and Value of Name, Endpoint and Process into Config Hashmap
    let mut Config: HashMap<String, String> = HashMap::new();
    Config.insert("Name".to_string(), name);
    Config.insert("Endpoint".to_string(), endpoint.to_string());
    Config.insert("Process".to_string(), process);

    
    // Returns Config Hashmap
    return Config   
}



#[tokio::main]
async fn main() {
    //Migrator::up(&establish_connection().await, None).await.unwrap();

    // Listening URL
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    // Reads config file
    let CoreConfig = File::open("varied.toml");

    // Gets CoreConfig Details from read config file then adds it to empty string
    let mut Cstring = String::new();
    CoreConfig.unwrap().read_to_string(&mut Cstring).unwrap();

    // Converts Rstring to Config Struct then get the url from the struct
    let read_config: Config = toml::from_str(&Cstring).unwrap();
    let file_size = read_config.Core.file_size_limit.parse::<usize>().unwrap();
    let front_end_url = Url::parse(read_config.Core.front_end_url.as_str()).unwrap();


    let feed_cors_url = front_end_url.join("Feed").unwrap();

    // Core Settings
    let origins = [
        front_end_url.as_str().parse::<HeaderValue>().unwrap(),
        feed_cors_url.as_str().parse::<HeaderValue>().unwrap(),
    ];

    
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::HEAD])
        .allow_origin(origins)
        .allow_headers([ORIGIN, ACCESS_CONTROL_REQUEST_HEADERS, CONTENT_TYPE, COOKIE, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_CREDENTIALS])
        .allow_credentials(true)
        .expose_headers([CONTENT_TYPE, COOKIE, ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_ALLOW_HEADERS])
        ;
        


    // Main Application Router that Routes each Request to its matching function
    let app = Router::new()
        .route("/api/account", post(Account::create_account))
        .route("/api/account", get(Account::view_account))
        .route("/api/login", post(Login::login))
        .route("/api/logout", post(Login::logout))
        .route("/api/follow", post(Follow::follow))
        .route("/api/follow/list", post(Follow::list))
        .route("/api/isfollowing", get(Follow::isfollowing))
        .route("/api/unfollow", post(Follow::unfollow))
        .route("/api/media", patch(Media::UpdateDetails))
        .route("/api/media", get(Media::ViewMedia))
        .route("/api/text", post(Text::Create_Text))
        .route("/api/video", post(Video::UploadVideo)).layer(DefaultBodyLimit::disable()).layer(RequestBodyLimitLayer::new(file_size))
        .route("/api/audio", post(Audio::UploadAudio)).layer(DefaultBodyLimit::disable()).layer(RequestBodyLimitLayer::new(file_size))
        .route("/api/feed", get(Feed::feed))
        .route("/api/tasks", get(Task::list_tasks))
        .layer(cors)
        ;

    
    println!("listening on {}", addr);
    axum::Server::try_bind(&addr).unwrap().serve(app.into_make_service()).await.unwrap();
}

use super::*;

#[derive(Deserialize, Serialize)]
pub struct UpdateDetails {
    pub PublicId: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub chapters: Option<Value>,
}

#[derive(Deserialize, Serialize)]
pub struct ViewDetails {
    pub PublicId: String,
}


#[derive(Deserialize, Serialize, Debug)]
pub struct Media {
    pub publicid: String, 
    pub title: String, 
    pub mediatype: String, 
    pub uploaded_at: NaiveDateTime, 
    pub username: String, 
    pub description: Option<String>, 
    pub chapters: Option<Value>, 
    pub storagepathorurl: Option<Vec<String>>, 
    pub properties: Option<Value>, 
    pub state: String 
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum MediaType {
    Note,
    Text,
    Audio,
    Video,
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MediaType::Note => write!(f, "Note"),
            MediaType::Text => write!(f, "Text"),
            MediaType::Audio => write!(f, "Audio"),
            MediaType::Video => write!(f, "Video"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MediaState {
    Uploading,
    Streaming,
    Published,
}

impl fmt::Display for MediaState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MediaState::Uploading => write!(f, "Uploading"),
            MediaState::Streaming => write!(f, "Streaming"),
            MediaState::Published => write!(f, "Published"),
        }
    }
}

// axum::http::Response<Body>
#[debug_handler]
pub async fn get_media_file(media: APath<HashMap<String, String>>) -> axum::http::Response<Body> {
    let op = get_dal_op().await.unwrap();
    let MediaPathStr = media.0["folder"].to_string() + "/" + &media.0["file"];
    let read = op.0.read(MediaPathStr.as_str()).await.unwrap();
    let name: Vec<&str> = MediaPathStr.split("/").collect();
    let File_ext = Path::new(name[1]).extension().unwrap();
    let file = format!("filename={}", name[1]);
    let mut ext = String::new();


    if File_ext == "webm" {
       ext.push_str("video/webm");
    }

    if File_ext == "webp" {
        ext.push_str("image/webp");
    }

    if File_ext == "flac" {
        ext.push_str("audio/flac");
    }

    let headers = [
        (header::CONTENT_TYPE, ext),
        (
          header::CONTENT_DISPOSITION,
          file
        )
    ];
    return (headers, Body::from(read)).into_response()

    //return Json(fixedpath);
}


#[debug_handler]
pub async fn ViewMedia(
    cookies: CookieJar,
    //cookies: CookieJar,
    Query(data): Query<ViewDetails>,   
) -> Json<String> {
    let connection = establish_connection().await;
    if data.PublicId == "" {
        return Json("A Media ID is required".to_string());
    }

    let ViewMedia = v_media::Entity::find().columns([
        v_media::Column::Publicid,
        v_media::Column::Title,
        v_media::Column::Mediatype,
        v_media::Column::UploadedAt,
        v_media::Column::Username,
        v_media::Column::Description,
        v_media::Column::Chapters,
        v_media::Column::Storagepathorurl,
        v_media::Column::Properties,
        v_media::Column::State]
    ).filter(v_media::Column::Publicid.eq(data.PublicId)).into_json().one(&connection).await.unwrap().unwrap();

    let ViewMediaV = ViewMedia.as_object().unwrap();
    let mut MediaJ = json!(ViewMediaV);

    

    let mut MediaUrl: Vec<Value> = Vec::new();
    let mut PosterUrl: Vec<Value> = Vec::new();

    
    let MediaResult = MediaJ.as_object_mut().unwrap();

    MediaResult.remove("storagepathorurl");
    MediaResult.remove("poster_storagepathorurl");
    MediaResult.remove("id");

    let MediaType = &ViewMedia["mediatype"];

    if MediaType != "Note" && MediaType != "Text"  {
        let Media: Vec<Value> = ViewMedia["storagepathorurl"].as_array().unwrap().to_vec();
        let Poster: Vec<Value> = ViewMedia["poster_storagepathorurl"].as_array().unwrap().to_vec();
        for l in Media {
            MediaUrl.push(json!("http://localhost:8000".to_owned() + "/api/media/file/" + &l.to_owned().as_str().unwrap().replace("Cache/", "")));
        }
    
        MediaResult.insert("Urls".to_string(), json!(MediaUrl));
    
        for p in Poster {
            PosterUrl.push(json!("http://localhost:8000".to_owned() + "/api/media/file/" + &p.to_owned().as_str().unwrap().replace('"', "").replace("Cache/", "").replace("S3/", "")  ));
        }
    
        MediaResult.insert("Posters".to_string(), json!(PosterUrl));
    }  


    return Json(json!(MediaResult).to_string());
}

pub async fn UpdateDetails(
    cookies: CookieJar,
    Json(data): Json<UpdateDetails>,
) -> String {
    let Username = get_session(cookies).await;
    let connection = establish_connection().await;

    if data.PublicId == "" {
        return "A Media ID is required".to_string();
    }

    let details: Option<v_media::Model> = v_media::Entity::find()
    .filter(v_media::Column::Publicid.eq(data.PublicId))
    .one(&connection).await.unwrap();

    let mut details: v_media::ActiveModel = details.unwrap().into();

    if data.title != None {
        details.title = Set(data.title.unwrap());
    }
 
    if data.description != None {
         details.description = Set(data.description);
    }
 
    if data.chapters != None {
        details.chapters = Set(data.chapters);
    }

    details.update(&connection).await.unwrap();


    return "Media has been Updated".to_string();
}

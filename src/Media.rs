use std::collections::HashMap;

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


// id text NOT NULL,
//     publicid text NOT NULL,
//     title text NOT NULL,
//     mediatype text NOT NULL,
//     uploaded_at timestamp default (now() at time zone 'utc') NOT NULL,
//     username text NOT NULL, 
//     description text,
//     chapters json,
//     storagepathorurl text[],
//     properties json,
//     state text NOT NULL,


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





#[debug_handler]
pub async fn ViewMedia(
    cookies: CookieJar,
    //cookies: CookieJar,
    Query(data): Query<ViewDetails>,   
) -> Either<Json<String>, Json<String>> {
    let connection = establish_connection().await;
    if data.PublicId == "" {
        return Either::E1(Json("A Media ID is required".to_string()));
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
    ).filter(v_media::Column::Publicid.eq(data.PublicId)).into_json().one(&connection).await.unwrap();

    return Either::E2(Json(ViewMedia.unwrap().to_string()));
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

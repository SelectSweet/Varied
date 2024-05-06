use super::*;

fn word_count(Text: String) -> [i64; 3] {
    let mut lines = 0;
    let mut words = 0;
    let mut chars = 0;

    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_chars = 0;

    let mut body = Text;

    for line in body.lines() {
        let my_line = line;
        total_lines = total_lines + 1;
        total_words += my_line.split_whitespace().count();
        total_chars = total_chars + my_line.len() + 1;
        lines += total_lines;
        words += total_words;
        chars += total_chars;
    }

    let results = [total_lines as i64, total_words as i64, total_chars as i64];

    return results;
}


#[derive(TryFromMultipart, Deserialize)]
pub struct Text {
    pub title: String,
    pub body: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateText {
    pub PublicId: String,
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct DeleteText {
    pub PublicId: String,
}


pub async fn Create_Text(
    cookies: CookieJar,
    headers: HeaderMap,
    Json(data): Json<Text>
) -> Result<(CookieJar, Json<String>), StatusCode> {
    let connection = establish_connection().await;

    let Username = get_session(cookies.clone()).await.replace("'", "").replace("\"", "");

    let Title = data.title.to_string();
    let ID = Uuid::new_v4().to_string();
    let PublicId = make_sqid(random_nums(10).await);
    let mut text_body = data.body.to_string();
    let word_count_res = word_count(text_body);

    let lines_res = word_count_res[0];
    let word_res = word_count_res[1];
    let charcters_res = word_count_res[2];
    let mut text_type = String::new();

    if charcters_res < 501 {
        text_type.push_str("Note")
    } else {
        text_type.push_str("Article")
    }

    let properties = json!({
        "body": data.body.to_string(),
        "words": word_res,
        "lines": lines_res,
        "charcters": charcters_res,
        "Poster": false,
        "Album": false,
        "Avatar": false
    });

    // gets the current datetime
    let now = Utc::now();

    //let mut poster: Vec<String> = Vec::new(); 
    //poster.push(data.poster.unwrap()); // replace this with the poster function

    let insert_details = v_media::ActiveModel { 
        id: ActiveValue::Set(ID), 
        publicid: ActiveValue::Set(PublicId), 
        title: ActiveValue::Set(Title),
        mediatype: ActiveValue::Set(text_type), 
        uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())), 
        username: ActiveValue::Set(Username),
        description: ActiveValue::NotSet, 
        chapters: ActiveValue::NotSet, 
        storagepathorurl: ActiveValue::NotSet,
        poster_storagepathorurl: ActiveValue::NotSet, // Change this to ActiveValue::Set(Some(poster)) when poster is fixed
        properties: ActiveValue::Set(properties), 
        state: ActiveValue::Set(Media::MediaState::Published.to_string()), 
    };

    insert_details.insert(&connection).await.unwrap();

    return Ok((
        cookies,
        Json("Success".to_string())
    ));
}

pub async fn UpdateText(
    cookies: CookieJar,
    Json(data): Json<UpdateText>
) -> Json<String> {
    let connection = establish_connection().await;
    let id = data.PublicId;

    let mut Text = v_media::Entity::find()
       .filter(v_media::Column::Publicid.eq(id.to_owned()))
       .into_json()
       .one(&connection).await.unwrap().unwrap();

    let TextObject = Text.as_object_mut().unwrap();

    let mut Properties = TextObject["properties"].to_owned();   


    if Some(id.to_owned()).is_some() && Some(data.body.to_owned()).is_some() {
       Properties["body"] = json!(data.body.unwrap());
       let Updated = format!("{} Has Been updated", id.to_owned());
       return Json(Updated.to_string());
    }

    else {
        return Json("An ID and Text Body are Required".to_string())
    }
}

pub async fn DeleteText(
    cookies: CookieJar,
    Json(data): Json<DeleteText>
) {
    let connection = establish_connection().await;
    let id = data.PublicId;
    let mut Text = v_media::Entity::find()
       .filter(v_media::Column::Publicid.eq(id.to_owned()))
       .into_json()
       .one(&connection).await.unwrap().unwrap();
    
}


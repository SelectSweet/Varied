use super::*;


#[derive(Deserialize, Serialize, Debug, FromQueryResult)]
pub struct Feed {
    pub publicid: String,
    pub title: String,
    pub mediatype: String, // need to validate that this can only be values from the mediatype enum here
    pub uploaded_at: NaiveDateTime,
    pub username: String,
    pub description: Option<String>,
    pub chapters: Option<Value>,
    pub properties: Option<Value>
}

#[debug_handler]
pub async fn feed(
    cookies: CookieJar,
    headers: HeaderMap,
    //cookies: CookieJar,
) -> Result<(CookieJar, Json<Vec<Feed>>), StatusCode>{
    let connection = establish_connection().await;
    
    let Username = get_session(cookies.clone()).await.replace("\"", "");

    let feed_query = format!("SELECT DISTINCT on (publicid) * FROM v_media INNER JOIN v_follow ON v_media.username = v_follow.following WHERE v_follow.follower = '{}' AND v_media.properties ->> 'Album' = 'false' AND v_media.properties  ->> 'Avatar' = 'false' AND v_media.properties ->> 'Poster' = 'false';", Username);
   
    let build_feed: Vec<Feed> = v_media::Entity::find()
    //.columns([ v_media::Column::Publicid, v_media::Column::Title, v_media::Column::Mediatype, v_media::Column::UploadedAt, v_media::Column::Username, v_media::Column::Description, v_media::Column::Chapters, v_media::Column::Properties])
    //.filter(v_media::Column::Username.eq(Username.replace("\"", "")))    
    .from_raw_sql(
        Statement::from_string(
            DbBackend::Postgres, 
            feed_query        
        )        
    )
    .into_model::<Feed>()
    //.into_json()
    .all(&connection)
    .await.unwrap();

    return Ok((
        cookies,
        Json(build_feed)
    ));
    
}

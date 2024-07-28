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
    pub properties: Value,
}

#[debug_handler]
pub async fn feed(
    cookies: Cookies,
    headers: HeaderMap,
    //cookies: CookieJar,
) -> Result<Json<Vec<Feed>>, StatusCode> {
    let connection = establish_connection().await;

    let Username = get_session(cookies.to_owned()).await;

    let Key = get_key(Username.to_owned()).await;

    let ID = cookies.get("id").unwrap().to_string();

    let Token = Biscuit::from(ID, PublicKey::from_bytes(&Key).unwrap()).unwrap();

    let TokenVerify = FeedVerify(Token.to_owned(), Username.to_owned()).await;

    if Token.authorize(&TokenVerify).is_ok() {
        let feed_query = format!("SELECT DISTINCT on (publicid) v_media.publicid, v_media.title, v_media.mediatype, v_media.uploaded_at, v_media.username, v_media.description, v_media.properties FROM v_media INNER JOIN v_follow ON v_media.username = v_follow.following WHERE v_follow.follower = '{}' AND v_media.properties ->> 'Album' = 'false' AND v_media.properties  ->> 'Avatar' = 'false' AND v_media.properties ->> 'Poster' = 'false';", Username);

        let build_feed = v_media::Entity::find()
            //.columns([ v_media::Column::Publicid, v_media::Column::Title, v_media::Column::Mediatype, v_media::Column::UploadedAt, v_media::Column::Username, v_media::Column::Description, v_media::Column::Chapters, v_media::Column::Properties])
            //.filter(v_media::Column::Username.eq(Username.replace("\"", "")))
            .from_raw_sql(Statement::from_string(DbBackend::Postgres, feed_query))
            .into_model::<Feed>()
            //.into_json()
            .all(&connection)
            .await
            .unwrap();

        return Ok(Json(build_feed));
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
}

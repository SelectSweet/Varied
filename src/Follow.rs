use sea_orm::ActiveValue::NotSet;

use super::*;


#[derive(Deserialize, Debug)]
pub struct Follow {
    follow: String,
    properties: Option<Value>,
}


#[derive(Deserialize, Serialize, Debug, FromQueryResult)]
pub struct Following {
    pub follower: String,
    pub following: Option<String>,
    pub properties: Option<Vec<Value>>,
}

#[debug_handler]
pub async fn list(
    cookies: CookieJar,
    //cookies: CookieJar,
) -> Json<String> {
    let connection = establish_connection().await;
    let Username = get_session(cookies).await;

    let list = v_follow::Entity::find()
    .filter(v_follow::Column::Follower.eq(Username))
    .into_json()
    .one(&connection)
    .await.unwrap();

    return Json(list.unwrap().as_str().unwrap().to_string())
}

#[debug_handler]
pub async fn isfollowing(
    cookies: CookieJar,
    //cookies: CookieJar,
    headers: HeaderMap,
    data: Query<Follow>,
) -> Json<String> {
    let connection = establish_connection().await;
    let following = &data.follow.to_string();
    let properties = &data.properties;
    
    let Username = get_session(cookies).await;

    let list = v_follow::Entity::find()
    .filter(v_follow::Column::Follower.eq(Username))
    .into_json()
    .one(&connection)
    .await.unwrap().unwrap();

    let mut s = String::new();

    if list["Following"].get(&following).is_some() {
        s.push_str("true");        
    }

    else {
        s.push_str("false");
    }

    let res = Json(s);
    return res;
}


pub async fn follow(
    cookies: CookieJar,
    //cookies: CookieJar,
    Json(data): Json<Follow>,
) -> Result<(CookieJar, Json<String>), StatusCode> {
    let connection = establish_connection().await;
    let following = data.follow.to_string().replace("\"", "");
    let properties = data.properties;
    
    // gets the current datetime
    let now = Utc::now();
    
    let Username = get_session(cookies.clone()).await.replace("\"", "");

    let properties = json!({});

    let mut follow = v_follow::ActiveModel {
        follower: Set(Username),
        following: Set(Some(following)),
        properties: Set(Some(properties)),
        id: Set(make_sqid(random_nums(5).await)) // remove the need for this asap
    };

    let follow: v_follow::Model = follow.insert(&connection).await.unwrap();

    return Ok((
        cookies,
        Json("Success".to_string())
    ));
}

pub async fn unfollow(
    cookies: CookieJar,
    //cookies: CookieJar,
    Json(data): Json<Follow>,
) -> Json<String> {
    let connection = establish_connection().await;
    let following = data.follow.to_string();
    let properties = data.properties;
    
    let Username = get_session(cookies).await;

    let unfollow: v_follow::Model = v_follow::Entity::find().filter(
        Condition::all().add(v_follow::Column::Follower.eq(Username)).add(v_follow::Column::Following.eq(following))
    ).one(&connection).await.unwrap().unwrap();

    unfollow.delete(&connection).await.unwrap();

    
    return Json("Success".to_string());    
}

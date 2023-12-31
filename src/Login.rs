use sea_orm::{Condition, Statement};

use super::*;

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub Username: String,
    pub Password: String,
}

#[debug_handler]
pub async fn login(
    cookies: CookieJar,
    headers: HeaderMap,
    Form(data): Form<User>
) -> Result<(CookieJar, Json<String>), StatusCode>  {
    
    let connection = establish_connection().await;

    let CurrentUsername = data.Username.to_string();

    //let SelectAccount = v_account::Entity().filter(v_account::Column::Username.eq(CurrentUsername)).into_json().one(&connection).await.unwrap().unwrap();

    let SelectAccount = v_account::Entity::find().from_raw_sql(
        Statement::from_sql_and_values(
            DbBackend::Postgres, 
            "SELECT * FROM v_account WHERE username = $1",
            [ CurrentUsername.into() ] 
        )
    )
    .into_json()
    .one(&connection).await.unwrap().unwrap();


    let username = data.Username.to_string(); // SANITIZE THIS
    let password = data.Password.to_string(); // AND THIS

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();

    if username == SelectAccount["username"].as_str().unwrap() {
        if argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
        {
            let session_id = encode_base64_id(Uuid::new_v4().to_string());

            let session = v_session::ActiveModel { 
                session_id: ActiveValue::Set(session_id.to_owned()), 
                username: ActiveValue::Set(username.to_owned())
            };

            session.insert(&connection).await.unwrap();

            let c = cookies.add(CookieJ::new("id", session_id.to_owned()));

            return Ok((
                c,
                Json(session_id.to_owned())
            ));

        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        println!("Password Error");
        return Err(StatusCode::UNAUTHORIZED);
    }
}



pub async fn logout(
    cookies: CookieJar
) -> Json<String> {
    let connection = establish_connection().await;

    let id = cookies.get("id").unwrap().to_string().replace("id=", "");

    let drop_session: Option<v_session::Model> = v_session::Entity::find().filter(
        v_session::Column::SessionId.eq(id)
    ).one(&connection).await.unwrap();

    let drop_session: v_session::Model = drop_session.unwrap();

    drop_session.delete(&connection).await.unwrap();

    return Json("Success".to_string());
}

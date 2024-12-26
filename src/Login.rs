use Settings::Get_Tower_Key;

use super::*;

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub Username: String,
    pub Password: String,
}

// Result<(CookieJar, Json<String>), StatusCode>
#[debug_handler]
pub async fn login(
    cookies: Cookies,
    headers: HeaderMap,
    State(state): State<Arc<SessionCache>>,
    Json(data): Json<User>,
) -> Result<Json<String>, StatusCode> {
    let connection = establish_connection().await;

    let CurrentUsername = data.Username.to_string();

    //let SelectAccount = v_account::Entity().filter(v_account::Column::Username.eq(CurrentUsername)).into_json().one(&connection).await.unwrap().unwrap();

    let SelectAccount = v_account::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT * FROM v_account WHERE username = $1",
            [CurrentUsername.into()],
        ))
        .into_json()
        .one(&connection)
        .await
        .unwrap()
        .unwrap();

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
        if argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            //let session_id = encode_base64_id(Uuid::new_v4().to_string());

            let Key = Get_Tower_Key(username.to_owned()).await;

            //SessionKey.set(Key.to_owned()).unwrap();

            let TowerKey = Key::from(&Key);

            let Session = cookies.private(&TowerKey);

            let authority_id = create_token(username.to_owned()).await.to_string();

            let SessionCookie = Cookie::new("id", authority_id.to_owned());

            let c = Session.add(SessionCookie);

            let session_id = cookies.get("id").unwrap().to_string().replace("id=", "");

            let session = v_session::ActiveModel {
                session_id: ActiveValue::Set(session_id.to_owned()),
                username: ActiveValue::Set(username.to_owned()),
                authority: ActiveValue::Set(authority_id.to_string()),
            };

            session.insert(&connection).await.unwrap();

            state
                .Key
                .insert(session_id.to_owned(), Key.to_owned())
                .await;

            return Ok(Json("Login successful".to_string()));
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        println!("Password Error");
        return Err(StatusCode::UNAUTHORIZED);
    }
}

pub async fn logout(cookies: Cookies, State(state): State<Arc<SessionCache>>) -> Json<String> {
    let connection = establish_connection().await;

    let id = cookies.list()[0].to_string().replace("id=", "");
    //let id = cookie_id.to_string().replace("id=", "");
    let mut Username = String::new();

    let drop_session: Option<v_session::Model> = v_session::Entity::find()
        .filter(v_session::Column::SessionId.eq(id))
        .one(&connection)
        .await
        .unwrap();

    let drop_session: v_session::Model = drop_session.unwrap();

    Username.push_str(drop_session.username.as_str());

    drop_session.delete(&connection).await.unwrap();

    state.Key.invalidate(&Username).await;

    cookies.list()[0].make_removal();

    return Json("Success".to_string());
}

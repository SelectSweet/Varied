use super::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct Account {
    pub username: String,
    pub password: String,
    pub email: String,
    pub display_name: String,
    pub avatar: String,
    pub profile_metadata: Option<Value>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, FromQueryResult)]
pub struct ViewAccount {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub avatar: String,
    pub profile_metadata: Option<Value>,
    pub description: Option<String>,
}

// #[derive(Deserialize, Serialize)]
// pub struct AccountUpdate {
//     username: Option<String>,
//     password: Option<String>,
//     email: Option<String>,
//     display_name: Option<String>,
//     avatar: Option<Url>,
//     profile_metadata: Option<Value>,
//     description: Option<String>,
// }

pub async fn create_account(Json(data): Json<Account>) -> impl IntoResponse {

    // Database connection
    let connect = establish_connection().await;

    // Salt String for Password
    let salt = SaltString::generate(&mut OsRng);

    // Account ID
    let ID = Uuid::new_v4().to_string();

    // Main Argon2 Context Function
    let argon2 = Argon2::default();

    // inputted password
    let Password = data.password.to_string();

    // Hashes Password to PHC String
    let password_hash = argon2
        .hash_password(Password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // Parses PHC String into Password Hash
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();

    // display name input
    let display = data.display_name;

    

    // Checkes if display name input is none, 
    // if its none push username to display name to display_name string
    // else push display string to display_name
    let mut display_name = String::new();
    
    if display == "" {
        display_name.push_str(&data.username);
    } else {
        display_name.push_str(&display);
    }


    // verifys the password hash is ok then run the code with the if statement
    if argon2
        .verify_password(Password.as_bytes(), &parsed_hash)
        .is_ok()
    {
          
          // gets the current datetime
          let now = Utc::now();

          // Sets each value from the input into an ActiveModel including the ones that have been processed
          let mut CreateAccount = v_account::ActiveModel {
            id: ActiveValue::Set(ID),
            username: ActiveValue::Set(data.username.to_string()),
            password: ActiveValue::Set(parsed_hash.to_string()),
            email: ActiveValue::Set(data.email),
            created_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            display_name: ActiveValue::Set(display_name),
            avatar: ActiveValue::Set(data.avatar),
            profile_metadata: ActiveValue::Set(data.profile_metadata),
            description: ActiveValue::Set(data.description),
        };

        // insert the CreateAccount ActiveModel into the database
        CreateAccount.insert(&connect).await.unwrap();

        // Returns Account has been created 
        (StatusCode::CREATED, Json("Success"))
    } else {
        (StatusCode::default(), Json("Error passwords do not match"))
    }
}


pub async fn view_account( cookies: CookieJar, ) -> Result<(CookieJar, Json<ViewAccount>), StatusCode>  {

    // Database Connection
    let connection = establish_connection().await;

    // Get Username from Session ID in Cookie
    let Username = get_session(cookies.clone()).await;

    // Querys's the Account table, Select Username, Email, DisplayName, Avatar, ProfileMetadata and Description  

    let view = v_account::Entity::find()
    .filter(
        v_account::Column::Username.eq(Username.replace("\"", ""))
    )
    .columns([
        v_account::Column::Username,
        v_account::Column::Email,
        v_account::Column::DisplayName,
        v_account::Column::Avatar,
        v_account::Column::ProfileMetadata,
        v_account::Column::Description,
    ])
    .into_model::<ViewAccount>()
    //.into_json()
    .one(&connection)
    .await
    .unwrap()
    ;
    
    // Converts SeaORM/Serde JSON query result to Axum Json
    let AccountResult = Json(view.unwrap());
    
    // returns CookieJar, account details as json
    return Ok((
        cookies,
        AccountResult
    ));
    
}
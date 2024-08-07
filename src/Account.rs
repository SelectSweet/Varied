use Settings::Create_Tower_Key;

use self::Image::{ImageUpload, ProcessImages};

use super::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct Account {
    pub username: String,
    pub password: String,
    pub email: String,
    pub display_name: String,
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

#[derive(Deserialize, Serialize, Debug, FromQueryResult)]
pub struct CardAccount {
    pub username: String,
    pub display_name: String,
    pub avatar: String,
    pub profile_metadata: Option<Value>,
    pub description: Option<String>,
}

#[derive(TryFromMultipart)]
pub struct AvatarUpload {
    #[form_data(limit = "unlimited")]
    pub image: FieldData<NamedTempFile>,
}

#[derive(Deserialize, Serialize)]
pub struct AccountUpdate {
    password: Option<String>,
    email: Option<String>,
    display_name: Option<String>,
    profile_metadata: Option<Value>,
    description: Option<String>,
}

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
            avatar: ActiveValue::NotSet,
            profile_metadata: ActiveValue::Set(data.profile_metadata),
            description: ActiveValue::Set(data.description),
        };

        // Creates Biscuit KeyPair
        create_key(data.username.to_owned()).await;

        // Creates Tower Sessions Key
        Create_Tower_Key(data.username).await;

        // insert the CreateAccount ActiveModel into the database
        CreateAccount.insert(&connect).await.unwrap();

        // Returns Account has been created
        (StatusCode::CREATED, Json("Success"))
    } else {
        (StatusCode::default(), Json("Error passwords do not match"))
    }
}

#[debug_handler]
pub async fn view_account(cookies: Cookies) -> Result<Json<ViewAccount>, StatusCode> {
    // Database Connection
    let connection = establish_connection().await;

    // Get Username from Session ID in Cookie and push it to Username
    let Username = get_session(cookies.to_owned()).await;

    // Querys's the Account table, Select Username, Email, DisplayName, Avatar, ProfileMetadata and Description

    let view = v_account::Entity::find()
        .filter(v_account::Column::Username.eq(Username.replace("\"", "")))
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
        .unwrap();

    // Converts SeaORM/Serde JSON query result to Axum Json
    let AccountResult = Json(view.unwrap());

    // returns CookieJar, account details as json
    return Ok(AccountResult);
}

#[derive(Deserialize)]
pub struct AccountCard {
    username: String,
}

#[debug_handler]
pub async fn account_card(Json(username): Json<AccountCard>) -> Json<String> {
    // Database Connection
    let connection = establish_connection().await;

    let config = get_core_config();

    let card = v_account::Entity::find()
        .filter(v_account::Column::Username.eq(username.username))
        .columns([
            v_account::Column::Username,
            v_account::Column::DisplayName,
            v_account::Column::Avatar,
            v_account::Column::ProfileMetadata,
            v_account::Column::Description,
        ])
        .into_model::<CardAccount>()
        .one(&connection)
        .await
        .unwrap()
        .unwrap();
    // http://localhost:8000/api/media/file/yBV6fYOyoGsMvw30TtBs/yBV6fYOyoGsMvw30TtBs_1280.webm

    let avatar_url =
        "http://".to_owned() + &config.1 + "/api/media/file/" + &card.avatar.replace("\"", "");

    let view_card = json!({
        "Username": card.username.as_str(),
        "DisplayName": card.display_name.as_str(),
        "Avatar": Url::parse(avatar_url.as_str()).unwrap(),
        "ProfileMetadata": card.profile_metadata.unwrap().as_str(),
        "Description": card.description,
    });

    let CardResult = Json(view_card.to_string());

    return CardResult;
}

enum AccountUpdateValue {
    Json(Value),
    String(String),
}

#[debug_handler]
pub async fn update_avatar(
    cookies: Cookies,
    TypedMultipart(AvatarUpload { image }): TypedMultipart<AvatarUpload>,
) {
    // Database connection
    let connection = establish_connection().await;

    // Get Username from Session ID in Cookie
    let Username = get_session(cookies.clone())
        .await
        .replace("'", "")
        .replace("\"", "");

    let account: Option<v_account::Model> = v_account::Entity::find()
        .filter(v_account::Column::Username.eq(Username.to_owned()))
        .one(&connection)
        .await
        .unwrap();
    let mut account: v_account::ActiveModel = account.unwrap().into();

    let Image = TypedMultipart(ImageUpload {
        image: image,
        title: None,
        addtocollection: false,
        addtoalbum: false,
        Description: None,
        CollectionId: None,
    });
    let Avatar = ProcessImages(Image, Username, false, true, cookies.to_owned()).await;
    let AvatarUrlString = Avatar["Avatar"].to_string();

    account.avatar = Set(Some(AvatarUrlString.as_str().to_string()));

    let account: v_account::Model = account.update(&connection).await.unwrap();
}

pub async fn update_account(
    cookies: Cookies,
    Json(data): Json<AccountUpdate>,
) -> Result<Json<String>, StatusCode> {
    let mut Account: HashMap<String, AccountUpdateValue> = HashMap::new();

    // Database connection
    let connection = establish_connection().await;

    // Get Username from Session ID in Cookie
    let Username = get_session(cookies.clone())
        .await
        .replace("'", "")
        .replace("\"", "");

    let account: Option<v_account::Model> = v_account::Entity::find()
        .filter(v_account::Column::Username.eq(Username))
        .one(&connection)
        .await
        .unwrap();
    let mut account: v_account::ActiveModel = account.unwrap().into();

    if data.password.is_some() {
        let password = account.password.to_owned().unwrap();
        let Password = data.password.unwrap();

        // Main Argon2 Context Function
        let argon2 = Argon2::default();
        // Salt String for Password
        let salt = SaltString::generate(&mut OsRng);
        // Hashes Password to PHC String
        let password_hash = argon2
            .hash_password(Password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        // Parses PHC String into Password Hash
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();

        if parsed_hash.to_string() == password {
            return Ok(Json("New Password Equals Current Password".to_string()));
        }
        // verifys the password hash is ok then run the code with the else if statement
        else if argon2
            .verify_password(Password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            let UpdatedPassword = parsed_hash.to_string();
            account.password = Set(UpdatedPassword);
        }
    }

    if data.email.is_some() {
        let email = account.email.unwrap();
        let Email = data.email.unwrap();

        if Email == email {
            return Ok(Json("New Email Equals Current Email".to_string()));
        } else {
            account.email = Set(Email);
        }
    }

    if data.display_name.is_some() {
        let display_name = account.display_name.unwrap();
        let Display_name = data.display_name.unwrap();

        if Display_name == display_name {
            return Ok(Json(
                "New Display Name Equals Current Display Name".to_string(),
            ));
        } else {
            account.display_name = Set(Display_name);
        }
    } else {
        Account.insert(
            "Display_name".to_string(),
            AccountUpdateValue::String("".to_string()),
        );
    }

    if data.description.is_some() {
        let description = account.description.unwrap().unwrap();
        let Description = data.description.unwrap();

        if Description == description {
            return Ok(Json(
                "New Description Equals Current Description".to_string(),
            ));
        } else {
            account.description = Set(Some(Description));
        }
    }

    let account: v_account::Model = account.update(&connection).await.unwrap();

    let AccountResult = Json("Success".to_string());

    return Ok(AccountResult);
}

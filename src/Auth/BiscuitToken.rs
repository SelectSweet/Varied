use super::*;

#[derive(Deserialize, Serialize, Debug, FromQueryResult)]
pub struct BiscuitKey {
    Key: Vec<u8>,
    Username: String,
}

pub async fn create_key(username: String) {
    let connection = establish_connection().await;

    let Key = KeyPair::new();
    let PrKey = Key.private();
    let PuKey = Key.public();
    let PrivateBytesString = PrKey.to_bytes().to_owned().to_vec();

    let KeyInsert = v_biscuitkey::ActiveModel {
        key: ActiveValue::Set(PrivateBytesString),
        username: ActiveValue::Set(username),
    };

    Insert::one(KeyInsert).exec(&connection).await.unwrap();
}

pub async fn get_key(username: String) -> Vec<u8> {
    let connection = establish_connection().await;

    let Key = v_biscuitkey::Entity::find()
        .filter(v_biscuitkey::Column::Username.eq(username))
        .columns([v_biscuitkey::Column::Username, v_biscuitkey::Column::Key])
        //.into_model::<BiscuitKey>()
        .one(&connection)
        .await
        .unwrap()
        .unwrap();

    return Key.key;
}

pub async fn create_token(username: String) -> Biscuit {
    let operation = "Write";
    let key = get_key(username.to_owned()).await;
    let mut authority = biscuit!(
        r#"
    user({username});

    right({username}, "AllMediaUpload", "Write");
    right({username}, "Feed", "Read");
    right({username}, "Follow", "Read");
    right({username}, "Follow", "Write");

    "#
    );

    let Pair = KeyPair::from(&PrivateKey::from_bytes(&key).unwrap());
    return authority.build(&Pair).unwrap();
}

pub async fn AllMediaVerify(
    bis: Biscuit,
    username: String,
    MediaUsername: String,
    MediaID: String,
) -> Authorizer {
    let key = get_key(username).await;
    let RootKey = KeyPair::from(&PrivateKey::from_bytes(&key).unwrap());
    let biscuit = Biscuit::from_base64(bis.to_base64().unwrap(), RootKey.public()).unwrap();

    let auth = authorizer!(
        r#"
          resource({MediaID});
          operation("Write");

          allow if right("AllMediaUpload", "Write");
      "#
    );

    return auth;
}

pub async fn VerifyAllMedia(
    Username: String,
    cookies: Cookies,
    MediaUsername: String,
    MediaID: String,
) -> (Biscuit, Authorizer) {
    let Key = get_key(Username.to_owned()).await;

    let ID = cookies.get("id").unwrap().to_string();

    let Token = Biscuit::from(ID, PublicKey::from_bytes(&Key).unwrap()).unwrap();

    let TokenVerify = AllMediaVerify(Token.to_owned(), Username, MediaUsername, MediaID).await;

    return (Token, TokenVerify);
}

pub async fn FeedVerify(bis: Biscuit, username: String) -> Authorizer {
    let key = get_key(username.to_owned()).await;
    let RootKey = KeyPair::from(&PrivateKey::from_bytes(&key).unwrap());
    let biscuit = Biscuit::from_base64(bis.to_base64().unwrap(), RootKey.public()).unwrap();

    let auth = authorizer!(
        r#"
          resource("Feed");
          operation("Read");

          allow if user({username});
          allow if right("Feed", "Read");
      "#
    );

    return auth;
}

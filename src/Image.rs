use super::*;

#[derive(TryFromMultipart)]
pub struct ImageUpload {
    #[form_data(limit = "unlimited")]
    pub image: FieldData<NamedTempFile>,

    pub title: Option<String>,
    pub addtocollection: bool,
    pub addtoalbum: bool,
    pub Description: Option<String>,
    pub CollectionId: Option<String>,
}

pub async fn PushImage(Path: String, PublicID: String, images: String, op: Operator) {
    let image = fs::read(Path).unwrap(); 
    let to = PublicID.as_str().to_owned() + "/" + &images;
    op.write(&to, image).await.unwrap();
}

pub async fn ProcessImages(
    TypedMultipart(ImageUpload {
        image,
        addtocollection,
        addtoalbum,
        Description,
        CollectionId,
        title,
    }): TypedMultipart<ImageUpload>,
    username: String,
    poster: bool,
    avatar: bool,
    cookies: CookieJar,
) -> Value {
    let name = image.metadata.file_name.unwrap();
    let filetype = image.metadata.content_type.unwrap();
    let path = Path::new("/tmp").join(name.to_owned());
    let PublicID = make_sqid(random_nums(12).await);
    let mut details: HashMap<String, CollectionValues> = HashMap::new();
    let Username = get_session(cookies.clone())
        .await
        .replace("'", "")
        .replace("\"", "");

    details.insert("Ids".to_owned(), Collection::CollectionValues::String(PublicID.to_owned()));
    details.insert("Type".to_owned(), Collection::CollectionValues::String("Image".to_string()));

    let connection = establish_connection().await;

    if CollectionId.is_some() {
        details.insert("Collection_Id".to_owned(), Collection::CollectionValues::String(CollectionId.unwrap()));
    }

    if addtoalbum == true {
        details.insert("AddToAlbum".to_owned(), Collection::CollectionValues::String(true.to_string()));
    }

    let mut description = String::new();

    if Description.is_some() {
        description.insert_str(0, &Description.unwrap());
    }

    image.contents.persist(path.to_owned()).unwrap();

    let object = get_object_config();

    let image_path = path.as_path().to_str().unwrap().to_string();

    let mut Paths: Vec<String> = Vec::new();

    let Process = object["Process"].to_owned();

    let ImageBucket = object["Name"].to_owned();

    let ImageStorageUrl = object["Endpoint"].to_owned() + &ImageBucket.to_owned() + "/";

    let ID = Uuid::new_v4().to_string();

    let op = get_dal_op().await.unwrap();

    std::fs::create_dir_all(Process.to_owned() + "/" + &ImageBucket + "/" + &PublicID.to_owned()).unwrap();

    // gets the current datetime
    let now = Utc::now();


    let mut Paths: Vec<String> = Vec::new();
    let mut images = Vec::new();

    images.push(PublicID.as_str().to_owned() + "_High.webp");

    Paths.push(Process.to_owned() + "/" + &ImageBucket + "/" + PublicID.to_owned().as_str() + "/" + images[0].as_str());   

    // Paths.push(
    //     process.to_owned() + "/" + &ImageBucket + "/" + PublicID.to_owned().as_str() + "/" + images[0].as_str(),
    // );
    // Paths.push(
    //     process.to_owned() + "/" + &ImageBucket + "/" + PublicID.to_owned().as_str() + "/" + images[0].as_str(),
    // );

    let mut UploadPath: Vec<String> = Vec::new();

    UploadPath.push(PublicID.as_str().to_owned()  + "/" + &images[0]);
    // UploadPath.push(PublicID.as_str().to_owned()  + "/" + &images[1]);
    // UploadPath.push(PublicID.as_str().to_owned()  + "/" + &images[2]);

    let oper = op.0;
    
    if addtoalbum == true {        

        FfmpegCommand::new()
            .input(image_path)
            .hide_banner()
            .arg("-y")
            .arg(Paths[0].as_str())
            .spawn()
            .unwrap()
            .iter()
            .expect("Image Not comverted");


        let properties = json!({
            "Poster": false,
            "Album": true,
            "Avatar": false
        });

        let image = v_media::ActiveModel {
            id: ActiveValue::Set(ID),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(name.to_owned()),
            mediatype: ActiveValue::Set("Image".to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(username),
            description: ActiveValue::Set(Some(description)),
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::Set(Some(UploadPath.to_owned())),
            poster_storagepathorurl: ActiveValue::NotSet,
            properties: ActiveValue::Set(properties),
            state: ActiveValue::Set(Media::MediaState::Published.to_string()),
        };

        let image: v_media::Model = image.insert(&connection).await.unwrap();

        let collection = add_to_collection(details, cookies).await;

        PushImage(Paths[0].to_owned(), PublicID.to_owned(), images[0].to_owned(), oper).await;                
    
        std::fs::remove_dir_all(Process.to_owned() + "/" + &ImageBucket + "/" + &PublicID.to_owned()).unwrap();

        let result = json!({
            "Result": "Success",
            "Publicid": PublicID,
            "Collection_Publicid": collection["Publicid"]
        });

        return result;
    } else if addtocollection == true && addtoalbum == false {        

        FfmpegCommand::new()
            .input(image_path)
            .hide_banner()
            .arg("-y")
            .arg(Paths[0].as_str())
            .spawn()
            .unwrap()
            .iter()
            .expect("Image Not comverted");


        let properties = json!({
            "Poster": false,
            "Avatar": false,
            "Album": true     
        });

        let ImageStorageURL = ImageStorageUrl + "/" + &images[0];

        let image = v_media::ActiveModel {
            id: ActiveValue::Set(ID),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(name.to_owned()),
            mediatype: ActiveValue::Set("Image".to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(username),
            description: ActiveValue::Set(Some(description)),
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::Set(Some(vec![images[0].to_owned()])),
            poster_storagepathorurl: ActiveValue::NotSet,
            properties: ActiveValue::Set(properties),
            state: ActiveValue::Set(Media::MediaState::Published.to_string()),
        };

        let image: v_media::Model = image.insert(&connection).await.unwrap();

        let collection = add_to_collection(details, cookies).await;

        PushImage(Paths[0].to_owned(), PublicID.to_owned(), images[0].to_owned(), oper).await;                
    
        std::fs::remove_dir_all(Process.to_owned() + "/" + &ImageBucket + "/" + &PublicID.to_owned()).unwrap();

        let result = json!({
            "Result": "Success",
            "Publicid": PublicID,
            "Collection_Publicid": collection["Publicid"]
        });

        return result;
    } else if addtocollection == true && addtoalbum == false {        

        FfmpegCommand::new()
            .input(image_path)
            .hide_banner()
            .arg("-y")
            .arg(Paths[0].as_str())
            .spawn()
            .unwrap()
            .iter()
            .expect("Image Not comverted");
        
        let properties = json!({
            "Poster": false,
            "Album": false,
            "Avatar": false
        });

        let image = v_media::ActiveModel {
            id: ActiveValue::Set(ID),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(name.to_owned()),
            mediatype: ActiveValue::Set("Image".to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(username),
            description: ActiveValue::Set(Some(description)),
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::Set(Some(UploadPath.to_owned())),
            poster_storagepathorurl: ActiveValue::NotSet,
            properties: ActiveValue::Set(properties),
            state: ActiveValue::Set(Media::MediaState::Published.to_string()),
        };

        let image: v_media::Model = image.insert(&connection).await.unwrap();

        let collection = add_to_collection(details, cookies).await;

        PushImage(Paths[0].to_owned(), PublicID.to_owned(), images[0].to_owned(), oper).await; 
         

        std::fs::remove_dir_all(Process.to_owned() + "/" + &ImageBucket + "/" + &PublicID.to_owned()).unwrap();

        let result = json!({
            "Result": "Success",
            "Publicid": PublicID,
            "Collection_Publicid": collection["Publicid"]
        });

        return result;
    } else if poster == true {

        FfmpegCommand::new()
            .input(image_path)
            .hide_banner()
            .arg("-y")
            .arg(Paths[0].as_str())
            .spawn()
            .unwrap()
            .wait()
            .expect("Image Not converted");

        let properties = json!({
            "Poster": true,
            "Album": false,
            "Avatar": false
        });
        let image = v_media::ActiveModel {
            id: ActiveValue::Set(ID),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(name.to_owned()),
            mediatype: ActiveValue::Set("Image".to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(username),
            description: ActiveValue::Set(Some(description)),
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::Set(Some(UploadPath.to_owned())),
            poster_storagepathorurl: ActiveValue::NotSet,
            properties: ActiveValue::Set(properties),
            state: ActiveValue::Set(Media::MediaState::Published.to_string()),
        };

        let image: v_media::Model = image.insert(&connection).await.unwrap();

        let collection = add_to_collection(details, cookies).await;

        PushImage(Paths[0].to_owned(), PublicID.to_owned(), images[0].to_owned(), oper).await; 
         

        std::fs::remove_dir_all(Process.to_owned() + "/" + &ImageBucket + "/" + &PublicID.to_owned()).unwrap();

        let result = json!({
            "Result": "Success",
            "Publicid": PublicID,
            "Collection_Publicid": collection["Publicid"],
            "Poster": UploadPath
        });

        return result;
    }

    else if avatar == true {

        FfmpegCommand::new()
            .input(image_path)
            .hide_banner()
            .arg("-y")
            .arg(Paths[0].as_str())
            .spawn()
            .unwrap()
            .wait()
            .expect("Image Not converted");

        let properties = json!({
            "Poster": false,
            "Album": false,
            "Avatar": true
        });

        let image = v_media::ActiveModel {
            id: ActiveValue::Set(ID),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(name.to_owned()),
            mediatype: ActiveValue::Set("Image".to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(username),
            description: ActiveValue::NotSet,
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::Set(Some(UploadPath.to_owned())),
            poster_storagepathorurl: ActiveValue::NotSet,
            properties: ActiveValue::Set(properties),
            state: ActiveValue::Set(Media::MediaState::Published.to_string()),
        };

        let image: v_media::Model = image.insert(&connection).await.unwrap();

        let PosterUrl = PublicID.as_str().to_owned() + "/" + &images[0];

        //let PosterUrl = object["Endpoint"].to_string() + "/" + &to;

        PushImage(Paths[0].to_owned(), PublicID.to_owned(), images[0].to_owned(), oper).await;           
    
        std::fs::remove_dir_all(Process.to_owned() + "/" + &ImageBucket + "/" + &PublicID.to_owned()).unwrap();

        let result = json!({
            "Result": "Success",
            "Publicid": PublicID,
            "Avatar": PosterUrl
        });

        return result;
    }

    if addtocollection == true && poster == true {
        let result = json!({
            "Result": "Cant add poster to collection"
        });

        return result;
    }

    if addtocollection == false && poster == false {

        FfmpegCommand::new()
            .input(image_path)
            .hide_banner()
            .arg("-y")
            .arg(Paths[0].as_str())
            .spawn()
            .unwrap()
            .wait()
            .expect("Image Not converted");

        let properties = json!({
            "Poster": false,
            "Album": false,
            "Avatar": false
        });

        let image = v_media::ActiveModel {
            id: ActiveValue::Set(ID),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(name.to_owned()),
            mediatype: ActiveValue::Set("Image".to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(username),
            description: ActiveValue::Set(Some(description)),
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::Set(Some(UploadPath.to_owned())),
            poster_storagepathorurl: ActiveValue::NotSet,
            properties: ActiveValue::Set(properties),
            state: ActiveValue::Set(Media::MediaState::Published.to_string()),
        };

        let image: v_media::Model = image.insert(&connection).await.unwrap();

        PushImage(Paths[0].to_owned(), PublicID.to_owned(), images[0].to_owned(), oper).await;  

        std::fs::remove_dir_all(Process.to_owned() + "/" + &ImageBucket + "/" + &PublicID.to_owned()).unwrap();

        let result = json!({
            "Result": "Success",
            "Publicid": PublicID
        });

        return result;
    } else {
        let result = json!({
            "Result": "Failure",
            "Publicid": PublicID
        });

        return result;
    }
}

#[debug_handler]
pub async fn UploadImage(
    cookies: CookieJar,
    Query(details): Query<HashMap<String, String>>,
    TypedMultipart(ImageUpload {
        image,
        addtoalbum,
        Description,
        addtocollection,
        CollectionId,
        title,
    }): TypedMultipart<ImageUpload>,
) -> Json<String> {
    let Username = get_session(cookies.clone())
        .await
        .replace("'", "")
        .replace("\"", "");

    let image = ProcessImages(
        TypedMultipart(ImageUpload {
            image,
            addtoalbum,
            addtocollection,
            Description,
            CollectionId,
            title,
        }),
        Username,
        false,
        false,
        cookies,
    )
    .await;


    return Json(image.to_string());
}

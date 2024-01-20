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

pub async fn ProcessImages(TypedMultipart(
    ImageUpload { image, 
        addtocollection, 
        addtoalbum, 
        Description, 
        CollectionId,
        title, }): TypedMultipart<ImageUpload>,
    username: String,
    poster: bool,
) -> Value {
    let name = image.metadata.file_name.unwrap();
    let filetype = image.metadata.content_type.unwrap();
    let path = Path::new("/tmp").join(name.to_owned());
    let PublicID = make_sqid(random_nums(12).await);
    let mut details: HashMap<String, String> = HashMap::new();
    
    details.insert("Ids".to_owned(), PublicID.to_owned());
    details.insert("Type".to_owned(), "Image".to_string());
    
    let connection = establish_connection().await;


    if CollectionId.is_some() {
        details.insert("Collection_Id".to_owned(), CollectionId.unwrap());
    }

    if addtoalbum == true {
        details.insert("AddToAlbum".to_owned(), true.to_string());
    }

    let mut description = String::new();

    if Description.is_some() {
        description.insert_str(0, &Description.unwrap());
    }


    image.contents.persist(path.to_owned()).unwrap();

    let RCloneConfig = get_rclone_config();

    let image_path = path.as_path().to_str().unwrap().to_string();

    let mut Paths: Vec<String> = Vec::new();

    let process = RCloneConfig["Process"].to_owned();   
    
    let ImageBucket = RCloneConfig["Name"].to_owned();

    let ID = Uuid::new_v4().to_string();

    std::fs::create_dir_all(process.to_owned() + "/" + &ImageBucket + "/" + &PublicID).unwrap();

    // gets the current datetime
    let now = Utc::now();

    if addtocollection == true {

        Paths.push(
            process.to_owned() + "/" + &ImageBucket + "/" + &PublicID + "/" + &PublicID + ".webp",
        );
        // Paths.push(
        //     process.to_owned() + "/" + &ImageBucket + "/" + &PublicID + "/" + &PublicID + "-High.webp",
        // );
        // Paths.push(
        //     process.to_owned() + "/" + &ImageBucket + "/" + &PublicID + "/" + &PublicID + "-High.webp",
        // );
    
        FfmpegCommand::new()
                .input(image_path)
                .hide_banner()
                .arg("-y")
                .arg(
                    Paths[0].as_str(),
                    
                )
                .spawn()
                .unwrap()
                .iter()
                .expect("Image Not comverted");

        let properties = json!({
            "Poster": false
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
            storagepathorurl: ActiveValue::NotSet, 
            poster_storagepathorurl: ActiveValue::NotSet, 
            properties: ActiveValue::Set(Some(properties)),
            state: ActiveValue::Set(Media::MediaState::Published.to_string()) 
        };
    
        let image: v_media::Model = image.insert(&connection).await.unwrap();

        let collection = add_to_collection(details).await;

    
        let result = json!({
            "Result": "Success",
            "Publicid": PublicID,
            "Collection_Publicid": collection["Publicid"]
        });
    
        return result;
    }

    else if poster == true {

        Paths.push(
            process.to_owned() + "/" + &ImageBucket + "/" + &PublicID + "/" + &PublicID + ".webp",
        );
        // Paths.push(
        //     process.to_owned() + "/" + &ImageBucket + "/" + &PublicID + "/" + &PublicID + "-High.webp",
        // );
        // Paths.push(
        //     process.to_owned() + "/" + &ImageBucket + "/" + &PublicID + "/" + &PublicID + "-High.webp",
        // );
    
        FfmpegCommand::new()
                .input(image_path)
                .hide_banner()
                .arg("-y")
                .arg(
                    Paths[0].as_str(),
                    
                )
                .spawn()
                .unwrap()
                .iter()
                .expect("Image Not comverted");
            let properties = json!({
                "Poster": true
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
            storagepathorurl: ActiveValue::NotSet, 
            poster_storagepathorurl: ActiveValue::NotSet, 
            properties: ActiveValue::Set(Some(properties)),
            state: ActiveValue::Set(Media::MediaState::Published.to_string()) 
        };
    
        let image: v_media::Model = image.insert(&connection).await.unwrap();

        let PosterUrl = RCloneConfig["Endpoint"].to_string() + "/" + &PublicID.to_owned();
    
        let result = json!({
            "Result": "Success",
            "Publicid": PublicID,
            "Poster": PosterUrl
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

        Paths.push(
            process.to_owned() + "/" + &ImageBucket + "/" + &PublicID + "/" + &PublicID + ".webp",
        );
        // Paths.push(
        //     process.to_owned() + "/" + &ImageBucket + "/" + &PublicID + "/" + &PublicID + "-High.webp",
        // );
        // Paths.push(
        //     process.to_owned() + "/" + &ImageBucket + "/" + &PublicID + "/" + &PublicID + "-High.webp",
        // );
    
        FfmpegCommand::new()
                .input(image_path)
                .hide_banner()
                .arg("-y")
                .arg(
                    Paths[0].as_str(),
                    
                )
                .spawn()
                .unwrap()
                .iter()
                .expect("Image Not comverted");

                let properties = json!({
                    "Poster": false
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
            storagepathorurl: ActiveValue::NotSet, 
            poster_storagepathorurl: ActiveValue::NotSet, 
            properties: ActiveValue::NotSet,
            state: ActiveValue::Set(Media::MediaState::Published.to_string()) 
        };
        
    
        let image: v_media::Model = image.insert(&connection).await.unwrap();

        let result = json!({
            "Result": "Success",
            "Publicid": PublicID
        });

        return result;
    }

    else {
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
        title
    } ): TypedMultipart<ImageUpload>
) -> Json<String> {

    let Username = get_session(cookies.clone()).await.replace("'", "").replace("\"", "");

    


    let image = ProcessImages(TypedMultipart(ImageUpload {
        image, 
        addtoalbum, 
        addtocollection, 
        Description, 
        CollectionId,
        title 
    } ), Username, false).await;
    return Json(image.to_string());
    
}
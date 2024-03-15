use super::*;

pub fn VideoFrames(file: &str) -> String {
    let command = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-count_packets",
            "-show_entries",
            "stream=nb_read_packets",
            "-of",
            "csv=p=0",
        ])
        .arg(file)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let out = command.wait_with_output().unwrap().stdout;
    let binding = std::str::from_utf8(&out.to_owned()).unwrap().to_string();
    let OutString = binding.strip_suffix("\n").unwrap();
    return OutString.to_string();
}

#[derive(TryFromMultipart)]
pub struct VideoUpload {
    #[form_data(limit = "unlimited")]
    pub video: FieldData<NamedTempFile>,

    #[form_data(limit = "unlimited")]
    pub poster: Option<FieldData<NamedTempFile>>,

    pub addtocollection: bool,
    pub addtoalbum: bool,
    pub title: Option<String>,
    pub Description: Option<String>,
    pub CollectionId: Option<String>,
}

#[debug_handler]
pub async fn UploadVideo(
    cookies: CookieJar,
    TypedMultipart(VideoUpload {
        video,
        poster,
        addtocollection,
        addtoalbum,
        title,
        Description,
        CollectionId,
    }): TypedMultipart<VideoUpload>, //mut multipart: Multipart,
) -> Result<(CookieJar, Json<String>), StatusCode> {
    let connection = establish_connection().await;

    let ID = Uuid::new_v4().to_string();
    let PublicId = make_sqid(random_nums(10).await);
    let mut Title = String::new();
    let object = get_object_config();

    let name = video.metadata.file_name.unwrap();
    let filetype = video.metadata.content_type.unwrap();
    let path = Path::new("/tmp").join(name.to_owned());
    let mut details: HashMap<String, CollectionValues> = HashMap::new();

    details.insert(
        "Ids".to_owned(),
        Collection::CollectionValues::String(PublicId.to_owned()),
    );
    details.insert(
        "Type".to_owned(),
        Collection::CollectionValues::String("Video".to_string()),
    );

    if CollectionId.is_some() {
        details.insert(
            "Collection_Id".to_owned(),
            Collection::CollectionValues::String(CollectionId.to_owned().unwrap()),
        );
    }

    if addtoalbum == true {
        details.insert(
            "AddToAlbum".to_owned(),
            Collection::CollectionValues::String(true.to_string()),
        );
    }

    let op = get_dal_op().await.unwrap();

    video.contents.persist(path.to_owned()).unwrap();

    let process = object["Process"].to_owned();

    let VideoBucket = object["Name"].to_owned();

    let Username = get_session(cookies.clone())
        .await
        .replace("'", "")
        .replace("\"", "");

    let mut Paths: Vec<String> = Vec::new();
    let mut Videos: Vec<String> = Vec::new();

    Videos.push(PublicId.as_str().to_owned() + "_320.webm");
    Videos.push(PublicId.as_str().to_owned() + "_720.webm");
    Videos.push(PublicId.as_str().to_owned() + "_1280.webm");

    Paths
        .push(process.to_owned() + "/" + &VideoBucket + "/" + PublicId.as_str() + "/" + &Videos[0]);
    Paths
        .push(process.to_owned() + "/" + &VideoBucket + "/" + PublicId.as_str() + "/" + &Videos[1]);
    Paths
        .push(process.to_owned() + "/" + &VideoBucket + "/" + PublicId.as_str() + "/" + &Videos[2]);

    std::fs::create_dir_all(process.to_owned() + "/" + &VideoBucket + "/" + PublicId.as_str())
        .unwrap();

    let mut UploadPath: Vec<String> = Vec::new();

    UploadPath.push(PublicId.as_str().to_owned()  + "/" + &Videos[0]);
    UploadPath.push(PublicId.as_str().to_owned()  + "/" + &Videos[1]);
    UploadPath.push(PublicId.as_str().to_owned()  + "/" + &Videos[2]);


    if title.is_none() {
        Title.push_str(name.as_str())
    }

    let video_name = path.as_path().to_str().unwrap().to_string();

    // gets the current datetime
    let now = Utc::now();

    let mut PosterVec = Vec::new();

    if poster.is_some() {
        let Poster = ProcessImages(
            TypedMultipart(Image::ImageUpload {
                image: poster.unwrap(),
                addtocollection: addtocollection,
                addtoalbum: false,
                Description: None,
                CollectionId: None,
                title: title.to_owned(),
            }),
            Username.to_owned(),
            true,
            false,
            cookies.to_owned(),
        )
        .await;

        let PosterUrls = Poster["Poster"].as_array().unwrap();

        for u in PosterUrls {
            PosterVec.push(u.as_str().unwrap().to_string())
        }
    }

    if addtocollection == true && CollectionId.is_some() {

        let properties = json!({
           "Poster": false,
           "Album": false,
           "Avatar": false
        });

        let insert_details = v_media::ActiveModel {
            id: ActiveValue::Set(ID.to_owned()),
            publicid: ActiveValue::Set(PublicId.to_owned()),
            title: ActiveValue::Set(Title.to_owned()),
            mediatype: ActiveValue::Set(Media::MediaType::Video.to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(Username.to_owned()),
            description: ActiveValue::NotSet,
            chapters: ActiveValue::NotSet,
            poster_storagepathorurl: ActiveValue::Set(Some(PosterVec)),
            storagepathorurl: ActiveValue::Set(Some(UploadPath.to_owned())),
            properties: ActiveValue::Set(Some(properties)),
            state: ActiveValue::Set(Media::MediaState::Uploading.to_string()),
        };

        insert_details.insert(&connection).await.unwrap();

        let collection = add_to_collection(details, cookies.to_owned()).await;

        let result = json!({
            "Result": "Success",
            "Publicid": PublicId.to_owned(),
            "Collection_Publicid": collection["Publicid"]
        });

        return Ok((cookies, Json(result.to_string())));
    }

    if addtoalbum == true && CollectionId.is_some() && addtocollection == false {
        let properties = json!({
            "Poster": false,
            "Album": true,
            "Avatar": false
         });
 
         let insert_details = v_media::ActiveModel {
             id: ActiveValue::Set(ID.to_owned()),
             publicid: ActiveValue::Set(PublicId.to_owned()),
             title: ActiveValue::Set(Title.to_owned()),
             mediatype: ActiveValue::Set(Media::MediaType::Video.to_string()),
             uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
             username: ActiveValue::Set(Username.to_owned()),
             description: ActiveValue::NotSet,
             chapters: ActiveValue::NotSet,
             poster_storagepathorurl: ActiveValue::Set(Some(PosterVec)),
             storagepathorurl: ActiveValue::Set(Some(UploadPath.to_owned())),
             properties: ActiveValue::Set(Some(properties)),
             state: ActiveValue::Set(Media::MediaState::Uploading.to_string()),
         };
 
         insert_details.insert(&connection).await.unwrap();
 
         let collection = add_to_collection(details, cookies.to_owned()).await;
 
         let result = json!({
             "Result": "Success",
             "Publicid": PublicId.to_owned(),
             "Collection_Publicid": collection["Publicid"]
         });
 
         return Ok((cookies, Json(result.to_string())));
    }

    if addtocollection == false {
        let properties = json!({
            "Poster": false,
            "Album": false,
            "Avatar": false
        });
        let insert_details = v_media::ActiveModel {
            id: ActiveValue::Set(ID.to_owned()),
            publicid: ActiveValue::Set(PublicId.to_owned()),
            title: ActiveValue::Set(Title.to_owned()),
            mediatype: ActiveValue::Set(Media::MediaType::Video.to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(Username.to_owned()),
            description: ActiveValue::NotSet,
            chapters: ActiveValue::NotSet,
            poster_storagepathorurl: ActiveValue::Set(Some(PosterVec)),
            storagepathorurl: ActiveValue::Set(Some(UploadPath.to_owned())),
            properties: ActiveValue::Set(Some(properties)),
            state: ActiveValue::Set(Media::MediaState::Uploading.to_string()),
        };

        insert_details.insert(&connection).await.unwrap();
    } else {
        let result = json!({
            "Result": "Failure",
            "Publicid": PublicId.to_owned()
        });

        return Ok((cookies, Json(result.to_string())));
    }

    let total_frames = VideoFrames(&video_name.clone()).parse::<u32>().unwrap();

    let task_init = v_task::ActiveModel {
        id: ActiveValue::Set(PublicId.to_owned()),
        username: ActiveValue::Set(Username.to_owned()),
        r#type: ActiveValue::Set("Video upload".to_string()),
        progress: ActiveValue::Set("0%".to_string()),
    };

    let task_init: v_task::Model = task_init.insert(&connection).await.unwrap();

    let mut total_progress = String::new();

    FfmpegCommand::new()
        .input(video_name.clone().as_str())
        .hide_banner()
        .arg("-y")
        .args([
            "-s",
            "320x240",
            "-acodec",
            "libvorbis",
            "-vcodec",
            "libvpx-vp9",
            Paths[0].as_str(),
            "-s",
            "640x480",
            "-acodec",
            "libvorbis",
            "-vcodec",
            "libvpx-vp9",
            Paths[1].as_str(),
            "-s",
            "1280x720",
            "-acodec",
            "libvorbis",
            "-vcodec",
            "libvpx-vp9",
            Paths[2].as_str(),
        ])
        .spawn()
        .unwrap()
        .iter()
        .unwrap()
        .filter_progress()
        .for_each(|progress: FfmpegProgress| {
            let progress_value = (progress.frame * 100 / total_frames).to_string() + "%";
            let progress = progress_value.as_str();
            if total_progress.is_empty() {
                total_progress.push_str(progress);

                let _progress_result = Create_Progress(
                    PublicId.to_owned(),
                    Username.to_owned(),
                    "Video".to_string(),
                    total_progress.to_owned(),
                );
            } else {
                total_progress.push_str(progress);

                let _progress_result =
                    Update_Progress(PublicId.to_owned(), total_progress.to_owned());
            }
        });

    let mut VideoInt = 0;

    for p in Paths {
        let from = Path::new(p.as_str());
        let to = (PublicId.as_str().to_owned() + "/" + &Videos[VideoInt]);
        let video: Vec<u8> = fs::read(from).unwrap();
    
        op.0.write(&to, video).await.unwrap();
    
        VideoInt = VideoInt + 1;
    }

    std::fs::remove_dir_all(process.to_owned() + "/" + &VideoBucket + "/" + &PublicId.to_owned())
        .unwrap();

    let insert_video: Option<v_media::Model> = v_media::Entity::find()
        .filter(v_media::Column::Id.eq(ID))
        .one(&connection)
        .await
        .unwrap();

    let mut insert_video: v_media::ActiveModel = insert_video.unwrap().into();

    insert_video.storagepathorurl = Set(Some(UploadPath));

    insert_video.state = Set(Media::MediaState::Published.to_string());

    insert_video.update(&connection).await.unwrap();

    let results = json!({
        "Result": "Success",
        "PublicID": PublicId
    });

    return Ok((cookies, Json(results.to_string())));
}

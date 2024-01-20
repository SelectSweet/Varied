use super::*;

use Image::ProcessImages;

pub fn AudioFrames(file: &str) -> String {
    let command = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
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
pub struct AudioUpload {
    #[form_data(limit = "unlimited")]
    pub audio: FieldData<NamedTempFile>,

    #[form_data(limit = "unlimited")]
    pub poster: Option<FieldData<NamedTempFile>>,

    pub addtocollection: bool,
    pub addtoalbum: bool,
    pub title: Option<String>,
    pub Description: Option<String>,
    pub CollectionId: Option<String>,
}

#[debug_handler]
pub async fn UploadAudio(
    cookies: CookieJar,
    TypedMultipart(AudioUpload {
        audio,
        poster,
        addtocollection,
        addtoalbum,
        Description,
        CollectionId,
        title
    }): TypedMultipart<AudioUpload>,
) -> Result<(CookieJar, Json<String>), StatusCode> {
    let connection = establish_connection().await;

    //let mount_path = "Cache";

    let UID = Uuid::new_v4().to_string();
    let ID = UID.as_str();
    let mut Title = String::new();
    let mut RCloneConfig = get_rclone_config();
    let process = RCloneConfig["Process"].to_owned();
    let AudioBucket = RCloneConfig["Name"].to_owned();

    let Username = get_session(cookies.clone()).await;

    let name = audio.metadata.file_name.unwrap();
    let filetype = audio.metadata.content_type.unwrap();
    let path = Path::new("/tmp").join(name.to_owned());
    let PublicID = make_sqid(random_nums(12).await);
    let mut details: HashMap<String, String> = HashMap::new();

    std::fs::create_dir_all(process.to_owned() + "/" + &AudioBucket + "/" + ID).unwrap();

    let mut Paths: Vec<String> = Vec::new();
    Paths.push(process.to_owned() + "/" + &AudioBucket + "/" + ID + "/" + ID + "-High.flac");
    // Paths.push(
    //     process.to_owned() + "/" + &AudioBucket + "/" + ID + "/" + ID + "-.flac",
    // );
    // Paths.push(
    //     process.to_owned() + "/" + &AudioBucket + "/" + ID + "/" + ID + "-.flac",
    // );

    if Title == "" {
        Title.push_str(name.as_str())
    }

    audio.contents.persist(path.to_owned()).unwrap();

    let Audio_name = path.as_path().to_str().unwrap().to_string();

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
                title: Some(Title.to_owned())
            }),
            Username.to_owned(),
            true,
        )
        .await;

        let Poster_Url = RCloneConfig["Endpoint"].to_owned() + &Poster["Publicid"].to_string().as_str();

        PosterVec.push(Poster_Url);
    }

    if addtocollection == false {
        let insert_details = v_media::ActiveModel {
            id: ActiveValue::Set(ID.to_owned()),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(Title.to_owned()),
            mediatype: ActiveValue::Set(Media::MediaType::Audio.to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(Username.to_owned()),
            description: ActiveValue::NotSet,
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::NotSet,
            poster_storagepathorurl: ActiveValue::Set(Some(PosterVec.to_owned())),
            properties: ActiveValue::NotSet,
            state: ActiveValue::Set(Media::MediaState::Uploading.to_string()),
        };
    
        let insert_details: v_media::Model = insert_details.insert(&connection).await.unwrap();
    }

    if addtocollection == true && CollectionId.is_some() {
        let insert_details = v_media::ActiveModel {
            id: ActiveValue::Set(ID.to_owned()),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(Title.to_owned()),
            mediatype: ActiveValue::Set(Media::MediaType::Audio.to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(Username.to_owned()),
            description: ActiveValue::NotSet,
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::NotSet,
            poster_storagepathorurl: ActiveValue::Set(Some(PosterVec)),
            properties: ActiveValue::NotSet,
            state: ActiveValue::Set(Media::MediaState::Uploading.to_string()),
        };
    
        let insert_details: v_media::Model = insert_details.insert(&connection).await.unwrap();

        let collection = add_to_collection(details).await;
    
        let result = json!({
            "Result": "Success",
            "Publicid": PublicID,
            "Collection_Publicid": collection["Publicid"]
        });
    
        return Ok((cookies, Json(result.to_string())));
    }

    let mut total_progress = String::new();
    let total_frames = AudioFrames(Audio_name.as_str()).parse::<u32>().unwrap();

    FfmpegCommand::new()
        .input(Audio_name.as_str())
        .hide_banner()
        .arg("-y")
        .arg(Paths[0].as_str())
        .spawn()
        .unwrap()
        .iter()
        .expect("Iter not created")
        .filter_progress()
        .for_each(|progress: FfmpegProgress| {
            let progress_value = (progress.frame * 100 / total_frames).to_string() + "%";
            let progress = progress_value.as_str();
            if total_progress.is_empty() {
                total_progress.push_str(progress);
                let _progress_result = Create_Progress(
                    PublicID.to_owned(),
                    Username.to_owned(),
                    "Audio".to_string(),
                    total_progress.to_owned(),
                );
            } else {
                total_progress.push_str(progress);
                let _progress_result =
                    Update_Progress(PublicID.to_owned(), total_progress.to_owned());
            }
        });

    let bucket_path = AudioBucket.to_owned() + "/" + ID;

    let process_path = process.to_owned() + "/" + &AudioBucket + "/" + ID;
    Command::new("rclone")
        .arg("move")
        .arg(process_path)
        .arg(AudioBucket.to_owned() + ":" + &bucket_path)
        .arg("--delete-empty-src-dirs")
        .output()
        .unwrap();

    let mut UploadPaths: Vec<String> = Vec::new();

    let endpoint = RCloneConfig["Endpoint"].to_owned();

    UploadPaths.insert(
        0,
        endpoint.to_owned()
            + "/"
            + AudioBucket.to_owned().as_str()
            + "/"
            + ID
            + "/"
            + ID
            + "-High.flac",
    );

    // UploadPaths.insert(
    //     1,
    //     endpoint.to_owned()
    //         + "/"
    //         + AudioBucket.to_owned().as_str()
    //         + "/"
    //         + ID
    //         + "/"
    //         + ID
    //         + "-720.webm",
    // );

    // UploadPaths.insert(
    //     2,
    //     endpoint.to_owned()
    //         + "/"
    //         + AudioBucket.to_owned().as_str()
    //         + "/"
    //         + ID
    //         + "/"
    //         + ID
    //         + "-320.webm",
    // );

    let insert_audio: Option<v_media::Model> = v_media::Entity::find()
        .filter(v_media::Column::Id.eq(ID))
        .one(&connection)
        .await
        .unwrap();

    let mut insert_audio: v_media::ActiveModel = insert_audio.unwrap().into();

    insert_audio.storagepathorurl = Set(Some(UploadPaths));

    insert_audio.state = Set(Media::MediaState::Published.to_string());

    insert_audio.update(&connection).await.unwrap();

    let results = json!({
        "Result": "Success",
        "PublicID": PublicID
    });

    return Ok((cookies, Json(results.to_string())));
}

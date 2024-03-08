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


    let UID = Uuid::new_v4().to_string();
    let ID = UID.as_str();
    let mut Title = String::new();
    let mut object = get_object_config();
    let Process = object["Process"].to_owned();
    let AudioBucket = object["Name"].to_owned();

    let Username = get_session(cookies.clone()).await;

    let name = audio.metadata.file_name.unwrap();
    let filetype = audio.metadata.content_type.unwrap();
    let path = Path::new("/tmp").join(name.to_owned());
    let PublicID = make_sqid(random_nums(12).await);
    let mut details: HashMap<String, CollectionValues> = HashMap::new();

    std::fs::create_dir_all(Process.to_owned() + "/" + &AudioBucket + "/" + &PublicID.to_owned()).unwrap();

    let op = get_dal_op().await.unwrap();

    let mut Paths: Vec<String> = Vec::new();
    let mut Audios = Vec::new();

    Audios.push(PublicID.as_str().to_owned() + "_High.flac");


    Paths.push(Process.to_owned() + "/" + &AudioBucket + "/" + PublicID.to_owned().as_str() + "/" + Audios[0].as_str());
    // Paths.push(
    //     process.to_owned() + "/" + &AudioBucket + "/" + ID + "/" + ID + "-.flac",
    // );
    // Paths.push(
    //     process.to_owned() + "/" + &AudioBucket + "/" + ID + "/" + ID + "-.flac",
    // );

    if Title == "" {
        Title.push_str(name.as_str())
    }

    if CollectionId.is_some() {
        details.insert("Collection_Id".to_owned(), Collection::CollectionValues::String(CollectionId.to_owned().unwrap()));
    }

    if addtoalbum == true {
        details.insert("AddToAlbum".to_owned(), Collection::CollectionValues::String(true.to_string()));
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
            false,
            cookies.to_owned()
        )
        .await;

        let Poster_Url = object["Endpoint"].to_owned() + &Poster["Publicid"].to_string().as_str();

        PosterVec.push(Poster_Url);
    }

    if addtocollection == false {
        let properties = json!({
            "Poster": false,
            "Album": false,
            "Avatar": false
        });
        let insert_details = v_media::ActiveModel {
            id: ActiveValue::Set(ID.to_owned()),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(Title.to_owned()),
            mediatype: ActiveValue::Set(Media::MediaType::Audio.to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(Username.to_owned()),
            description: ActiveValue::NotSet,
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::Set(Some(Paths.to_owned())),
            poster_storagepathorurl: ActiveValue::Set(Some(PosterVec.to_owned())),
            properties: ActiveValue::NotSet,
            state: ActiveValue::Set(Media::MediaState::Uploading.to_string()),
        };
    
        let insert_details: v_media::Model = insert_details.insert(&connection).await.unwrap();
    }

    if addtocollection == true && CollectionId.is_some() {

        let properties = json!({
            "Poster": false,
            "Album": false,
            "Avatar": false
        });

        let insert_details = v_media::ActiveModel {
            id: ActiveValue::Set(ID.to_owned()),
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(Title.to_owned()),
            mediatype: ActiveValue::Set(Media::MediaType::Audio.to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(Username.to_owned()),
            description: ActiveValue::NotSet,
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::Set(Some(Paths.to_owned())),
            poster_storagepathorurl: ActiveValue::Set(Some(PosterVec)),
            properties: ActiveValue::NotSet,
            state: ActiveValue::Set(Media::MediaState::Uploading.to_string()),
        };
    
        let insert_details: v_media::Model = insert_details.insert(&connection).await.unwrap();

        let collection = add_to_collection(details, cookies.to_owned()).await;
    
        let result = json!({
            "Result": "Success",
            "Publicid": PublicID,
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
            publicid: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(Title.to_owned()),
            mediatype: ActiveValue::Set(Media::MediaType::Audio.to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(Username.to_owned()),
            description: ActiveValue::NotSet,
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::Set(Some(Paths.to_owned())),
            poster_storagepathorurl: ActiveValue::Set(Some(PosterVec)),
            properties: ActiveValue::NotSet,
            state: ActiveValue::Set(Media::MediaState::Uploading.to_string()),
        };
    
        let insert_details: v_media::Model = insert_details.insert(&connection).await.unwrap();

        let collection = add_to_collection(details, cookies.to_owned()).await;
    
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

    

    let mut UploadPaths: Vec<String> = Vec::new();

    let ProcessFolder = Process.to_owned() + "/" + &AudioBucket;

    // let mut AudioInt = 0;

    // for p in Paths {
    //     let from = Path::new(p.as_str());
    //     let to = (PublicID.as_str().to_owned() + "/" + &Audios[AudioInt]);
    //     println!("From: {:?}", from);
    //     let audio: Vec<u8> = fs::read(from).unwrap();

    //     op.0.write(&to, audio).await.unwrap();

    //     AudioInt = AudioInt + 1;
    // }
     
    let from = Paths[0].to_owned();
    let FromPath = Path::new(from.as_str());
    let to = (PublicID.as_str().to_owned() + "/" + &Audios[0]);
    let audio: Vec<u8> = fs::read(FromPath.to_owned()).unwrap();
    op.0.write(&to, audio).await.unwrap();  

    std::fs::remove_dir_all(Process.to_owned() + "/" + &AudioBucket + "/" + &PublicID.to_owned()).unwrap();

    UploadPaths.push(ProcessFolder.to_owned() + "/" + ID + "-High.flac");
    //UploadPaths.push(ProcessFolder.to_owned() + "/" + ID.as_str() + "-.flac");

    let endpoint = object["Endpoint"].to_owned();

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

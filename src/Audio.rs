use super::*;

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

#[debug_handler]
pub async fn UploadAudio(
    cookies: CookieJar,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(CookieJar, Json<String>), StatusCode> {
    let connection = establish_connection().await;

    //let mount_path = "Cache";

    let UID = Uuid::new_v4().to_string();
    let ID =  UID.as_str();
    let PublicId = encode_base64_id(Uuid::new_v4().to_string());
    let mut Title = String::new();
    let mut RCloneConfig = get_rclone_config();
    let process = RCloneConfig["Process"].to_owned();
    let AudioBucket = RCloneConfig["Name"].to_owned();

    let Username = get_session(cookies.clone()).await;

    //std::fs::create_dir_all(mount_path.to_owned() + "/" + PublicId.as_str()).unwrap();

    let mut Paths: Vec<String> = Vec::new();
    Paths.push(
        process.to_owned() + "/" + &AudioBucket + "/" + ID + "/" + ID + ".flac",
    );
    // Paths.push(
    //     process.to_owned() + "/" + &AudioBucket + "/" + ID + "/" + ID + "-.flac",
    // );
    // Paths.push(
    //     process.to_owned() + "/" + &AudioBucket + "/" + ID + "/" + ID + "-.flac",
    // );

    let process_dir = process.to_owned() + "/" + ID;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        if Title == "" {
            Title.push_str(file_name.as_str())
        }

        let InputAudioPath = std::path::Path::new(&file_name);

        let AudioFileName = PublicId.as_str().to_owned()
            + "-"
            + InputAudioPath.extension().unwrap().to_str().unwrap();

        let Audio_name = process.to_owned() + "/" + ID + "/" + AudioFileName.as_str();

        let AudioFilePath = std::path::Path::new(&Audio_name);

        let mut AudioFile = File::create(AudioFilePath.as_os_str()).unwrap();

        AudioFile
            .write_all(&data)
            .expect("Data not Written to File");

        // gets the current datetime
        let now = Utc::now();

        let insert_details = v_media::ActiveModel {
            id: ActiveValue::Set(ID.to_owned()),
            publicid: ActiveValue::Set(PublicId.to_owned()),
            title: ActiveValue::Set(Title.to_owned()),
            mediatype: ActiveValue::Set(Media::MediaType::Audio.to_string()),
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())),
            username: ActiveValue::Set(Username.to_owned()),
            description: ActiveValue::NotSet,
            chapters: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::NotSet,
            poster_storagepathorurl: ActiveValue::NotSet,
            properties: ActiveValue::NotSet,
            state: ActiveValue::Set(Media::MediaState::Uploading.to_string()),
        };

        let insert_details: v_media::Model = insert_details.insert(&connection).await.unwrap();

        let mut total_progress: Vec<String> = Vec::new();
        let total_frames = AudioFrames(Audio_name.as_str()).parse::<u32>().unwrap();
        println!("Total frame: {}", total_frames);

        FfmpegCommand::new()
            .input(&Audio_name.as_str())
            .hide_banner()
            .arg("-y")
            .args(
                ["-s", "-c:a", "flac", Paths[0].as_str()
                //"-s", "-c:a", "flac", "",
                //"-s", "-c:a", "flac", ""
                ]
            )
            .spawn()
            .unwrap()
            .iter()
            .expect("Iter not created")
            .filter_progress()
            .for_each(|progress: FfmpegProgress| {
                let progress = (progress.frame * 100 / total_frames).to_string() + "%";
                if total_progress.is_empty() {
                    total_progress[0] = progress;

                    let _progress_result = Create_Progress(
                        PublicId.to_owned(),
                        Username.to_owned(),
                        "Audio".to_string(),
                        total_progress[0].to_owned(),
                    );
                } else {
                    total_progress[0] = progress;

                    let _progress_result =
                        Update_Progress(PublicId.to_owned(), total_progress[0].to_owned());
                }
            });
        std::fs::remove_dir_all(process_dir.to_owned()).unwrap();

        let mut BucketKey = String::new();
        let bucket_path = AudioBucket.to_owned() + "/" + ID;

        BucketKey.insert(AudioBucket.len(), ':');
        BucketKey.insert_str(AudioBucket.len(), &bucket_path);

        BucketKey.insert_str(0, &AudioBucket);

        let process_path = process.to_owned() + "/" + &AudioBucket + "/" + ID;
        Command::new("rclone")
            .arg("move")
            .arg(process_path)
            .arg(BucketKey)
            .output()
            .unwrap();

        let mut UploadPaths: Vec<String> = Vec::new();

        let endpoint = RCloneConfig["endpoint"].to_owned();

        UploadPaths.insert(
            0,
            endpoint.to_owned()
                + "/"
                + AudioBucket.to_owned().as_str()
                + "/"
                + ID
                + "/"
                + ID
                + ".flac",
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
            "PublicID": PublicId
        });
    };

    return Ok((cookies, Json("Success".to_string())));
}

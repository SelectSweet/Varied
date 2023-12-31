use super::*;

pub fn VideoFrames(file: &str) -> String {
    let command = Command::new("ffprobe")
        .args([ "-v", "error", "-select_streams", "v:0", "-count_packets", "-show_entries", "stream=nb_read_packets", "-of", "csv=p=0"])
        .arg(file)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let out = command.wait_with_output().unwrap().stdout;
    let binding = std::str::from_utf8(&out.to_owned()).unwrap().to_string();
    let OutString = binding.strip_suffix("\n").unwrap();
    //println!("Frames: {}", OutString);
    return OutString.to_string()
}


#[debug_handler]
pub async fn UploadVideo(
    cookies: CookieJar,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(CookieJar, Json<String>), StatusCode>  {

    let connection = establish_connection().await;

    //let c = connection.begin().await.unwrap();
    
    let ID = Uuid::new_v4().to_string();
    let PublicId = make_sqid(random_nums(10).await);
    let mut Title = String::new();
    let RCloneConfig = get_rclone_config();

    let process = RCloneConfig["Process"].to_owned();   
    
    let VideoBucket = RCloneConfig["Name"].to_owned();
    
    
    let Username = get_session(cookies.clone()).await.replace("'", "").replace("\"", "");

    let mut Paths: Vec<String> = Vec::new();

    Paths.push(process.to_owned() + "/" + &VideoBucket + "/" + ID.as_str() + "/" + ID.as_str() + "-320.webm");
    Paths.push(process.to_owned() + "/" + &VideoBucket + "/" + ID.as_str() + "/" + ID.as_str() + "-720.webm");
    Paths.push(process.to_owned() + "/" + &VideoBucket + "/" + ID.as_str() + "/" + ID.as_str() + "-1280.webm");

    let process_dir = process.to_owned() + "/" + ID.as_str();

    std::fs::create_dir_all(&process_dir).unwrap();
    std::fs::create_dir_all(process.to_owned() + "/" + &VideoBucket + "/" + ID.as_str()).unwrap();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        
        if Title == "" {
           Title.push_str(file_name.as_str())
        }        
 
        let InputVideoPath = std::path::Path::new(&file_name);

        let VideoFileName = ID.as_str().to_owned() + "." + InputVideoPath.extension().unwrap().to_str().unwrap(); 

        let video_name = process.to_owned() + "/" + ID.as_str() + "/" + VideoFileName.as_str();

        let VideoFilePath = std::path::Path::new(&video_name);

        let mut VideoFile = File::create(VideoFilePath.as_os_str()).unwrap();

        VideoFile.write(&data).unwrap();

        // gets the current datetime
        let now = Utc::now();

        let insert_details = v_media::ActiveModel { 
            id: ActiveValue::Set(ID.to_owned()), 
            publicid: ActiveValue::Set(PublicId.to_owned()), 
            title: ActiveValue::Set(Title.to_owned()), 
            mediatype: ActiveValue::Set(Media::MediaType::Video.to_string()), 
            uploaded_at: ActiveValue::Set(DateTime::new(now.date_naive(), now.time())), 
            username: ActiveValue::Set(Username.to_owned()), 
            description: ActiveValue::NotSet, 
            chapters: ActiveValue::NotSet,
            poster_storagepathorurl: ActiveValue::NotSet,
            storagepathorurl: ActiveValue::NotSet, 
            properties: ActiveValue::NotSet, 
            state: ActiveValue::Set(Media::MediaState::Uploading.to_string())
        };

        insert_details.insert(&connection).await.unwrap();

        let total_frames = VideoFrames(&video_name.clone()).parse::<u32>().unwrap();
       

        let task_init = v_task::ActiveModel { 
            id: ActiveValue::Set(PublicId.to_owned()), 
            username: ActiveValue::Set(Username.to_owned()), 
            r#type: ActiveValue::Set("Video upload".to_string()), 
            progress: ActiveValue::Set("0%".to_string())
        };

        let task_init: v_task::Model = task_init.insert(&connection).await.unwrap();

        let mut total_progress = Vec::new();

        
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
    .spawn().unwrap()
    .iter()
    .unwrap()
    .filter_progress()
    .for_each(|progress: FfmpegProgress| {
        let progress = (progress.frame * 100 / total_frames).to_string() + "%";
         if total_progress.is_empty() {                    
           total_progress.push(progress);

           let _progress_result = Create_Progress(PublicId.to_owned(), Username.to_owned(), "Video".to_string(), total_progress[0].to_owned());
        
        }
        
        else {
           total_progress[0] = progress;

           let _progress_result = Update_Progress(PublicId.to_owned(), total_progress[0].to_owned());

        }
    });

    };

    let mut BucketKey = String::new();
    let bucket_path = VideoBucket.to_owned() + "/" + ID.as_str();

    BucketKey.insert(VideoBucket.len(), ':');
    BucketKey.insert_str(VideoBucket.len(), &bucket_path);

    BucketKey.insert_str(0, &VideoBucket);

    std::fs::remove_dir_all(process_dir).unwrap();

    let process_path = process.to_owned() + "/" + &VideoBucket + "/" + ID.as_str();

    Command::new("rclone")
       .arg("move")
       .arg(process_path)
       .arg(BucketKey)
       .arg("--delete-empty-src-dirs")
       .output()
       .unwrap();

    let mut UploadPaths: Vec<String> = Vec::new();

    let endpoint = RCloneConfig["endpoint"].to_owned();


    UploadPaths.insert(
        0,
        endpoint.to_owned() + "/" + VideoBucket.to_owned().as_str() + "/" + ID.as_str() + "/" + ID.as_str() + "-1280.webm"
    );

    UploadPaths.insert(
        1,
        endpoint.to_owned() + "/" + VideoBucket.to_owned().as_str() + "/" + ID.as_str() + "/" + ID.as_str() + "-720.webm"
    );

    UploadPaths.insert(
        2,
        endpoint.to_owned() + "/" + VideoBucket.to_owned().as_str() + "/" + ID.as_str() + "/" + ID.as_str() + "-320.webm"
    );

    let insert_video: Option<v_media::Model> = v_media::Entity::find().filter(v_media::Column::Id.eq(ID)).one(&connection).await.unwrap();

    let mut insert_video: v_media::ActiveModel = insert_video.unwrap().into();

    insert_video.storagepathorurl = Set(Some(UploadPaths));

    insert_video.state = Set(Media::MediaState::Published.to_string());

    insert_video.update(&connection).await.unwrap();

    let results = json!({
        "Result": "Success",
        "PublicID": PublicId
    });
    
    return Ok((
        cookies,
        Json(results.to_string())
    ));
}
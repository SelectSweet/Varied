use super::*;

async fn ProcessImages(mut multipart: Multipart, details: Option<Json<String>>) -> String {
    let mut Paths: Vec<String> = Vec::new();
    let ID: Uuid;

    if details.is_some() {

    }

    else {
        let ID = Uuid::new_v4().to_string();
        let PublicId = make_sqid(random_nums(10).await);        
    }

    let process = "";
    let Bucket = "";
    let RCloneConfig = get_rclone_config();

    Paths.push(process.to_owned() + "/" + Bucket + "/" + ID.as_str() + "/" + ID.as_str() + "");
    Paths.push(process.to_owned() + "/" + Bucket + "/" + ID.as_str() + "/" + ID.as_str() + "");
    Paths.push(process.to_owned() + "/" + Bucket + "/" + ID.as_str() + "/" + ID.as_str() + "");

    return "Return a Proper Thing".to_string()
}



pub async fn UploadImage(mut multipart: Multipart) {
    
}
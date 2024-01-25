use super::*;

#[derive(Debug, Clone)]
pub enum CollectionValues {
    Vec(Vec<String>),
    String(String)
}

impl Display for CollectionValues{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CollectionValues::String(s) => write!(f, "{s}"),
            CollectionValues::Vec(v) => {
                for o in v {
                    write!(f, "{o}").unwrap();
                }
                Ok(())
            },
        }
    }
}


pub async fn add_to_collection(details: HashMap<String, CollectionValues>, cookies: CookieJar) -> Value {
    let connection = establish_connection().await;

    let mut Id: Vec<&str> = Vec::new();
    let mut CollectionId = String::new();
    let mut MediaType = Vec::new();

    match &details["Id"] {
        CollectionValues::Vec(id) => {
            for i in id {
                Id.push(i)
            }
        },
        CollectionValues::String(id) => {
            Id.push(id);
        },
    }

    match &details["Collection_Id"] {
        CollectionValues::Vec(id) => {
            println!("Only one Collection ID Allowed");
        },
        CollectionValues::String(id) => {
            CollectionId.push_str(id);
        },
    }

    match &details["Type"] {
        CollectionValues::Vec(t) => {
            for i in t {
                MediaType.push(i)
            }
        },
        CollectionValues::String(t) => {
            MediaType.push(t);
        },
    }


    let mut collection = v_collection::Entity::find_by_id(CollectionId.to_owned()).into_json().one(&connection).await.unwrap().unwrap();
    
    for t in MediaType {
        match t.as_str() {
            "Audio" => {
                let Audio = collection["Properties"]["Audio"].as_u64().unwrap() + 1;
                collection["Properties"]["Audio"] = serde_json::to_value(Audio.to_string()).unwrap();
            }
            "Image" => {
                let Image = collection["Properties"]["Image"].as_u64().unwrap() + 1;
                collection["Properties"]["Image"] = serde_json::to_value(Image.to_string()).unwrap();
            }
            "Video" => {
                let Video = collection["Properties"]["Video"].as_u64().unwrap() + 1;
                collection["Properties"]["Video"] = serde_json::to_value(Video.to_string()).unwrap();
                
            }
            "Text" => {
                let Video = collection["Properties"]["Text"].as_u64().unwrap() + 1;
                collection["Properties"]["Text"] = serde_json::to_value(Video.to_string()).unwrap();
                
            }
            "Note" => {
                let Video = collection["Properties"]["Note"].as_u64().unwrap() + 1;
                collection["Properties"]["Note"] = serde_json::to_value(Video.to_string()).unwrap();
                
            }
            _ => {},
        }
    }
    

    let id_name = "public_id";
    let id_value = &collection[id_name];
    let id_string = &id_value.to_owned().to_string();
    let mut current_ids: Vec<&str> = id_string.split(",").collect();
    Id.append(&mut current_ids);
    let id_vec_json = json!(Id.to_vec());

    let UpdateCollection: Option<v_collection::Model> = v_collection::Entity::find_by_id(CollectionId).one(&connection).await.unwrap();
    
    let mut UpdateCollection: v_collection::ActiveModel = UpdateCollection.unwrap().into();

    UpdateCollection.i_ds = Set(Some(id_vec_json));
    UpdateCollection.properties = Set(Some(collection["Properties"].to_owned()));

    let UpdateCollection: v_collection::Model = UpdateCollection.update(&connection).await.unwrap();

    let result = json!({
        "Result": "Success",
    });

    return result;
}

pub async fn Create_Collection(
    cookies: CookieJar,
    Json(params): Json<HashMap<String, String>>
) -> Json<String> {

    let Username = get_session(cookies.clone()).await.replace("'", "").replace("\"", "");


    let title = params["Title"].to_owned();
    let description = Some(params["Description"].to_owned());
    let CollectionType = params["CollectionType"].to_owned();
    let state = params["State"].to_owned();
    let properties = json!({
        "Audio": 0,
        "Image": 0,
        "Text": 0,
        "Note": 0,
        "Video": 0,
    });

    let PublicID = make_sqid(random_nums(12).await);
    
    let connection = establish_connection().await;

    if CollectionType == "List" {
        let collection = v_collection::ActiveModel {
            public_id: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(title),
            description: ActiveValue::Set(description),
            r#type: ActiveValue::Set(CollectionType),
            i_ds: ActiveValue::NotSet,
            properties: ActiveValue::Set(Some(properties)),
            state: ActiveValue::Set(state),
            username: ActiveValue::Set(Username),
        };
        let collection: v_collection::Model = collection.insert(&connection).await.unwrap();

        let result = json!({
            "Result": "Success",
            "Publicid": PublicID
        });
    
        return Json(result.to_string());
    }

    if CollectionType == "Album" {
        let collection = v_collection::ActiveModel {
            public_id: ActiveValue::Set(PublicID.to_owned()),
            title: ActiveValue::Set(title),
            description: ActiveValue::Set(description),
            r#type: ActiveValue::Set(CollectionType),
            i_ds: ActiveValue::NotSet,
            properties: ActiveValue::Set(Some(properties)),
            state: ActiveValue::Set(state),
            username: ActiveValue::Set(Username),
        };
        let collection: v_collection::Model = collection.insert(&connection).await.unwrap();

        let result = json!({
            "Result": "Success",
            "Publicid": PublicID
        });
    
        return Json(result.to_string());
    }

    else {
        let result = json!({
            "Result": "Failure",
            "Publicid": PublicID
        });
    
        return Json(result.to_string());
    }
}



pub async fn Update_Collection(
    cookies: CookieJar,
    Json(mut params): Json<HashMap<String, String>>
) -> Json<String> {
    let connection = establish_connection().await;
    let IdVec: Vec<&str> = params["Id"].split(",").collect();

    let mut details: HashMap<String, CollectionValues>  = HashMap::new();

    for (k, v) in params.to_owned() {
        details.insert(k, Collection::CollectionValues::String(v));
    }

    let mut Ids: Vec<String> = Vec::new();
    let mut Types: Vec<String> = Vec::new();

    for i in IdVec.into_iter() {

        let ViewMedia = v_media::Entity::find().columns([
            v_media::Column::Publicid,
            v_media::Column::Mediatype
        ]
        ).filter(v_media::Column::Publicid.eq(i.to_string())).one(&connection).await.unwrap().unwrap();
        Ids.push(i.to_string());
        Types.push(ViewMedia.mediatype);
    }
    params.insert("Id".to_string(), Collection::CollectionValues::Vec(Ids).to_string());
    params.insert("Type".to_string(), Collection::CollectionValues::Vec(Types).to_string());

    let update = add_to_collection(details, cookies).await;

    return Json(update.to_string());
}
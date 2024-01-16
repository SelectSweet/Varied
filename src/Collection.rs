use super::*;


pub async fn add_to_collection(details: HashMap<String, String>) -> Value {
    let connection = establish_connection().await;
    let CollectionId = details["Collection_Id"].to_owned();
    let CollectionType = details["CollectionType"].to_owned();
    let MediaType = details["Type"].to_owned();

    let mut collection = v_collection::Entity::find_by_id(CollectionId.to_owned()).into_json().one(&connection).await.unwrap().unwrap();
    
    match MediaType.as_str() {
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
        _ => {},
    }

    if details["addtoalbum"] == "true" {
        collection["Properties"]["InAlbum"] = json!(true);
    }


    let ids = collection["ID"].as_str().unwrap().to_owned() + "," + CollectionId.as_str();

    let ids_json = json!(ids);    

    let UpdateCollection: Option<v_collection::Model> = v_collection::Entity::find_by_id(CollectionId).one(&connection).await.unwrap();
    
    let mut UpdateCollection: v_collection::ActiveModel = UpdateCollection.unwrap().into();

    UpdateCollection.i_ds = Set(Some(ids_json));
    UpdateCollection.properties = Set(Some(collection["Properties"].to_owned()));

    let UpdateCollection: v_collection::Model = UpdateCollection.update(&connection).await.unwrap();

    let result = json!({
        "Result": "Success",
    });

    return result;
}

pub async fn Create_Collection(Json(params): Json<HashMap<String, String>>) -> Json<String> {
    let title = params["Title"].to_owned();
    let description = Some(params["Description"].to_owned());
    let CollectionType = params["CollectionType"].to_owned();
    let state = params["State"].to_owned();
    let properties = json!({
        "Audio": 0,
        "Image": 0,
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

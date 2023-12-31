use super::*;

pub async fn Create_Progress(id: String, Username: String, Type: String, Progress: String) -> String {
    let connection = establish_connection().await;
    let progress = v_task::ActiveModel {
        id: ActiveValue::Set(id),
        username: ActiveValue::Set(Username),
        r#type: ActiveValue::Set(Type),
        progress: ActiveValue::Set(Progress),
    };

    let progress: v_task::Model = progress.insert(&connection).await.unwrap();

    return "Success".to_string();
}

pub async fn Update_Progress(PublicId: String, progress: String) -> String {
    let connection = establish_connection().await;

    let task: Option<v_task::Model> = v_task::Entity::find_by_id(&PublicId.to_owned())
        .one(&connection)
        .await
        .unwrap();

    let mut task: v_task::ActiveModel = task.unwrap().into();

    task.progress = Set(progress.to_owned());

    let task: v_task::Model = task.update(&connection).await.unwrap();
    
    return "Success".to_string();
}



pub async fn list_tasks(cookies: CookieJar) -> Result<(CookieJar, Json<String>), StatusCode> {
    // Database Connection
    let connection = establish_connection().await;

    // Get Username from Session ID in Cookie
    let Username = get_session(cookies.to_owned()).await;

    let TaskQuery = format!("SELECT * FROM v_task WHERE username = '{}'", Username);

    let tasks: Value = v_task::Entity::find()
        //.filter(v_task::Column::Username.eq(Username))
        //.columns([ v_task::Column::Id, v_task::Column::Progress, v_task::Column::Type, ])
        .from_raw_sql(Statement::from_string(
            DbBackend::Postgres,
            TaskQuery,
        ))
        .into_json()
        .one(&connection)
        .await
        .unwrap()
        .unwrap();

    return Ok((cookies, Json(tasks.as_str().unwrap().to_string())));
}

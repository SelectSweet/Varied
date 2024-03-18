
use super::*;

#[derive(Deserialize)]
pub struct Config {
    pub database: database,
    pub Object: Object,
    pub Core: Core,
    pub S3: ConfigS3
}

#[derive(Deserialize)]
pub struct Core {
    pub file_size_limit: String,
    pub front_end_url: Url,
    pub main_url: String,
}


#[derive(Deserialize)]
pub struct database {
    pub url: String,
}

#[derive(Deserialize)]
pub struct Object {
    pub name: String,
    pub endpoint: String,
    pub process: String
}

#[derive(Deserialize, Debug)]
pub struct ConfigS3 {
    pub bucket: String,
    pub root: String,
    pub region: String,
    pub endpoint: String,
    pub access_key_id: String,
    pub secret_access_key: String
}

pub async fn establish_connection() -> DatabaseConnection {

    // Reads config file
    let database_url = File::open("varied.toml");

    // Gets Database url from read config file then adds it to empty string
    let mut dstring = String::new();
    database_url.unwrap().read_to_string(&mut dstring).unwrap();

    // Converts Database url string to Config Struct then get the url from the struct
    let read_url: Config = toml::from_str(&dstring).unwrap();
    let url = read_url.database.url;

    // Connection Options
    let mut options = ConnectOptions::new(url);

    // Connection to Database
    let connection = Database::connect(options).await.unwrap();

    return connection;

}

const CUSTOM_ENGINE: engine::GeneralPurpose = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

pub fn encode_base64_id(id: String) -> String {
    let baseid = CUSTOM_ENGINE.encode(id);
    return baseid
}

pub fn make_sqid(nums: Vec<u64>) -> String {
    let sqid = Sqids::new(Some(Options::new(
        Some("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".to_string()), 
        Some(5), 
        None
    ))).unwrap();

    let id = sqid.encode(&nums).unwrap();

    return id;


}

pub async fn random_nums(range: u8) -> Vec<u64> {
    const CHARSET: &[u8] = b"0123456789";
    let mut rng = thread_rng();

    let rand_nums: Vec<u64> = (0..range).map(
        |_| {
            let rand_s = rng.gen_range(0..CHARSET.len());
            CHARSET[rand_s] as u64
        }
    ).collect();

    return rand_nums
}


pub async fn random_alpha(range: u8) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = thread_rng();

    let rand_string: String = (0..range).map(
        |_| {
            let rand_s = rng.gen_range(0..CHARSET.len());
            CHARSET[rand_s] as char
        }
    ).collect();

    return rand_string
}



pub async fn get_session(cookies: CookieJar) -> String {

    let connection = establish_connection().await;
    let session_id = cookies.get("id").unwrap().to_string().replace("id=", "");

    let session = v_session::Entity::find_by_id(session_id.clone())
    .into_json()
    .one(&connection).await.unwrap().unwrap()
    ;

    let Username = session["username"].to_string();    

    return Username;
}



async fn s3_builder(config: String) -> S3 {
    let mut builder = S3::default();

    let config: Config = toml::from_str(&config).unwrap();

    let Config = config.S3;

    builder.root(&Config.root);

    builder.bucket(&Config.bucket);

    builder.region(&Config.region);

    builder.endpoint(&Config.endpoint);

    builder.access_key_id(&Config.access_key_id);

    builder.secret_access_key(&Config.secret_access_key);

    return builder;
}

pub async fn get_dal_op() -> Result<(Operator, String, String), String>  {

    let Config = File::open("varied.toml");

    let mut cstring = String::new();
    Config.unwrap().read_to_string(&mut cstring).unwrap();

    let object: Config = toml::from_str(&cstring.to_owned()).unwrap();
    let ObjectName = object.Object.name;
    let ObjectEndpoint = object.Object.endpoint;
    let ObjectProccess = object.Object.process;

    if ObjectName == "S3" {
        let builder = s3_builder(cstring).await;
        let op: Operator = Operator::new(builder).unwrap().finish();
        return Ok((op, ObjectEndpoint, ObjectProccess));
    }

    else {
        return Err("Invaild Object Settings".to_string())
    }
        
}

pub fn get_object_config() -> HashMap<String, String> {
    // Reads config file
    let object = File::open("varied.toml");

    // Gets RClone Details from read config file then adds it to empty string
    let mut Ostring = String::new();
    object.unwrap().read_to_string(&mut Ostring).unwrap();

    // Converts Rstring to Config Struct then get name, endpoint and process from the struct
    let read_config: Config = toml::from_str(&Ostring).unwrap();
    let name = read_config.Object.name;
    let endpoint = read_config.Object.endpoint;
    let process: String = read_config.Object.process;

    // Insert Key and Value of Name, Endpoint and Process into Config Hashmap
    let mut Config: HashMap<String, String> = HashMap::new();
    Config.insert("Name".to_string(), name);
    Config.insert("Endpoint".to_string(), endpoint.to_string());
    Config.insert("Process".to_string(), process);

    
    // Returns Config Hashmap
    return Config   
}

pub fn get_core_config() -> (Url, Url, usize) {
    // Reads config file
    let CoreConfig = File::open("varied.toml");

    // Gets CoreConfig Details from read config file then adds it to empty string
    let mut Cstring = String::new();
    CoreConfig.unwrap().read_to_string(&mut Cstring).unwrap();

    // Converts Rstring to Config Struct then get front_end and main url from the struct
    let read_config: Config = toml::from_str(&Cstring).unwrap();
    let file_size = read_config.Core.file_size_limit.parse::<usize>().unwrap();
    let front_end_url = Url::parse(read_config.Core.front_end_url.as_str()).unwrap();
    let main_url = Url::parse(read_config.Core.main_url.as_str()).unwrap();

    return (front_end_url, main_url, file_size);
}
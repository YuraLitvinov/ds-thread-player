use std::{env, fs};
use crate::error::snafu_error::{ErrorHandling, IoOperationSnafu};
use yaml_rust2::{YamlLoader, Yaml};
use snafu::ResultExt;

#[derive(Debug)]
pub struct PostgreConfig {
    pub host: String,
    pub username: String,
    pub password: String
}

pub fn return_prompt() -> Result<PostgreConfig, ErrorHandling> {
    let cur_dir = env::current_dir().unwrap();
    let path = cur_dir.join("config.yaml");
    let config =
        fs::read_to_string(&path).context(IoOperationSnafu { path: path.clone() })?;
    let docs = YamlLoader::load_from_str(&config)?;
    let doc = &docs[0];
        if let Yaml::Hash(h) = doc {
            let host = h
                .get(&Yaml::String("host".into()))
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            let username = h
                .get(&Yaml::String("username".into()))
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            let password = h
                .get(&Yaml::String("password".into()))
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            Ok(PostgreConfig { host, username, password })
        }

        else {
            panic!("Err")
        }
        
}
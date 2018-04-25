
#[derive(Debug)]
pub struct AppConfig<'a> {
    pub bootstrap_server: &'a str
}

pub fn from<'a>(args: &'a Vec<String>) -> AppConfig<'a> {
    AppConfig {
        bootstrap_server: args.get(1).expect("Missing bootstrap server URL")
    }
}
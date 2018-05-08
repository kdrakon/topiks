
#[derive(Debug, Copy, Clone)]
pub struct AppConfig<'a> {
    pub bootstrap_server: &'a str,
    pub request_timeout_ms: i32
}

pub fn from<'a>(args: &'a Vec<String>) -> AppConfig<'a> {
    AppConfig {
        bootstrap_server: args.get(1).expect("Missing bootstrap server URL"),
        request_timeout_ms: 30000
    }
}
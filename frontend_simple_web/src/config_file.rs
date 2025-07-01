// frontend_simple_web/src/config_file.rs
const DEFAULT_API_URL: &str = "http://localhost:8000";
const CONFIG_FILE : &str = "/config/.env";


pub fn get_env_var(key: &str) -> String {
    let content = fs::read_to_string(CONFIG_FILE)
        .unwrap_or_else(|_| DEFAULT_API_URL.to_string());

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, '=');
        let k = parts.next().unwrap().trim();
        let v = parts.next().unwrap_or("").trim().trim_matches('"');
        if k == key {
            return v.to_string();
        }
    }

    DEFAULT_API_URL.to_string()
}
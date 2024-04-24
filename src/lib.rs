use std::env;

pub fn expand_tilde(s: &str) -> String {
    if s.starts_with("~/") {
        format!(
            "{}/{}",
            env::var("HOME").unwrap(),
            s.strip_prefix("~/").unwrap()
        )
    } else {
        s.to_owned()
    }
}

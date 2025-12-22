pub use playtime::log_playtime;

mod playtime;

pub fn expand_tilde(s: &str) -> String {
    use std::env;

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

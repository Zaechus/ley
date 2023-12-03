pub fn expand_tilde(s: &str, home: &str) -> String {
    if s.starts_with("~/") {
        format!("{}/{}", home, s.strip_prefix("~/").unwrap())
    } else {
        s.to_owned()
    }
}

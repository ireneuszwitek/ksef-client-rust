#[derive(Debug, Clone, Copy)]
pub struct Url {
    base: &'static str,
}

impl Url {
    pub fn new(base: &'static str) -> Self {
        Self { base }
    }

    pub fn join(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_simple() {
        let url = Url { base: "http://example.com".into() };
        let result = url.join("path");
        assert_eq!(result, "http://example.com/path");
    }

    #[test]
    fn join_base_with_trailing_slash() {
        let url = Url { base: "http://example.com/".into() };
        let result = url.join("path");
        assert_eq!(result, "http://example.com/path");
    }

    #[test]
    fn join_path_with_leading_slash() {
        let url = Url { base: "http://example.com".into() };
        let result = url.join("/path");
        assert_eq!(result, "http://example.com/path");
    }

    #[test]
    fn join_both_have_slashes() {
        let url = Url { base: "http://example.com/".into() };
        let result = url.join("/path");
        assert_eq!(result, "http://example.com/path");
    }    
}
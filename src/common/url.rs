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

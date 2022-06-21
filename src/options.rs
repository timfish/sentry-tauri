use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Options {
    pub release: String,
    pub init_javascript_sdk: bool,
    pub sentry_debug: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            release: "".to_owned(),
            init_javascript_sdk: true,
            #[cfg(not(debug_assertions))]
            sentry_debug: false,
            #[cfg(debug_assertions)]
            sentry_debug: true,
        }
    }
}

impl From<String> for Options {
    fn from(release: String) -> Self {
        Options {
            release,
            ..Default::default()
        }
    }
}

impl From<&str> for Options {
    fn from(release: &str) -> Self {
        Options {
            release: release.to_owned(),
            ..Default::default()
        }
    }
}

impl From<Option<Cow<'_, str>>> for Options {
    fn from(release: Option<Cow<'_, str>>) -> Self {
        Options {
            release: release.expect("A release should be supplied").into_owned(),
            ..Default::default()
        }
    }
}

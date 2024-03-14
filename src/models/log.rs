use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Log {
    content: String,
    continuation_token: String,
}

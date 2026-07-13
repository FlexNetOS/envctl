pub const ROOT_ENV: &str = "META_ROOT";
fn p() -> String { std::env::var("META_ROOT").unwrap() }

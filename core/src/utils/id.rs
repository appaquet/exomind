use uuid::Uuid;

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_prefixed_id(prefix: &str) -> String {
    format!("{}{}", prefix, Uuid::new_v4().as_simple())
}

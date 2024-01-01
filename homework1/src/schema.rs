use jsonschema::{Draft, JSONSchema};
use once_cell::sync::Lazy;

pub static USER_REGISTER: Lazy<JSONSchema> = Lazy::new(|| {
    let schema = include_str!("schema/user_register.json");
    let schema: serde_json::Value = serde_json::from_str(schema).unwrap();
    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("A valid schema")
});

pub fn validate(payload: &serde_json::Value, schema: &JSONSchema) -> anyhow::Result<()> {
    if let Err(err) = schema.validate(payload) {
        let msg = err
            .map(|e| {
                if e.instance_path.to_string().is_empty() {
                    e.to_string()
                } else {
                    format!("{}: {}", e.instance_path, e)
                }
            })
            .collect::<Vec<String>>()
            .join("; ");
        anyhow::bail!(msg)
    } else {
        Ok(())
    }
}

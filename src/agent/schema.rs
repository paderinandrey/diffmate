use crate::error::Error;
use jsonschema::Validator;
use serde_json::Value;

const FULL: &str = include_str!("../../schemas/review.schema.json");

pub(crate) struct Schemas {
    pub cli_arg: String,
    validator: Validator,
}

impl Schemas {
    pub fn load() -> Result<Schemas, Error> {
        let mut schema: Value = serde_json::from_str(FULL)?;
        let validator = jsonschema::draft202012::new(&schema)
            .map_err(|error| Error::Schema(error.to_string()))?;
        if let Some(object) = schema.as_object_mut() {
            object.remove("$schema");
        }
        Ok(Schemas {
            cli_arg: serde_json::to_string(&schema)?,
            validator,
        })
    }

    pub fn check(&self, value: &Value) -> Result<(), String> {
        let errors: Vec<String> = self
            .validator
            .iter_errors(value)
            .take(3)
            .map(|error| error.to_string())
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

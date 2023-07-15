use eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
struct PostgrestErrorResBody {
    code: String,
    details: String,
    message: String,
}

impl Display for PostgrestErrorResBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(code: {}, details: {}, message: {})",
            self.code, self.details, self.message
        )
    }
}

pub fn parse<'a, T: Deserialize<'a>>(data: &'a str) -> Result<T> {
    // 1. Check response
    match serde_json::from_str::<T>(data) {
        Ok(res) => Ok(res),
        Err(_) => match serde_json::from_str::<PostgrestErrorResBody>(data) {
            Ok(error_res) => Err(error_res.into()),
            Err(_) => Err(eyre::eyre!("Unknown error. Data from postgrest {}", data)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct Fake {
        name: String,
    }

    #[test]
    fn error_23505_duplicate_key() {
        let data = r#"{"code":"23505","details":"Key (version)=(0.0.2) already exists.","hint":null,"message":"duplicate key value violates unique constraint \"version_deployments_version_key\""}"#;
        let res = parse::<Fake>(data);
        assert!(res.is_err());
        let res = res
            .unwrap_err()
            .downcast::<PostgrestErrorResBody>()
            .unwrap();
        assert_eq!(res.code, "23505");
        assert_eq!(res.details, "Key (version)=(0.0.2) already exists.");
        assert_eq!(
            res.message,
            "duplicate key value violates unique constraint \"version_deployments_version_key\""
        );
    }
}

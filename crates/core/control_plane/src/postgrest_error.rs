use eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
struct PostgrestErrorResBody {
    code: String,
    details: Option<String>,
    message: Option<String>,
}

impl Display for PostgrestErrorResBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(code: {}, details: {}, message: {})",
            self.code,
            self.details.clone().unwrap_or_default(),
            self.message.clone().unwrap_or_default()
        )
    }
}

pub fn parse<'a, T: Deserialize<'a>>(data: &'a str) -> Result<T> {
    // 1. Check response
    match serde_json::from_str::<T>(data) {
        Ok(out) => Ok(out),
        Err(err) => match serde_json::from_str::<PostgrestErrorResBody>(data) {
            Ok(error_res) => Err(error_res.into()),
            Err(_) => Err(eyre::eyre!(
                "unknown error: data from postgrest \"{}\", msg: \"{}\"",
                data,
                err
            )),
        },
    }
}

pub fn parse_once<'a, T: Deserialize<'a>>(data: &'a str) -> Result<Option<T>> {
    let mut out: Vec<T> = parse(data)?;
    Ok(out.pop())
}

pub fn parse_require_once<'a, T: Deserialize<'a>>(data: &'a str) -> Result<T> {
    match parse_once(data)? {
        Some(out) => Ok(out),
        None => Err(eyre::eyre!("no data found: {}", data)),
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
        assert_eq!(
            res.details,
            Some("Key (version)=(0.0.2) already exists.".to_string())
        );
        assert_eq!(
            res.message,
            Some("duplicate key value violates unique constraint \"version_deployments_version_key\"".to_string())
        );
    }
}

use crate::client::SupabaseClient;
use crate::postgrest_error::parse;
use chrono::{DateTime, Utc};
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};

#[derive(Serialize, Deserialize, IntoStaticStr, Display)]
#[strum(serialize_all = "lowercase")]
enum VersionDeploymentStatus {
    Scheduled,
    Running,
    Failed,
    Deployed,
    Finished,
}

#[derive(Serialize, Deserialize)]
struct CreateReqBody {
    version: String,
}

#[derive(Deserialize)]
struct VersionDeployment {
    id: u32,
    version: String,
    created_at: DateTime<Utc>,
    status: VersionDeploymentStatus,
    updated_at: DateTime<Utc>,
}

impl VersionDeployment {
    fn from(c: &SupabaseClient) -> postgrest::Builder {
        c.from("version_deployments")
    }

    pub async fn insert(c: &SupabaseClient, version: &str) -> Result<Self> {
        let c = Self::from(c);

        let req_body = CreateReqBody {
            version: version.to_string(),
        };

        let resp = c
            .insert(serde_json::to_string(&req_body).unwrap())
            .execute()
            .await?;

        let data = resp.text().await?;
        let mut data: Vec<Self> = parse(&data)?;
        if let Some(data) = data.pop() {
            Ok(data)
        } else {
            Err(eyre!("No data"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "local_supabase")]
    async fn list_runner() {
        color_eyre::install().unwrap();
        let config = config::Config::new("");
        let client = super::SupabaseClient::new(&config);
        println!("toto");
        VersionDeployment::insert(&client, "0.0.8").await.unwrap();
    }
}

use crate::client::SupabaseClient;
use crate::postgrest_error::{parse, parse_require_once};
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

#[derive(Debug, Serialize, Deserialize, AsRefStr, Display, PartialEq)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum VersionDeploymentStatus {
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

#[derive(Serialize, Deserialize)]
struct UpdateReqBody {
    status: VersionDeploymentStatus,
}

#[derive(Debug, Deserialize)]
pub struct VersionDeployment {
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
        parse_require_once(&resp.text().await?).wrap_err("insert version deployment")
    }

    pub async fn delete(c: &SupabaseClient, version: &str) -> Result<Vec<Self>> {
        let c = Self::from(c);
        let resp = c.delete().eq("version", version).execute().await?;
        parse(&resp.text().await?).wrap_err("delete version deployment")
    }

    pub async fn list_all(c: &SupabaseClient) -> Result<Vec<Self>> {
        let c = Self::from(c);
        let resp = c
            .select("*")
            .order_with_options::<&str, &str>("created_at", None, false, false)
            .execute()
            .await?;
        parse(&resp.text().await?).wrap_err("list all version deployments")
    }

    pub async fn update_status(
        c: &SupabaseClient,
        version: &str,
        status: VersionDeploymentStatus,
    ) -> Result<Self> {
        let c = Self::from(c);
        let req_body = UpdateReqBody { status };
        let resp = c
            .update(serde_json::to_string(&req_body)?)
            .eq("version", version)
            .neq("status", VersionDeploymentStatus::Failed.as_ref())
            .execute()
            .await?;
        parse_require_once(&resp.text().await?).wrap_err("update version deployment status")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "local_supabase")]
    async fn crud() {
        color_eyre::install().ok();
        const VERSION: &str = "0.0.1";

        let config = config::Config::new("");
        let client = SupabaseClient::new(&config);

        VersionDeployment::delete(&client, VERSION).await.ok();
        VersionDeployment::insert(&client, VERSION).await.unwrap();
        let list = VersionDeployment::list_all(&client).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].version, VERSION);
        assert_eq!(list[0].status, VersionDeploymentStatus::Scheduled);

        VersionDeployment::update_status(&client, VERSION, VersionDeploymentStatus::Failed)
            .await
            .unwrap();
        let list = VersionDeployment::list_all(&client).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].version, VERSION);
        assert_eq!(list[0].status, VersionDeploymentStatus::Failed);

        VersionDeployment::insert(&client, VERSION).await.unwrap();
        let list = VersionDeployment::list_all(&client).await.unwrap();
        assert_eq!(list.len(), 2);

        VersionDeployment::update_status(&client, VERSION, VersionDeploymentStatus::Deployed)
            .await
            .unwrap();

        VersionDeployment::delete(&client, VERSION).await.unwrap();
        let list = VersionDeployment::list_all(&client).await.unwrap();
        assert_eq!(list.len(), 0);
    }
}

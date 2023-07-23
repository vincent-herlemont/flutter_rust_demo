use config::Config;
use serde::{Deserialize, Serialize};

pub trait Client {}

pub struct SupabaseClient {
    pub config: Config,
}

#[derive(Debug, Serialize, Deserialize)]
struct PostgrestErrorResBody {
    code: String,
    details: String,
    message: String,
}

impl SupabaseClient {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn from(&self, table: &'static str) -> postgrest::Builder {
        let client =
            postgrest::Postgrest::new(format!("{}/rest/v1", self.config.get_supabase_url()));
        client
            .insert_header("apikey", &*self.config.get_supabase_anon_key())
            .insert_header("Content-Type", "application/json")
            .from(table)
            .auth(&self.config.get_supabase_service_role_key().unwrap())
    }

    // pub fn run<T>(c: postgrest::Builder) -> T {}
}

#[cfg(test)]
mod tests {
    use serial_test::file_parallel;

    #[tokio::test]
    #[file_parallel]
    #[cfg(feature = "local_supabase")]
    async fn list_runner() {
        let config = config::Config::new("");
        let client = super::SupabaseClient::new(&config);
        let res = client.from("runners").select("*").execute().await;
        println!("{:?}", res);
    }
}

use config::Config;

pub trait Client {}

pub struct SupbaseClient {
    pub config: Config,
}

impl SupbaseClient {
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
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    #[cfg(feature = "local_supabase")]
    async fn list_runner() {
        let config = config::Config::new("");
        let client = super::SupbaseClient::new(&config);
        let res = client.from("runners").select("*").execute().await;
        println!("{:?}", res);
    }
}

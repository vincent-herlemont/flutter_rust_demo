use crate::client::Client;

mod client;
mod postgrest_error;
mod resource;

pub struct ControlPlane<T: Client> {
    client: T,
}

#[cfg(test)]
mod tests {
    use postgrest::Postgrest;
    use serde_json::json;

    #[tokio::test]
    async fn test_supabase() {
        let supabase_url = "http://localhost:54321/rest/v1";
        let client = Postgrest::new(supabase_url);
        let api_key = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0";
        let secret_key = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImV4cCI6MTk4MzgxMjk5Nn0.EGIM96RAZx35lJzdJsyH-qQwv8Hdp7fsn3W0YpN81IU";

        // Test insert runner
        let res = client
            .insert_header("apikey", api_key)
            .insert_header("Content-Type", "application/json")
            .from("hub_info")
            .auth(secret_key)
            .select("*")
            .execute()
            .await;
        println!("{:?}", res);

        // let client = Postgrest::new(supabase_url);
        // let runner_uri = "http://localhost:8080";
        // let res = client
        //     .insert_header("apikey", api_key)
        //     .from("runners")
        //     .auth(secret_key)
        //     .insert(
        //         json!({
        //             "name": "local",
        //             "type": "hub",
        //             "uri": runner_uri,
        //         })
        //         .to_string(),
        //     )
        //     .execute()
        //     .await;
        // println!("{:?}", res);
        // println!("body: {:?}", res.unwrap().text().await.unwrap());
    }
}

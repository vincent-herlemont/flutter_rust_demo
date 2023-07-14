use dotenvy;
use std::env;
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Clone)]
pub enum Source {
    EnvVar,
    EnvFile,
    LocalSupabase,
}

#[derive(Debug, Clone)]
pub struct Config {
    source: Source,
    supabase_url: String,
    supabase_anon_key: String,
    supabase_service_role_key: Option<String>,
    runner_uri: Option<String>,
}

impl Config {
    pub fn get_source(&self) -> &Source {
        &self.source
    }

    pub fn get_supabase_url(&self) -> &str {
        &self.supabase_url
    }

    pub fn get_supabase_anon_key(&self) -> &str {
        &self.supabase_anon_key
    }

    pub fn get_supabase_service_role_key(&self) -> Option<&str> {
        self.supabase_service_role_key.as_deref()
    }

    pub fn get_runner_uri(&self) -> Option<&str> {
        self.runner_uri.as_deref()
    }
}

impl Config {
    pub fn new_from_dot_env(path: &Path) -> Config {
        let dotenvy = dotenvy::from_path(path);

        Config {
            source: dotenvy.ok().map_or(Source::EnvVar, |_| Source::EnvFile),
            supabase_url: env::var("SUPABASE_URL").expect("SUPABASE_URL is not set"),
            supabase_anon_key: env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY is not set"),
            supabase_service_role_key: env::var("SUPABASE_SERVICE_ROLE_KEY").ok(),
            runner_uri: env::var("RUNNER_URI").ok(),
        }
    }

    pub fn new_from_local() -> Config {
        // Run the command
        let output = Command::new("sh")
            .arg("-c")
            .arg("cd ../../../infra && supabase status")
            .output()
            .expect("Failed to execute command");

        // Convert output to string
        let output = String::from_utf8(output.stdout).unwrap();

        // Parse the output
        let lines: Vec<&str> = output.lines().collect();

        // Initialize variables
        let mut api_url = String::new();
        let mut anon_key = String::new();
        let mut service_role_key = String::new();

        // Loop over each line
        for line in lines {
            if line.contains("API URL") {
                api_url = line.split(": ").nth(1).unwrap().to_string();
            } else if line.contains("anon key") {
                anon_key = line.split(": ").nth(1).unwrap().to_string();
            } else if line.contains("service_role key") {
                service_role_key = line.split(": ").nth(1).unwrap().to_string();
            }
        }

        Config {
            source: Source::LocalSupabase,
            supabase_url: api_url,
            supabase_anon_key: anon_key,
            supabase_service_role_key: Some(service_role_key),
            runner_uri: None,
        }
    }

    pub fn new<P: AsRef<Path>>(_path: P) -> Self {
        let _path = _path.as_ref();
        #[cfg(feature = "local_supabase")]
        let config = Config::new_from_local();
        #[cfg(not(feature = "local_supabase"))]
        let config = Config::new_from_dot_env(_path);
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn new_from_env() {
        let config = Config::new_from_dot_env(&PathBuf::from(".env.test"));
        assert_eq!(config.source, Source::EnvFile);
        assert_eq!(config.supabase_url, "https://test.supabase.co");
        assert_eq!(config.supabase_anon_key, "anon_key_test");
        assert_eq!(
            config.supabase_service_role_key,
            Some("service_role_key_test".to_string())
        );
        assert_eq!(config.runner_uri, None);

        let config = Config::new_from_dot_env(PathBuf::from(".env.test.not.exit").as_path());
        assert_eq!(config.source, Source::EnvVar);
    }

    #[test]
    #[cfg(feature = "local_supabase")]
    fn new_from_local() {
        let config = Config::new_from_local();
        assert_eq!(config.source, Source::LocalSupabase);
    }

    #[test]
    #[cfg(feature = "local_supabase")]
    fn new_with_local_supabase() {
        let config = Config::new(PathBuf::from(".not_important").as_path());
        assert_eq!(config.source, Source::LocalSupabase);
    }

    #[test]
    #[cfg(not(feature = "local_supabase"))]
    fn new_with_env() {
        let config = Config::new(PathBuf::from(".env.test").as_path());
        assert_eq!(config.source, Source::EnvFile);
    }
}

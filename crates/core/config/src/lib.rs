use dotenvy;
use eyre::eyre;
use eyre::Result;
use std::env;
use std::path::Path;
use std::process::Command;
use url::Url;

#[derive(Debug, Clone, Default)]
pub struct Config {
    service_name: Option<&'static str>, // /!\ Not loaded from env variables but set by the service
    service_port: Option<u16>,
    hub_url: Option<Url>,
    supabase_url: Option<String>,
    supabase_anon_key: Option<String>,
    supabase_service_role_key: Option<String>,
    runner_uri: Option<String>,
    monitoring_loki_scheme: Option<String>,
    monitoring_loki_host_port: Option<String>,
}

impl Config {
    pub fn set_service_name(&mut self, service_name: &'static str) {
        self.service_name = Some(service_name);
    }

    pub fn get_service_name(&self) -> &str {
        self.service_name
            .as_deref()
            .expect("SERVICE_NAME is not set")
    }

    pub fn set_service_port(&mut self, service_port: u16) {
        self.service_port = Some(service_port);
    }

    pub fn get_hub_url(&self) -> Url {
        self.hub_url.clone().expect("HUB_URL is not set")
    }

    pub fn get_service_port(&self) -> u16 {
        self.service_port.expect("SERVICE_PORT is not set")
    }

    pub fn get_supabase_url(&self) -> &str {
        self.supabase_url
            .as_deref()
            .expect("SUPABASE_URL is not set")
    }

    pub fn get_supabase_anon_key(&self) -> &str {
        self.supabase_anon_key
            .as_deref()
            .expect("SUPABASE_ANON_KEY is not set")
    }

    pub fn get_supabase_service_role_key(&self) -> Option<&str> {
        self.supabase_service_role_key.as_deref()
    }

    pub fn get_runner_uri(&self) -> Option<&str> {
        self.runner_uri.as_deref()
    }

    pub fn get_monitoring_loki_url(&self) -> Result<Option<String>> {
        if let (Some(scheme), Some(host_port)) = (
            self.monitoring_loki_scheme.as_deref(),
            self.monitoring_loki_host_port.as_deref(),
        ) {
            if scheme == "http://" || scheme == "https://" {
                return Ok(Some(format!("{}{}", scheme, host_port)));
            } else {
                return Err(eyre!("Invalid scheme for MONITORING_LOKI_SCHEME"));
            }
        } else {
            Ok(None)
        }
    }

    fn merge(mut self, mut other: Config) -> Self {
        Self {
            service_port: self.service_port.take().or(other.service_port.take()),
            service_name: self.service_name.take().or(other.service_name.take()),
            hub_url: self.hub_url.take().or(other.hub_url.take()),
            supabase_url: self.supabase_url.take().or(other.supabase_url.take()),
            supabase_anon_key: self
                .supabase_anon_key
                .take()
                .or(other.supabase_anon_key.take()),
            supabase_service_role_key: self
                .supabase_service_role_key
                .take()
                .or(other.supabase_service_role_key.take()),
            runner_uri: self.runner_uri.take().or(other.runner_uri.take()),
            monitoring_loki_scheme: self
                .monitoring_loki_scheme
                .take()
                .or(other.monitoring_loki_scheme.take()),
            monitoring_loki_host_port: self
                .monitoring_loki_host_port
                .take()
                .or(other.monitoring_loki_host_port.take()),
        }
    }

    pub fn new_from_dot_env(path: &Path) -> Config {
        if let Err(err) = dotenvy::from_path(path) {
            eprintln!("failed to load {} file: {}", path.to_string_lossy(), err);
        }

        Config {
            service_port: env::var("SERVICE_PORT")
                .ok()
                .map(|p| p.parse::<u16>().expect("SERVICE_PORT must be a number")),
            service_name: None,
            hub_url: env::var("HUB_URL")
                .ok()
                .map(|url| Url::parse(&url).expect("HUB_URL must be a valid url")),
            supabase_url: env::var("SUPABASE_URL").ok(),
            supabase_anon_key: env::var("SUPABASE_ANON_KEY").ok(),
            supabase_service_role_key: env::var("SUPABASE_SERVICE_ROLE_KEY").ok(),
            runner_uri: env::var("RUNNER_URI").ok(),
            monitoring_loki_scheme: env::var("MONITORING_LOKI_SCHEME").ok(),
            monitoring_loki_host_port: env::var("MONITORING_LOKI_HOST_PORT").ok(),
        }
    }

    pub fn new_from_local() -> Config {
        println!("Try to get env via \"supabase status\" command");

        let mut parent_dir = env::current_dir().expect("Failed to get current execution directory");
        while parent_dir.file_name().unwrap() != "crates" {
            parent_dir = parent_dir.parent().unwrap().to_path_buf();
        }
        let cd_command: String = format!("cd {}/../infra", parent_dir.display());
        // Run the command
        let output = Command::new("sh")
            .arg("-c")
            // .arg("cd ../../../infra && supabase status")
            .arg(format!("{} && supabase status", cd_command))
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
            service_port: None,
            service_name: None,
            hub_url: None,
            supabase_url: Some(api_url),
            supabase_anon_key: Some(anon_key),
            supabase_service_role_key: Some(service_role_key),
            runner_uri: None,
            monitoring_loki_scheme: None,
            monitoring_loki_host_port: None,
        }
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        println!("Try to load env from {}", path.to_string_lossy());
        #[cfg(feature = "local_supabase")]
        let config_from_local = Config::new_from_local();
        let config = Config::new_from_dot_env(path);
        #[cfg(feature = "local_supabase")]
        let config = config_from_local.merge(config);
        config
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        env::remove_var("SUPABASE_URL");
        env::remove_var("SUPABASE_ANON_KEY");
        env::remove_var("SUPABASE_SERVICE_ROLE_KEY");
        env::remove_var("RUNNER_URI");
        env::remove_var("MONITORING_LOKI_SCHEME");
        env::remove_var("MONITORING_LOKI_HOST_PORT");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::file_serial;
    use std::path::PathBuf;
    use std::thread::sleep;

    #[test]
    #[file_serial]
    fn a_new_from_env_bad_file() {
        sleep(std::time::Duration::from_secs(1));
        let config = Config::new_from_dot_env(PathBuf::from(".env.test.not.exit").as_path());
        assert_eq!(config.supabase_url, None);
        assert_eq!(config.supabase_anon_key, None);
        assert_eq!(config.supabase_service_role_key, None);
        assert_eq!(config.runner_uri, None);
        assert_eq!(config.monitoring_loki_host_port, None)
    }

    #[test]
    #[file_serial]
    fn b_new_from_env() {
        let config = Config::new_from_dot_env(&PathBuf::from(".env.test"));
        assert_eq!(
            config.supabase_url,
            Some("https://test.supabase.co".to_string())
        );
        assert_eq!(config.supabase_anon_key, Some("anon_key_test".to_string()));
        assert_eq!(
            config.supabase_service_role_key,
            Some("service_role_key_test".to_string())
        );
        assert_eq!(config.runner_uri, None);
    }

    #[test]
    #[file_serial]
    fn b_new_from_env_loki() {
        let config = Config::new_from_dot_env(&PathBuf::from(".env.loki.test"));
        let url = config.get_monitoring_loki_url();
        dbg!(url);
    }

    #[test]
    #[file_serial]
    #[cfg(feature = "local_supabase")]
    fn c_new_from_local() {
        let config = Config::new_from_local();
        assert_eq!(
            config.supabase_url,
            Some("http://localhost:54321".to_string())
        );
    }

    #[test]
    #[file_serial]
    #[cfg(feature = "local_supabase")]
    fn d_new_with_local_supabase() {
        let config = Config::new(PathBuf::from(".not_important").as_path());
        assert_eq!(
            config.supabase_url,
            Some("http://localhost:54321".to_string())
        );
    }
}

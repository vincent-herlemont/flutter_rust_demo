use crate::Result;

pub struct Config {
    port: u16,
}

impl Config {
    pub fn new() -> Result<Self> {
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3030".to_string())
            .parse()?;
        Ok(Self { port })
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

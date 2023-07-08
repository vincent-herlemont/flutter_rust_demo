use hyper::client::HttpConnector;
use hyper::{Body, Client, Uri};

pub struct ClientHttp {
    uri: Uri,
    hyper: Client<HttpConnector, Body>,
}

impl ClientHttp {
    pub fn new() -> Self {
        Self {
            uri: Uri::from_static("https://api.supabase.co/rest/v1/"),
            hyper: Client::new(),
        }
    }
}

fn toto() {
    let _client_http = hyper::Client::new();
}

#[cfg(test)]
mod tests {
    use crate::toto;

    #[test]
    fn test_supabase() {
        toto()
        // let _client_http = hyper::Client::new();
        // let uri = Uri::from_static("https://api.supabase.co/rest/v1/");
        // dbg!(uri);
        // client_http.get(Uri::fro
    }
}

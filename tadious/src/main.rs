extern crate thrussh;
extern crate thrussh_keys;
extern crate futures;
extern crate tokio;
use std::sync::{Mutex, Arc};
use thrussh::*;
use thrussh::server::{Auth, Session};
use thrussh_keys::*;
use std::collections::HashMap;
use futures::Future;

// https://docs.rs/thrussh/0.33.5/thrussh/

#[tokio::main]
async fn main() {
    let client_key = thrussh_keys::key::KeyPair::generate_ed25519().unwrap();
    let client_pubkey = Arc::new(client_key.clone_public_key());
    let mut config = thrussh::server::Config::default();
    config.connection_timeout = Some(std::time::Duration::from_secs(3));
    config.auth_rejection_time = std::time::Duration::from_secs(3);
    config.keys.push(thrussh_keys::key::KeyPair::generate_ed25519().unwrap());

    if let Some(ip) = public_ip::addr().await {
        println!("public ip address: {:?}", ip);
    }
    println!("Hello, world!");

    let config = Arc::new(config);
    let sh = Server{
        client_pubkey,
        clients: Arc::new(Mutex::new(HashMap::new())),
        id: 0
    };

    tokio::time::timeout(
        std::time::Duration::from_secs(1),
        thrussh::server::run(config, "0.0.0.0:2222", sh)
    ).await.unwrap_or(Ok(()));
}

#[derive(Clone)]
struct Server {
    client_pubkey: Arc<thrussh_keys::key::PublicKey>,
    clients: Arc<Mutex<HashMap<(usize, ChannelId), thrussh::server::Handle>>>,
    id: usize,
}
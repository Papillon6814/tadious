extern crate thrussh;
extern crate thrussh_keys;
extern crate futures;
extern crate tokio;
use std::sync::{Mutex, Arc};
use thrussh::*;
use thrussh::server::{Auth, Session};
use thrussh_keys::*;
use std::collections::HashMap;

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

    let config = Arc::new(config);
    let sh = Server{
        client_pubkey,
        clients: Arc::new(Mutex::new(HashMap::new())),
        id: 0
    };

    // let res = tokio::time::timeout(
    //     std::time::Duration::from_secs(1),
    //     thrussh::server::run(config, "0.0.0.0:2222", sh)
    // ).await;

    thrussh::server::run(config, "0.0.0.0:22", sh).await;

    // match res {
    //     Ok(v) => println!("{:?}", v),
    //     Err(err) => println!("{}", err)
    // }
}

#[derive(Clone)]
struct Server {
    client_pubkey: Arc<thrussh_keys::key::PublicKey>,
    clients: Arc<Mutex<HashMap<(usize, ChannelId), thrussh::server::Handle>>>,
    id: usize,
}

impl server::Server for Server {
    type Handler = Self;
    fn new(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.id += 1;
        s
    }
}

impl server::Handler for Server {
    type Error = anyhow::Error;
    type FutureAuth = futures::future::Ready<Result<(Self, server::Auth), anyhow::Error>>;
    type FutureUnit = futures::future::Ready<Result<(Self, Session), anyhow::Error>>;
    type FutureBool = futures::future::Ready<Result<(Self, Session, bool), anyhow::Error>>;

    fn finished_auth(mut self, auth: Auth) -> Self::FutureAuth {
        futures::future::ready(Ok((self, auth)))
    }
    
    fn finished_bool(self, b: bool, s: Session) -> Self::FutureBool {
        futures::future::ready(Ok((self, s, b)))
    }

    fn finished(self, s: Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, s)))
    }

    fn channel_open_session(self, channel: ChannelId, session: Session) -> Self::FutureUnit {
        {
            let mut clients = self.clients.lock().unwrap();
            clients.insert((self.id, channel), session.handle());
        }
        self.finished(session)
    }

    fn auth_publickey(self, _: &str, _: &key::PublicKey) -> Self::FutureAuth {
        self.finished_auth(server::Auth::Accept)
    }

    fn data(self, channel: ChannelId, data: &[u8], mut session: Session) -> Self::FutureUnit {
        {
            let mut clients = self.clients.lock().unwrap();
            for ((id, channel), ref mut s) in clients.iter_mut() {
                if *id != self.id {
                    s.data(*channel, CryptoVec::from_slice(data));
                }
            }
        }
        session.data(channel, CryptoVec::from_slice(data));
        self.finished(session)
    }
}
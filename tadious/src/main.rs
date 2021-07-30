extern crate thrussh;
extern crate thrussh_keys;
extern crate futures;

#[tokio::main]
async fn main() {
    if let Some(ip) = public_ip::addr().await {
        println!("public ip address: {:?}", ip);
    }
    println!("Hello, world!");
}

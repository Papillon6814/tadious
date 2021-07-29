#[tokio::main]
async fn main() {
    if let Some(ip) = public_ip::addr().await {
        println!("public ip address: {:?}", ip);
    }
    println!("Hello, world!");
}

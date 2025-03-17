use craftping::tokio::ping;
use tokio::net::TcpStream;

#[tokio::main(flavor = "current_thread")]
async fn main() -> craftping::Result<()> {
    let servers = ["mc.hypixel.net"];
    for &server in servers.iter() {
        let mut stream = TcpStream::connect((server, 25565)).await?;
        let response = ping(&mut stream, server, 25565).await?;
        println!("ping to {}:", server);
        println!("{:?}", response);
    }
    Ok(())
}

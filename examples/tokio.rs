use craftping::tokio::ping;

#[tokio::main(flavor = "current_thread")]
async fn main() -> craftping::Result<()> {
    let servers = ["us.mineplex.com", "mc.hypixel.net"];
    for &server in servers.iter() {
        let response = ping(server, 25565).await?;
        println!("ping to {}:", server);
        println!("{:?}", response);
    }
    Ok(())
}

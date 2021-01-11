pub use craftping::sync::ping;

fn main() {
    let servers = ["us.mineplex.com", "mc.hypixel.net"];
    for &server in servers.iter() {
        let response = ping(server, 25565).unwrap();
        println!("ping to {}:", server);
        println!("{:?}", response);
    }
}

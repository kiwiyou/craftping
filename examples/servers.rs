fn main() {
    let servers = ["us.mineplex.com", "mc.hypixel.net"];
    for &server in servers.iter() {
        let response = craftping::ping(server, 25565).unwrap();
        println!("ping to {}:", server);
        println!("{:?}", response);
    }
}

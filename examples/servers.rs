use std::net::TcpStream;

pub use craftping::sync::ping;

fn main() {
    let servers = ["us.mineplex.com", "mc.hypixel.net"];
    for &server in servers.iter() {
        let mut stream = TcpStream::connect((server, 25565)).unwrap();
        let response = ping(&mut stream, server, 25565).unwrap();
        println!("ping to {}:", server);
        println!("{:?}", response);
    }
}

# craftping [![crates.io](https://img.shields.io/crates/v/craftping.svg)](https://crates.io/crates/craftping) [![docs.rs](https://docs.rs/craftping/badge.svg)](https://docs.rs/craftping) ![license](https://img.shields.io/github/license/kiwiyou/craftping.svg) [![Actively Maintained](https://img.shields.io/badge/Maintenance%20Level-Actively%20Maintained-green.svg)](https://github.com/kiwiyou/craftping)

craftping is a Rust library to ping Minecraft Servers.

## Usage

```toml
[dependencies]
craftping = "0.4.0"
```

You can synchronously ping to the server with `craftping::sync::ping`:

```rust
use std::net::TcpStream;
use craftping::sync::ping;

fn main() {
    let hostname = "localhost";
    let port = 25565;
    let mut stream = TcpStream::connect((hostname, port)).unwrap();
    let pong = ping(&mut stream, hostname, port).expect("Cannot ping server");
    println!("Ping result: {:?}", pong);
}
```

`sync` module requires `sync` feature, which is enabled by default.

If you want to send pings asynchronously, you can use `craftping::tokio::ping` or `craftping::futures::ping`:

- `craftping::tokio::ping`

```rust
use tokio::net::TcpStream;
use craftping::tokio::ping;

#[tokio::main]
async fn main() {
    let hostname = "localhost";
    let port = 25565;
    let mut stream = TcpStream::connect((hostname, port)).await.unwrap();
    let pong = ping(&mut stream, hostname, port).await.expect("Cannot ping server");
    println!("Ping result: {:?}", pong);
}
```

- `craftping::futures::ping`

```rust
use async_std::net::TcpStream;
use craftping::futures::ping;

#[async_std::main]
async fn main() {
    let hostname = "localhost";
    let port = 25565;
    let mut stream = TcpStream::connect((hostname, port)).await.unwrap();
    let pong = ping(&mut stream, hostname, port).await.expect("Cannot ping server");
    println!("Ping result: {:?}", pong);
}
```

Note that `tokio` module requires `async-tokio` feature and `futures` `async-futures`.

Check [here](https://wiki.vg/Server_List_Ping#Response) for more information about ping result.

## Contributing

Pull requests are welcome. For major issues, please open the issue on this repository first.

## License

[MIT](https://choosealicense.com/licenses/mit/)

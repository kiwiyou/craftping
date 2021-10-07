# craftping [![crates.io](https://img.shields.io/crates/v/craftping.svg)](https://crates.io/crates/craftping) [![docs.rs](https://docs.rs/craftping/badge.svg)](https://docs.rs/craftping) ![license](https://img.shields.io/github/license/kiwiyou/craftping.svg) [![Actively Maintained](https://img.shields.io/badge/Maintenance%20Level-Actively%20Maintained-green.svg)](https://github.com/kiwiyou/craftping)

craftping is a Rust library to ping Minecraft Servers.

## Usage

```toml
[dependencies]
craftping = "0.2.1"
```

You can synchronously ping to the server with `craftping::sync::ping`:

```rust
use craftping::sync::ping;

fn main() {
    let pong = ping("localhost", 25565).expect("Cannot ping server");
    println!("Ping result: {:?}", pong);
}
```

`sync` module requires `sync` feature, which is enabled by default.

If you want to send pings asynchronously, you can use `craftping::tokio::ping`:

```rust
use craftping::tokio::ping;

#[tokio::main]
fn main() {
    let pong = ping("localhost", 25565).await.expect("Cannot ping server");
    println!("Ping result: {:?}", pong);
}
```

Note that `tokio` module requires `async-tokio` feature.

Check [here](https://wiki.vg/Server_List_Ping#Response) for more information about ping result.

## Contributing

Pull requests are welcome. For major issues, please open the issue on this repository first.

## License

[MIT](https://choosealicense.com/licenses/mit/)

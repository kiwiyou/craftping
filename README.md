# craftping ![crates.io](https://img.shields.io/crates/v/craftping.svg) ![license](https://img.shields.io/github/license/kiwiyou/craftping.svg)

craftping is a Rust library to ping Minecraft Servers.

## Features
- Ping for servers **after version 1.6**

## Usage

```toml
[dependencies]
craftping = "0.1.0"
```
Note that craftping is **synchronous**, which means it will block until server responds.
```rust
use craftping::ping;

fn main() {
    let pong = ping("localhost", 25565).expect("Cannot ping server");
    println!("Ping result: {}", pong);
}
```
Check [here](https://wiki.vg/Server_List_Ping#Response) for more information about ping result.

## Contributing
Pull requests are welcome. For major issues, please open the issue on this repository first.

## License
[MIT](https://choosealicense.com/licenses/mit/)

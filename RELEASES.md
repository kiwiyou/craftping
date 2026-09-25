Version 0.8.0 (2026-09-26)
==========================

<a id="0.8.0-Libraries"></a>
Libraries
---------
- [Allow callers to set the protocol version](https://github.com/kiwiyou/craftping/commit/c9d66d3) sent in modern server pings. Use `PROTOCOL_VERSION_NOT_SET` (-1) when the version is unknown.
- [Declare Rust 1.85.1 as the minimum supported version](https://github.com/kiwiyou/craftping/commit/227adea).

<a id="0.8.0-Documentation"></a>
Documentation
-------------
- [Fix the broken Server List Ping wiki link](https://github.com/kiwiyou/craftping/commit/2e42101) in the README, contributed by SeongHoon Ryu (@ryush00).

<a id="0.8.0-Compatibility-Notes"></a>
Compatibility Notes
-------------------
- `sync::ping`, `tokio::ping`, and `futures::ping` now require a fourth `protocol_version: i32` argument. Pass `PROTOCOL_VERSION_NOT_SET` to retain the previous handshake value.

Version 0.7.0 (2025-03-18)
==========================

<a id="0.7.0-Libraries"></a>
Libraries
---------
- [Represent server descriptions as `serde_json::Value`](https://github.com/kiwiyou/craftping/commit/e60de58), accepting JSON description formats that the `Chat` type could not represent.

<a id="0.7.0-Compatibility-Notes"></a>
Compatibility Notes
-------------------
- `Response::description` is now `Option<serde_json::Value>` instead of `Option<Chat>`, and the `Chat` type has been removed. Callers that read chat fields or construct `Chat` values need to use JSON values instead.

Version 0.6.0 (2025-03-15)
==========================

<a id="0.6.0-Libraries"></a>
Libraries
---------
- [Allow server responses with no `description` field](https://github.com/kiwiyou/craftping/commit/23cd4d5). `Response::description` is now `Option<Chat>`.
- [Format `Chat` text and nested `extra` components in its `Debug` output](https://github.com/kiwiyou/craftping/commit/798f30b), removing newlines and repeated whitespace.
- [Update the crate to the Rust 2024 edition and refresh dependencies](https://github.com/kiwiyou/craftping/commit/8c5ff96).

<a id="0.6.0-Compatibility-Notes"></a>
Compatibility Notes
-------------------
- Callers must handle a missing `Response::description` and should not rely on `Chat`'s previous derived `Debug` output.

Version 0.5.0 (2024-01-23)
==========================

<a id="0.5.0-Libraries"></a>
Libraries
---------
- [Expose the optional `enforcesSecureChat` field](https://github.com/kiwiyou/craftping/commit/f58c3d9) as `Response::enforces_secure_chat`.
- [Expose the optional `previewsChat` field](https://github.com/kiwiyou/craftping/commit/a800713) as `Response::previews_chat`.
- [Update the crate to the Rust 2021 edition](https://github.com/kiwiyou/craftping/commit/2d6c67b).

Version 0.4.1 (2023-04-20)
==========================

<a id="0.4.1-Libraries"></a>
Libraries
---------
- [Return `UnsupportedProtocol` instead of panicking on an oversized VarInt shift](https://github.com/kiwiyou/craftping/commit/7cdfd35) in the synchronous, Tokio, and futures ping implementations.
- [Update favicon decoding to use the base64 0.21 API](https://github.com/kiwiyou/craftping/commit/0cc4d14).

Version 0.4.0 (2023-03-21)
==========================

<a id="0.4.0-Libraries"></a>
Libraries
---------
- [Add Serde serialization and deserialization to response entities](https://github.com/kiwiyou/craftping/commit/1c39374).
- [Expose the original server response bytes through `Response::raw()`](https://github.com/kiwiyou/craftping/commit/2ea9131) for both modern and legacy pings.
- [Read Forge's `fmlNetworkVersion` as an integer](https://github.com/kiwiyou/craftping/commit/1c39374).

<a id="0.4.0-Compatibility-Notes"></a>
Compatibility Notes
-------------------
- `Response` is marked `#[non_exhaustive]`, so downstream crates can no longer construct it with a struct literal or match every field without `..`.
- `ForgeData::fml_network_version` changed from `String` to `i32`.

Version 0.3.1 (2022-10-27)
==========================

<a id="0.3.1-Libraries"></a>
Libraries
---------
- [Return an error instead of panicking when a favicon value is too short](https://github.com/kiwiyou/craftping/commit/c80d79e).
- [Implement `Clone` for `Response` and its response entity types](https://github.com/kiwiyou/craftping/commit/fe0768e).

Version 0.3.0 (2021-10-07)
==========================

<a id="0.3.0-Libraries"></a>
Libraries
---------
- [Add `craftping::futures::ping`](https://github.com/kiwiyou/craftping/commit/3797cd1) for streams implementing the futures I/O traits, enabled by `async-futures`.
- [Accept caller supplied streams](https://github.com/kiwiyou/craftping/commit/3797cd1) in the synchronous and Tokio ping functions, allowing callers to control the connection.

<a id="0.3.0-Compatibility-Notes"></a>
Compatibility Notes
-------------------
- `sync::ping` and `tokio::ping` now take `&mut stream, hostname, port`; callers must open the connection before pinging.

Version 0.2.0 (2021-01-16)
==========================

<a id="0.2.0-Libraries"></a>
Libraries
---------
- [Rewrite ping responses as typed `Response` values](https://github.com/kiwiyou/craftping/commit/f24b8ce), including version, player, description, favicon, and Forge data.
- [Add fallback parsing for legacy server pings](https://github.com/kiwiyou/craftping/commit/f24b8ce).
- [Add a Tokio ping function](https://github.com/kiwiyou/craftping/commit/e403a76) behind the `async-tokio` feature; synchronous ping is enabled by default with `sync`.
- [Decode legacy response text as UTF-16](https://github.com/kiwiyou/craftping/commit/4046f08).

<a id="0.2.0-Compatibility-Notes"></a>
Compatibility Notes
-------------------
- The root `craftping::ping` function moved to `craftping::sync::ping` and returns `Response` instead of a JSON string. Errors use the crate's `Error` type.

Version 0.1.0 (2019-03-23)
==========================

<a id="0.1.0-Libraries"></a>
Libraries
---------
- [Initial release](https://github.com/kiwiyou/craftping/releases/tag/0.1.0): synchronous Minecraft server list ping for servers after version 1.6, returning the response as a JSON string.

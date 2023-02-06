# static-graph

[![Crates.io][crates-badge]][crates-url]
[![License][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/static-graph.svg
[crates-url]: https://crates.io/crates/static-graph
[license-badge]: https://img.shields.io/crates/l/static-graph.svg
[license-url]: LICENSE-MIT
[actions-badge]: https://github.com/volo-rs/static-graph/actions/workflows/ci.yaml/badge.svg
[actions-url]: https://github.com/volo-rs/static-graph/actions

This crate provides the ability to generate static graphs by analysing the node dependencies in DSL. It allows only one input and one output in a graph, and independent nodes can run in parallel.

## Usage

Add this to your `Cargo.toml`:

```toml
[build-dependencies]
static-graph = "0.1"
```

## Example

Write a graph description in a `.graph` file:

```txt
node E -> (X, Y) {

}

node X -> O {

}

node Y -> O {
    
}

node O {

}

graph G(E)
```

Then, in `build.rs`:

```rust
fn main() {
    static_graph::configure().compile(".graph").unwrap();
}
```

Finally, in `main.rs` write your own logic.

## License

Volo is dual-licensed under the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](https://github.com/volo-rs/.github/blob/main/LICENSE-MIT) and [LICENSE-APACHE](https://github.com/volo-rs/.github/blob/main/LICENSE-APACHE) for details.

## Community

- Email: [volo@cloudwego.io](mailto:volo@cloudwego.io)
- How to become a member: [COMMUNITY MEMBERSHIP](https://github.com/cloudwego/community/blob/main/COMMUNITY_MEMBERSHIP.md)
- Issues: [Issues](https://github.com/volo-rs/.github/issues)
- Feishu: Scan the QR code below with [Feishu](https://www.feishu.cn/) or [click this link](https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=7f0oe1a4-930f-41f9-808a-03b89a681020) to join our CloudWeGo Volo user group.

  <img src="https://github.com/cloudwego/volo/raw/main/.github/assets/volo-feishu-user-group.png" alt="Volo user group" width="50%" height="50%" />

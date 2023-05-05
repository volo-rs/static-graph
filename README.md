# static-graph

[![Crates.io][crates-badge]][crates-url]
[![License][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/static-graph.svg
[crates-url]: https://crates.io/crates/static-graph
[license-badge]: https://img.shields.io/crates/l/static-graph.svg
[license-url]: #license
[actions-badge]: https://github.com/volo-rs/static-graph/actions/workflows/ci.yaml/badge.svg
[actions-url]: https://github.com/volo-rs/static-graph/actions

This crate provides the ability to generate static graphs by analysing the node dependencies in DSL. It allows only one input and one output in a graph, and independent nodes can run in maximum parallel.

For example, in the following graph(the number represents the execution time of the node), run it in serial will take 6 seconds, but run it in maximum parallel will just take 2 seconds.

```mermaid
graph TD;
    A/0-->B/1;
    A/0-->C/2;
    A/0-->D/1;
    A/0-->E/1;
    B/1-->F/0;
    C/2-->F/0;
    D/1-->G/1;
    E/1-->G/1;
    F/0-->H/0;
    G/1-->H/0;
```

## Usage

Add this to your `Cargo.toml`:

```toml
[build-dependencies]
static-graph = "0.2"
```

## Example

Write a graph description in `example.graph` file:

```txt
node E -> (X, Y) {
    #[default = "crate::Custom::new"]
    custom: crate::Custom,
}

node X -> O {
    x: list<string>,
}

node Y -> O {
    y: map<i32, string>,
}

node O {
    #[editable = "true"]
    o: string,
}

graph G(E)
```

Then, in `build.rs`:

```rust
fn main() {
    static_graph::configure().compile("example.graph").unwrap();
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

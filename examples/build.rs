fn main() {
    static_graph::configure()
        .file_name("example.rs")
        .enable_mermaid(true)
        .compile("./graphs/example.graph")
        .unwrap();
    static_graph::configure()
        .file_name("parallel.rs")
        .enable_mermaid(true)
        .compile("./graphs/parallel.graph")
        .unwrap();
}

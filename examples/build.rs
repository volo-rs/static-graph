fn main() {
    static_graph::configure()
        .file_name("example.rs")
        .compile("./graphs/example.graph")
        .unwrap();
    static_graph::configure()
        .file_name("parallel.rs")
        .compile("./graphs/parallel.graph")
        .unwrap();
}

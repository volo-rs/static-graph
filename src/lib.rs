//! This crate provides the ability to generate static graphs by analysing the node dependencies in DSL. It allows only one input and one output in a graph, and independent nodes can run in maximum parallel.

//! For example, in the following graph(the number represents the execution time of the node), run it in serial will take 6 seconds, but run it in maximum parallel will just take 2 seconds.

//! ```mermaid
//! graph TD;
//!     A/0-->B/1;
//!     A/0-->C/2;
//!     A/0-->D/1;
//!     A/0-->E/1;
//!     B/1-->F/0;
//!     C/2-->F/0;
//!     D/1-->G/1;
//!     E/1-->G/1;
//!     F/0-->H/0;
//!     G/1-->H/0;
//! ```

//! ## Usage

//! Add this to your `Cargo.toml`:

//! ```toml
//! [build-dependencies]
//! static-graph = "0.2"
//! ```

//! ## Example

//! Write a graph description in `example.graph` file:

//! ```txt
//! node E -> (X, Y) {
//!     #[default = "crate::Custom::new"]
//!     custom: crate::Custom,
//! }

//! node X -> O {
//!     x: list<string>,
//! }

//! node Y -> O {
//!     y: map<i32, string>,
//! }

//! node O {
//!     #[editable = "true"]
//!     o: string,
//! }

//! graph G(E)
//! ```

//! Then, in `build.rs`:

//! ```rust, ignore
//! static_graph::configure()
//!     .file_name("example.rs")
//!     .compile("example.graph")
//!     .unwrap();
//! ```

//! Finally, in `main.rs` write your own logic for your nodes in the graph. The generated code will be in the `OUT_DIR` directory by default, the graph name is `G`, and the nodes name are `E`, `X`, `Y`, `O`. You should implement the `Runnable` trait for each node, and then you can automatically run the graph in maximum parallel by calling `G::new().run()`.

//! ```rust, ignore
//! use std::{
//!     sync::Arc,
//!     time::{Duration, Instant},
//! };

//! use gen_graph::{Runnable, E, G, O, X, Y};

//! #[allow(warnings, clippy::all)]
//! pub mod gen_graph {
//!     static_graph::include_graph!("example.rs");
//! }

//! #[derive(Default)]
//! pub struct Custom;

//! impl Custom {
//!     pub fn new() -> Self {
//!         Self
//!     }
//! }

//! #[tokio::main]
//! async fn main() {
//!     let start = Instant::now();
//!     let resp = G::new()
//!         .run::<Request, EResponse, XResponse, YResponse, OResponse, ()>(Request {
//!             msg: "**Hello, world!**".to_string(),
//!             user_age: 18,
//!         })
//!         .await;
//!     let duration = start.elapsed();

//!     println!("Time elapsed is {duration:?}, resp is {resp:?}");
//! }

//! #[derive(Clone)]
//! pub struct Request {
//!     msg: String,
//!     user_age: u8,
//! }

//! #[derive(Clone)]
//! pub struct EResponse(Duration);

//! //! impl Runnable<Request, ()> for E {
//!     type Resp = EResponse;
//!     type Error = ();

//!     async fn run(&self, _req: Request, _prev_resp: ()) -> Result<Self::Resp, Self::Error> {
//!         tokio::time::sleep(Duration::from_secs(1)).await;
//!         Ok(EResponse(Duration::from_secs(1)))
//!     }
//! }

//! #[derive(Clone)]
//! pub struct XResponse(bool);

//! //! impl Runnable<Request, EResponse> for X {
//!     type Resp = XResponse;
//!     type Error = ();

//!     async fn run(&self, req: Request, prev_resp: EResponse) -> Result<Self::Resp, Self::Error> {
//!         tokio::time::sleep(prev_resp.0).await;
//!         Ok(XResponse(!req.msg.contains('*')))
//!     }
//! }

//! #[derive(Clone)]
//! pub struct YResponse(bool);

//! //! impl Runnable<Request, EResponse> for Y {
//!     type Resp = YResponse;
//!     type Error = ();

//!     async fn run(&self, req: Request, prev_resp: EResponse) -> Result<Self::Resp, Self::Error> {
//!         tokio::time::sleep(prev_resp.0).await;
//!         Ok(YResponse(req.user_age >= 18))
//!     }
//! }

//! #[derive(Clone, Debug)]
//! pub struct OResponse(String);

//! //! impl Runnable<Request, (XResponse, YResponse)> for O {
//!     type Resp = OResponse;
//!     type Error = ();

//!     async fn run(
//!         &self,
//!         req: Request,
//!         prev_resp: (XResponse, YResponse),
//!     ) -> Result<Self::Resp, Self::Error> {
//!         self.o.store(Arc::new(req.msg.clone()));
//!         println!("O: {:#?}", self.o.load());
//!         if prev_resp.0 .0 && prev_resp.1 .0 {
//!             Ok(OResponse(req.msg))
//!         } else {
//!             Ok(OResponse("Ban".to_string()))
//!         }
//!     }
//! }
//! ```
//!
pub mod codegen;
pub mod context;
pub mod index;
pub mod parser;
pub mod resolver;
pub mod symbol;
pub mod tags;

pub use arc_swap::*;
pub use tokio::*;

use crate::{
    codegen::Codegen,
    context::Context,
    parser::{document::Document, Parser},
    resolver::{ResolveResult, Resolver},
};

use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{exit, Command},
};

#[macro_export]
macro_rules! include_graph {
    ($graph: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $graph)));
    };
}

#[must_use]
pub fn configure() -> Builder {
    Builder {
        emit_rerun_if_changed: std::env::var_os("CARGO").is_some(),
        out_dir: None,
        file_name: "gen_graph.rs".into(),
        enable_mermaid: false,
    }
}

#[derive(Debug, Clone)]
pub struct Builder {
    emit_rerun_if_changed: bool,
    out_dir: Option<PathBuf>,
    file_name: PathBuf,
    enable_mermaid: bool, // generate mermaid file
}

impl Builder {
    #[must_use]
    pub fn out_dir(mut self, out_dir: impl AsRef<Path>) -> Self {
        self.out_dir = Some(out_dir.as_ref().to_path_buf());
        self
    }

    #[must_use]
    pub fn file_name(mut self, file_name: impl AsRef<Path>) -> Self {
        self.file_name = file_name.as_ref().to_path_buf();
        self
    }

    #[must_use]
    pub fn emit_rerun_if_changed(mut self, enable: bool) -> Self {
        self.emit_rerun_if_changed = enable;
        self
    }

    #[must_use]
    pub fn enable_mermaid(mut self, enable: bool) -> Self {
        self.enable_mermaid = enable;
        self
    }

    pub fn compile(self, graph: impl AsRef<Path>) -> std::io::Result<()> {
        let out_dir = if let Some(out_dir) = self.out_dir.as_ref() {
            out_dir.clone()
        } else {
            PathBuf::from(std::env::var("OUT_DIR").unwrap())
        };

        if self.emit_rerun_if_changed {
            println!("cargo:rerun-if-changed={}", graph.as_ref().display());
        }

        let input = unsafe { String::from_utf8_unchecked(std::fs::read(graph).unwrap()) };

        let document = Document::parse(&input).unwrap().1;
        let ResolveResult {
            graphs,
            nodes,
            fields,
            tags,
            entrys,
        } = Resolver::default().resolve_document(document);

        let mut cx = Context::new();
        cx.set_graphs(graphs);
        cx.set_nodes(nodes);
        cx.set_fields(fields);
        cx.set_tags(tags);

        let mut cg = Codegen::new(cx);

        if self.enable_mermaid {
            let ret = cg.mermaid(&entrys);
            let mut name = self.file_name.file_stem().unwrap().to_os_string();
            name.push(".mermaid");
            let out = out_dir.join(name);
            let mut file = std::io::BufWriter::new(std::fs::File::create(&out).unwrap());
            file.write_all(ret.trim().as_bytes()).unwrap();
            file.flush().unwrap();
        }
        let stream = cg.write_document(entrys);
        let out = out_dir.join(self.file_name);
        let mut file = std::io::BufWriter::new(std::fs::File::create(&out).unwrap());
        file.write_all(stream.to_string().as_bytes()).unwrap();
        file.flush().unwrap();
        fmt_file(out);

        Ok(())
    }
}

fn fmt_file<P: AsRef<Path>>(file: P) {
    let file = file.as_ref();
    if let Some(a) = file.extension() {
        if a != "rs" {
            return;
        }
    };

    let result = Command::new(std::env::var("RUSTFMT").unwrap_or_else(|_| "rustfmt".to_owned()))
        .arg("--emit")
        .arg("files")
        .arg("--edition")
        .arg("2021")
        .arg(file)
        .output();

    match result {
        Err(e) => eprintln!("{e:?}",),
        Ok(output) => {
            if !output.status.success() {
                std::io::stderr().write_all(&output.stderr).unwrap();
                exit(output.status.code().unwrap_or(1))
            }
        }
    }
}

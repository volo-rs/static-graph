pub mod codegen;
pub mod context;
pub mod index;
pub mod parser;
pub mod resolver;
pub mod symbol;
pub mod tags;

pub use arc_swap::*;
pub use async_trait::*;
pub use tokio::*;

use crate::{
    codegen::Codegen,
    context::Context,
    parser::{document::Document, Parser},
    resolver::{ResolveResult, Resolver},
};
use std::{
    io::{self, Write},
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
    }
}

#[derive(Debug, Clone)]
pub struct Builder {
    emit_rerun_if_changed: bool,
    out_dir: Option<PathBuf>,
    file_name: PathBuf,
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

    pub fn compile(self, graph: impl AsRef<Path>) -> io::Result<()> {
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

use std::time::{Duration, Instant};

use gen_graph::{Runnable, E, G, O, Q, R, W, X, Y, Z};

#[allow(warnings, clippy::all)]
pub mod gen_graph {
    static_graph::include_graph!("parallel.rs");
}

#[derive(Default)]
pub struct Custom;

impl Custom {
    pub fn new() -> Self {
        Self
    }
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let resp = G::new()
        .run::<Request, EResponse, XResponse, YResponse, WResponse, ZResponse, QResponse, RResponse, OResponse, ()>(
            Request
        )
        .await;
    let duration = start.elapsed();

    println!("Time elapsed is {duration:?}, resp is {resp:?}");
}

#[derive(Clone)]
pub struct Request;

#[derive(Clone)]
pub struct EResponse;

impl Runnable<Request, ()> for E {
    type Resp = EResponse;
    type Error = ();

    async fn run(&self, _req: Request, _prev_resp: ()) -> Result<Self::Resp, Self::Error> {
        Ok(EResponse)
    }
}

#[derive(Clone)]
pub struct XResponse;

impl Runnable<Request, EResponse> for X {
    type Resp = XResponse;
    type Error = ();

    async fn run(&self, _req: Request, _prev_resp: EResponse) -> Result<Self::Resp, Self::Error> {
        tokio::time::sleep(Duration::from_secs(2)).await;
        Ok(XResponse)
    }
}

#[derive(Clone)]
pub struct YResponse;

impl Runnable<Request, EResponse> for Y {
    type Resp = YResponse;
    type Error = ();

    async fn run(&self, _req: Request, _prev_resp: EResponse) -> Result<Self::Resp, Self::Error> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(YResponse)
    }
}

#[derive(Clone)]
pub struct WResponse;

impl Runnable<Request, EResponse> for W {
    type Resp = WResponse;
    type Error = ();

    async fn run(&self, _req: Request, _prev_resp: EResponse) -> Result<Self::Resp, Self::Error> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(WResponse)
    }
}
#[derive(Clone)]
pub struct ZResponse;

impl Runnable<Request, EResponse> for Z {
    type Resp = ZResponse;
    type Error = ();

    async fn run(&self, _req: Request, _prev_resp: EResponse) -> Result<Self::Resp, Self::Error> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(ZResponse)
    }
}
#[derive(Clone)]
pub struct QResponse;

impl Runnable<Request, (XResponse, YResponse)> for Q {
    type Resp = QResponse;
    type Error = ();

    async fn run(
        &self,
        _req: Request,
        _prev_resp: (XResponse, YResponse),
    ) -> Result<Self::Resp, Self::Error> {
        Ok(QResponse)
    }
}

#[derive(Clone)]
pub struct RResponse;

impl Runnable<Request, (WResponse, ZResponse)> for R {
    type Resp = RResponse;
    type Error = ();

    async fn run(
        &self,
        _req: Request,
        _prev_resp: (WResponse, ZResponse),
    ) -> Result<Self::Resp, Self::Error> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(RResponse)
    }
}

#[derive(Clone, Debug)]
pub struct OResponse;

impl Runnable<Request, (QResponse, RResponse)> for O {
    type Resp = OResponse;
    type Error = ();

    async fn run(
        &self,
        _req: Request,
        _prev_resp: (QResponse, RResponse),
    ) -> Result<Self::Resp, Self::Error> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(OResponse)
    }
}

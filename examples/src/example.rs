use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use gen_graph::{Runnable, E, G, O, X, Y};

#[allow(warnings, clippy::all)]
pub mod gen_graph {
    static_graph::include_graph!("example.rs");
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
        .run::<Request, EResponse, XResponse, YResponse, OResponse, ()>(Request {
            msg: "**Hello, world!**".to_string(),
            user_age: 18,
        })
        .await;
    let duration = start.elapsed();

    println!("Time elapsed is {duration:?}, resp is {resp:?}");
}

#[derive(Clone)]
pub struct Request {
    msg: String,
    user_age: u8,
}

#[derive(Clone)]
pub struct EResponse(Duration);

impl Runnable<Request, ()> for E {
    type Resp = EResponse;
    type Error = ();

    async fn run(&self, _req: Request, _prev_resp: ()) -> Result<Self::Resp, Self::Error> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(EResponse(Duration::from_secs(1)))
    }
}

#[derive(Clone)]
pub struct XResponse(bool);

impl Runnable<Request, EResponse> for X {
    type Resp = XResponse;
    type Error = ();

    async fn run(&self, req: Request, prev_resp: EResponse) -> Result<Self::Resp, Self::Error> {
        tokio::time::sleep(prev_resp.0).await;
        Ok(XResponse(!req.msg.contains('*')))
    }
}

#[derive(Clone)]
pub struct YResponse(bool);

impl Runnable<Request, EResponse> for Y {
    type Resp = YResponse;
    type Error = ();

    async fn run(&self, req: Request, prev_resp: EResponse) -> Result<Self::Resp, Self::Error> {
        tokio::time::sleep(prev_resp.0).await;
        Ok(YResponse(req.user_age >= 18))
    }
}

#[derive(Clone, Debug)]
pub struct OResponse(String);

impl Runnable<Request, (XResponse, YResponse)> for O {
    type Resp = OResponse;
    type Error = ();

    async fn run(
        &self,
        req: Request,
        prev_resp: (XResponse, YResponse),
    ) -> Result<Self::Resp, Self::Error> {
        self.o.store(Arc::new(req.msg.clone()));
        println!("O: {:#?}", self.o.load());
        if prev_resp.0 .0 && prev_resp.1 .0 {
            Ok(OResponse(req.msg))
        } else {
            Ok(OResponse("Ban".to_string()))
        }
    }
}

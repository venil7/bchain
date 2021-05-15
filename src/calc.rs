use tonic::{Request, Response, Status};

pub mod hello_world {
  tonic::include_proto!("calculator");
}
use hello_world::calculator_server::Calculator;
use hello_world::{CalcReq, CalcResp};

#[derive(Debug, Default)]
pub struct Calc {}

#[tonic::async_trait]
impl Calculator for Calc {
  async fn calculate(&self, req: Request<CalcReq>) -> Result<Response<CalcResp>, Status> {
    let ab = req.into_inner();
    let reply = CalcResp {
      op1: ab.op1 + ab.op2,
    };
    Ok(Response::new(reply))
  }
}

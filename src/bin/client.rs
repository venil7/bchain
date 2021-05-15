use bchain::calc::hello_world::calculator_client::CalculatorClient;
use bchain::calc::hello_world::{CalcReq, Op};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut client = CalculatorClient::connect("http://[::1]:50051").await?;

  let [op1, op2] = [12, 13];

  let request = tonic::Request::new(CalcReq {
    op1: op1,
    op2: op2,
    op: Op::Add as i32,
  });

  let response = client.calculate(request).await?;

  println!("{}+{} = {}", op1, op2, response.into_inner().op1);

  Ok(())
}

use bchain::calc::hello_world::calculator_server::CalculatorServer;
use bchain::calc::Calc;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let addr = "[::1]:50051".parse()?;
  let calculator = Calc::default();

  Server::builder()
    .add_service(CalculatorServer::new(calculator))
    .serve(addr)
    .await?;

  Ok(())
}

use bchain::cli::Cli;
use bchain::error::AppError;
use bchain::network::full_node::FullNode;
use bchain::result::AppResult;
use structopt::StructOpt;

#[async_std::main]
async fn main() -> AppResult<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let cli = Cli::from_args();
    let addr = cli.listen.parse()?;

    let server_handle = tokio::spawn(async move {
        FullNode::run(addr).await?;
        Ok::<(), AppError>(())
    });

    server_handle.await??;

    Ok(())
}

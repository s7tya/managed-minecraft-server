use agent::minecraft::server;

fn main() -> anyhow::Result<()> {
    server::Server::serve()?;
    Ok(())
}

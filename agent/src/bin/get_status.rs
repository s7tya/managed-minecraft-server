use std::env;

use agent::minecraft;

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    if args.is_empty() || 2 < args.len() {
        println!("Usage: [HOST] (PORT)");
        return Ok(());
    }

    let host = args[0].as_str();
    let port: u16 = if args.len() == 2 {
        args[1].parse()?
    } else {
        25565
    };

    let mut client = minecraft::client::Client::new(host, port)?;
    let status = client.status()?;
    println!("{:#?}", status);

    Ok(())
}

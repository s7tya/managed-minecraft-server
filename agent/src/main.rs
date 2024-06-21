use minecraft::client;

pub mod minecraft;

fn main() -> anyhow::Result<()> {
    let mut client = client::Client::new("mofucraft.net", 25565)?;
    let count = client.get_online_players_count()?;
    println!("{}", count);
    Ok(())
}

use minecraft::get_online_players_count;

pub mod minecraft;

fn main() -> anyhow::Result<()> {
    get_online_players_count("dan5.red", 25565)?;

    Ok(())
}

use anyhow::Result;
use std::{thread, time::Duration};

use agent::minecraft::client;

fn main() -> Result<()> {
    let servers: Vec<(&str, u16)> = vec![("localhost", 25565)];

    let mut unused_count = vec![0_usize; servers.len()];

    loop {
        let handles: Vec<_> = servers
            .iter()
            .map(|&(host, port)| {
                thread::spawn(move || -> Result<usize> {
                    let mut client = client::Client::new(host, port)?;
                    let count = client.get_online_players_count()?;
                    Ok(count)
                })
            })
            .collect();

        for (i, handle) in handles.into_iter().enumerate() {
            let count = handle.join().unwrap().unwrap_or(0);

            if count == 0 {
                unused_count[i] += 1;

                if unused_count[i] > 3 {
                    println!("Should shutdown {}:{}", servers[i].0, servers[i].1);
                }
            }
            println!("{}:{} -> {}", servers[i].0, servers[i].1, unused_count[i]);
        }

        thread::sleep(Duration::from_secs(5));
    }
}

use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use failure::Error;

type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let f = File::open("data/day01/input")?;
    let br = BufReader::new(f);

    let masses: Vec<u32> = br
        .lines()
        .map(|l| l?.parse().map_err(|e: std::num::ParseIntError| e.into()))
        .collect::<Result<Vec<u32>>>()?;

    let fuel: u32 = masses.iter().map(|m| (m / 3) - 2).sum();

    println!("Fuel needed: {}", fuel);

    Ok(())
}

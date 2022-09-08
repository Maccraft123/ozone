pub mod basic;
use crate::basic::{
    init,
    Config,
};

use anyhow::Result;

fn main() -> Result<()> {
    let conf = Config::new();
    init(&conf)?;
    println!("Hello world!");
    std::thread::sleep(std::time::Duration::from_secs(5));
    Ok(())
}

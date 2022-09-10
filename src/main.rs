pub mod basic;
use crate::basic::{
    init,
    Config,
};

use anyhow::Result;

fn main() -> Result<()> {
    let conf = Config::new().stdio(true).mount_boot(true).mount_sys(true);
    init(&conf)?;
    println!("Hello world!");
    loop {}
}

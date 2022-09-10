pub mod basic;
use crate::basic::{
    init,
    Config,
};

use anyhow::Result;

fn main() -> Result<()> {
    let conf = Config::new().mount_boot(true).mount_sys(true);
    println!("Hello world!");
    init(&conf)?;
    println!("Hello world!");
    loop {}
}

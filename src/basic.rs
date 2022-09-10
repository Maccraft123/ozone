use nix::{
    fcntl::{
        open,
        OFlag,
    },
    unistd::{
        dup2,
        close,
        mkdir,
    },
    sys::{
        stat::Mode,
    },
    mount::{
        MsFlags,
        mount,
    },
};
use anyhow::Result;
use std::path::PathBuf;

pub struct Config {
    stdio: bool,
    mount_boot: bool,
    mount_sys: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            stdio: true,
            mount_boot: false,
            mount_sys: true,
        }
    }
    pub fn stdio(mut self, arg: bool) -> Self {
        self.stdio = arg;
        self
    }
    pub fn mount_boot(mut self, arg: bool) -> Self {
        self.mount_boot = arg;
        self
    }
    pub fn mount_sys(mut self, arg: bool) -> Self {
        self.mount_sys = arg;
        self
    }
}

fn debug_dir(dir: &str) -> Result<()> {
    let paths = std::fs::read_dir(dir).unwrap();
    println!("{}", dir);
    for path in paths {
        println!("{:#?}", path);
    }
    Ok(())
}

pub fn init(config: &Config) -> Result<()> {
    // TODO: last time i tried it paniced for some reason on getpid()
    //if std::env::args().collect::<Vec<String>>().get(0).as_deref() != Some(&"/init".to_string()) {
    //    return Ok(())
    //}

    mount(Some("none"), "/dev", Some("devtmpfs"), MsFlags::empty(), Some(""))?;

    if config.stdio {
        // open up what-will-be stdin and stdout
        let input = open("/dev/console", OFlag::O_RDONLY, Mode::empty())?;
        let output = open("/dev/console", OFlag::O_WRONLY, Mode::empty())?;

        // and wire them up
        dup2(input, 0)?;
        dup2(output, 1)?;
        dup2(output, 2)?;

        if input > 2 {
            close(input)?;
        }
        if output > 2 {
            close(output)?;
        }
    }

    if config.mount_sys {
        mkdir("/sys/", Mode::from_bits(0o777).unwrap())?;
        mount(Some("none"), "/sys", Some("sysfs"), MsFlags::empty(), Some(""))?;

        let has_efi = PathBuf::from("/sys/firmware/efi").exists();
        if has_efi {
            mount(Some("efivarfs"), "/sys/firmware/efi/efivars", Some("efivarfs"), MsFlags::empty(), Some(""))?;
        }
    }

    if config.mount_boot {
        mkdir("/boot", Mode::from_bits(0o777).unwrap())?;
        //mount(Some("none"), "/sys", Some("sysfs"), MsFlags::empty(), Some(""))?;
        println!("mounting /boot is todo");
    }

    Ok(())
}

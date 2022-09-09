use nix::{
    fcntl::{
        open,
        OFlag,
    },
    unistd::{
        dup2,
        close,
    },
    sys::{
        stat::Mode,
    },
};
use anyhow::Result;

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

pub fn init(config: &Config) -> Result<()> {
    if std::process::id() != 1 {
        return Ok(())
    }

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
    if config.mount_boot {
        todo!("Mounting /boot partition from BootCurrent efivar");
    }
    if config.mount_sys {
        todo!("Mounting /sys filesystem");
    }

    Ok(())
}

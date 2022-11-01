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
    dir::Dir,
};
use anyhow::Result;
use std::{
    collections::HashSet,
    time::Duration,
    path::PathBuf,
    thread,
};
use efivar::efi::VariableName;
use uefi::CStr16;
use uefi::proto::device_path::{PartitionFormat, PartitionSignature, DevicePath};
use gpt::GptConfig;

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
    if std::process::id() != 1 {
        return Ok(())
    }

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

    let has_efi = PathBuf::from("/sys/firmware/efi").exists();
    
    if config.mount_sys {
        mkdir("/sys/", Mode::from_bits(0o777).unwrap())?;
        mount(Some("none"), "/sys", Some("sysfs"), MsFlags::empty(), Some(""))?;

        if has_efi {
            mount(Some("efivarfs"), "/sys/firmware/efi/efivars", Some("efivarfs"), MsFlags::empty(), Some(""))?;
        }
    }

    if config.mount_boot && has_efi {
        let efi_mgr = efivar::system();
        let bootcurrent = VariableName::new("BootCurrent");
        let mut buf: [u8; 1024] = [0u8; 1024];
        let mut buf_cur: [u8; 2] = [0u8; 2];
        let mut esp_uuid = None;
        if let Ok(_) = efi_mgr.read(&bootcurrent, &mut buf_cur) {
            let cur = buf_cur[0] as u16 + buf_cur[1] as u16 * 256;
            let cur_var = VariableName::new(&format!("Boot{:#04}", cur));

            if let Ok(_) = efi_mgr.read(&cur_var, &mut buf) {
                let desc_start_offset = (32+16)/8;
                let desc = unsafe { CStr16::from_ptr(buf[desc_start_offset..].as_ptr().cast()) };
                let desc_end_offset = desc_start_offset + desc.num_bytes();

                let device_path: &DevicePath = unsafe {
                    std::mem::transmute(&buf[desc_end_offset..])
                };

                for node in device_path.node_iter() {
                    if let Some(hdd) = node.as_hard_drive_media_device_path() {
                        if hdd.partition_format() == PartitionFormat::GPT {
                            if let Some(PartitionSignature::GUID(uuid)) = hdd.partition_signature() {
                                esp_uuid = Some(uuid);
                            }
                        }
                    }
                }
            }
        }

        let mut seen_device_set = HashSet::new();
        let mut esp_path = None;
        if let Some(uuid) = esp_uuid {
            while esp_path.is_none() {
                if let Ok(mut blocks) = Dir::open("/sys/block/", OFlag::O_DIRECTORY | OFlag::O_RDONLY, Mode::empty()) {
                    let blocks_iter = blocks.iter();
                    for block in blocks_iter {
                        if let Ok(disk) = block {
                            if seen_device_set.contains(&disk) {
                                continue;
                            }

                            let name = disk.file_name().to_str().unwrap();
                            if name.starts_with('.') {
                                continue;
                            }
                            let dev_disk = PathBuf::from(format!("/dev/{}", name));
                            let gpt_cfg = GptConfig::new().writable(false);
                            if let Ok(gpt_disk) = gpt_cfg.open(dev_disk) {
                                for (i, part) in gpt_disk.partitions() {
                                    if format!("{}", uuid) == format!("{}", part.part_guid) {
                                        let with_p = PathBuf::from(format!("/dev/{}p{}", name, i));
                                        let without_p = PathBuf::from(format!("/dev/{}{}", name, i));

                                        if with_p.exists() {
                                            esp_path = Some(with_p);
                                        } else if without_p.exists() {
                                            esp_path = Some(without_p);
                                        }
                                        break;
                                    }
                                }
                            }

                            seen_device_set.insert(disk);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        }

        if let Some(path) = esp_path {
            mkdir("/boot/", Mode::from_bits(0o777).unwrap())?;
            mount(Some(&path), "/boot", Some("vfat"), MsFlags::empty(), Some(""))?;
        }
    }

    Ok(())
}

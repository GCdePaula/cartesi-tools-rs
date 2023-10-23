use crate::emulator::output::{AdvanceStatus, Outputs};

use super::{
    input::Input,
    machine_config::MachineConfig,
    output::{Notice, Voucher},
};

use clap::Parser;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command};
use thiserror::Error;

#[derive(Parser)]
#[command(author, version)]
pub struct MachineArguments {
    #[arg()]
    pub image: PathBuf,

    #[command(flatten)]
    pub config: MachineConfig,
}

#[derive(Error, Debug)]
pub enum MachineError {
    #[error("unknown")]
    Unknown,
}

type Result<T> = std::result::Result<T, MachineError>;

pub struct Machine {
    config: MachineConfig,
    handle: Child,
    input_index: usize,
    epoch_index: usize,
}

impl Default for Machine {
    fn default() -> Self {
        let args = MachineArguments::parse();
        Self::new(
            args.config,
            args.image.to_str().expect("invalid image path string"),
            0,
            0,
        )
    }
}

impl Machine {
    pub fn new(config: MachineConfig, image: &str, input_index: usize, epoch_index: usize) -> Self {
        let handle = spawn_remote_machine(&config, image);
        Self {
            config,
            input_index,
            epoch_index,
            handle,
        }
    }

    pub fn advance_state(&mut self, input: Input) -> AdvanceStatus {
        let dir = std::env::temp_dir();

        let metadata_file_pattern = {
            let metadata = input.encoded_metadata(self.input_index.try_into().unwrap());
            let mut d = dir.clone();
            d.push(format!(
                "epoch-{}-input-metadata-{}.bin",
                self.epoch_index, self.input_index
            ));
            let mut file =
                File::create(&d).expect("could not create temporary input metadata file");
            std::io::Write::write_all(&mut file, &metadata)
                .expect("could not write to temporary input metadata file");
            format!("{}/epoch-%e-input-metadata-%i.bin", dir.to_str().unwrap())
        };

        let input_file_pattern = {
            let payload = input.payload();
            let mut d = dir.clone();
            d.push(format!(
                "epoch-{}-input-{}.bin",
                self.epoch_index, self.input_index
            ));
            let mut file = File::create(&d).expect("could not create temporary input payload file");
            std::io::Write::write_all(&mut file, payload)
                .expect("could not write to temporary input payload file");
            format!("{}/epoch-%e-input-%i.bin", dir.to_str().unwrap())
        };

        let voucher_file_pattern =
            format!("{}/epoch-%e-input-%i-voucher-%o.bin", dir.to_str().unwrap());
        let notice_file_pattern =
            format!("{}/epoch-%e-input-%i-notice-%o.bin", dir.to_str().unwrap());
        let report_file_pattern =
            format!("{}/epoch-%e-input-%i-report-%o.bin", dir.to_str().unwrap());

        let status = Command::new(&self.config.cartesi_machine)
            .arg(format!(
                "--remote-address=localhost:{}",
                self.config.remote_machine_port
            ))
            .arg(format!(
                "--checkin-address=localhost:{}",
                REMOTE_MACHINE_CHECKIN_PORT
            ))
            .arg("--no-remote-create")
            .arg("--no-remote-destroy")
            .arg(format!(
                "--rollup-advance-state=epoch_index:0,input:{},input_metadata:{},input_index_begin:{},input_index_end:{},voucher:{},notice:{},report:{}",
                input_file_pattern,
                metadata_file_pattern,
                self.input_index,
                self.input_index + 1,
                voucher_file_pattern,
                notice_file_pattern,
                report_file_pattern
            ))
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .status()
            .expect("failed to execute cartesi-machine process").success();

        assert!(status, "cartesi-machine process failed");

        let notices: Vec<_> = (0..usize::MAX)
            .map_while(|notice_index| {
                let path = format!(
                    "{}/epoch-{}-input-{}-notice-{}.bin",
                    dir.to_str().unwrap(),
                    self.epoch_index,
                    self.input_index,
                    notice_index
                );

                File::open(path)
                    .map(|mut file| {
                        let mut payload = Vec::new();
                        file.read_to_end(&mut payload)
                            .expect("could not read notice file");
                        Notice { payload }
                    })
                    .ok()
            })
            .collect();

        let vouchers: Vec<_> = (0..usize::MAX)
            .map_while(|voucher_index| {
                let path = format!(
                    "{}/epoch-{}-input-{}-voucher-{}.bin",
                    dir.to_str().unwrap(),
                    self.epoch_index,
                    self.input_index,
                    voucher_index
                );

                File::open(path)
                    .map(|mut file| {
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer)
                            .expect("could not read voucher file");

                        Voucher::new(buffer)
                    })
                    .ok()
            })
            .collect();

        self.input_index += 1;

        AdvanceStatus::Ok(Outputs { notices, vouchers })
    }

    pub fn inspect(&self) -> Result<()> {
        todo!()
    }
}

const REMOTE_MACHINE_CHECKIN_PORT: u16 = 8081;

impl Drop for Machine {
    fn drop(&mut self) {
        let _ = self.handle.kill();
    }
}

fn spawn_remote_machine(config: &MachineConfig, image: &str) -> Child {
    let handle = Command::new(&config.remote_machine)
        .arg(format!(
            "--server-address=localhost:{}",
            config.remote_machine_port
        ))
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("couldn't spawn remote machine");

    assert!(
        Command::new(&config.cartesi_machine)
            .arg(format!("--load={}", image))
            .arg(format!(
                "--remote-address=localhost:{}",
                config.remote_machine_port
            ))
            .arg(format!(
                "--checkin-address=localhost:{}",
                REMOTE_MACHINE_CHECKIN_PORT
            ))
            .arg("--no-remote-destroy")
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .status()
            .expect("failed to execute cartesi-machine process (load)")
            .success(),
        "cartesi-machine process failed (load)"
    );

    handle
}

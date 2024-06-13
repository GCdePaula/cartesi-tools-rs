use alloy_primitives::{Address, U256};
use cartesi_machine::configuration::MachineConfigRef;
use clap::Parser;
use std::path::PathBuf;
use types::InputBuilder;

#[derive(Parser)]
#[command(author, version)]
pub struct MachineArguments {
    #[arg()]
    pub image: PathBuf,
}

type MachineError = cartesi_machine::errors::MachineError;
type Result<T> = std::result::Result<T, MachineError>;

pub struct Machine {
    chain_id: usize,
    dapp: Address,

    cartesi_machine: cartesi_machine::machine::Machine,
    input_index: usize,
}

impl TryFrom<PathBuf> for Machine {
    type Error = MachineError;
    fn try_from(_value: PathBuf) -> std::result::Result<Self, Self::Error> {
        todo!()
    }
}

impl Machine {
    // TODO consider builder pattern
    pub fn try_new(
        image: &PathBuf,
        dapp: Address,
        chain_id: usize,
        input_index: usize,
    ) -> Result<Self> {
        let runtime_config =
            cartesi_machine::configuration::RuntimeConfig::default().no_console_putchar(true);
        assert!(
            runtime_config.values.htif.no_console_putchar,
            "console getchar must be disabled for cmio"
        );

        // Instantiate Machine
        let cartesi_machine = {
            let cm = cartesi_machine::Machine::load(image, runtime_config)?;
            let c = cm.initial_config()?;
            sanity_check_cm_config(&c);
            cm
        };

        Ok(Self {
            chain_id,
            dapp,
            cartesi_machine,
            input_index,
        })
    }

    pub fn advance_state(&mut self, input: InputBuilder) -> Result<(Vec<Vec<u8>>, Vec<Vec<u8>>)> {
        let encoded_input = input.encode(self.chain_id, U256::from(self.input_index), self.dapp);

        self.cartesi_machine.send_cmio_response(
            cartesi_machine::htif::fromhost::ADVANCE_STATE,
            &encoded_input,
        )?;

        let mut outputs = Vec::new();
        let mut reports = Vec::new();

        loop {
            match self.cartesi_machine.run(1 << 40)? {
                cartesi_machine::break_reason::FAILED => panic!("run failed"),

                cartesi_machine::break_reason::HALTED => {
                    panic!("run halted: {}", self.cartesi_machine.read_mcycle()?)
                }

                cartesi_machine::break_reason::YIELDED_MANUALLY => {
                    let (_, reason, _) = get_yield(&self.cartesi_machine)?;

                    match reason {
                        cartesi_machine::htif::tohost::manual::TX_EXCEPTION => {
                            unimplemented!("TX_EXCEPTION not implemented")
                        }
                        cartesi_machine::htif::tohost::manual::RX_REJECTED => {
                            unimplemented!("RX_REJECTED not implemented")
                        }
                        cartesi_machine::htif::tohost::manual::RX_ACCEPTED => break,

                        i => unreachable!("cartesi machine impossible manual reason: {}", i),
                    }
                }

                cartesi_machine::break_reason::YIELDED_AUTOMATICALLY => {
                    let (_, reason, length) = get_yield(&self.cartesi_machine)?;
                    let data = self
                        .cartesi_machine
                        .read_memory(cartesi_machine::pma::CMIO_TX_BUFFER_START, length)?;

                    match reason {
                        cartesi_machine::htif::tohost::automatic::PROGRESS => (),

                        cartesi_machine::htif::tohost::automatic::TX_OUTPUT => outputs.push(data),

                        cartesi_machine::htif::tohost::automatic::TX_REPORT => reports.push(data),

                        i => unreachable!("cartesi machine impossible automatic reason: {}", i),
                    }
                }

                cartesi_machine::break_reason::YIELDED_SOFTLY => todo!(),

                cartesi_machine::break_reason::REACHED_TARGET_MCYCLE => todo!(),

                i => unreachable!("cartesi machine impossible break reason: {}", i),
            }
        }

        Ok((outputs, reports))
    }

    pub fn inspect(&self) -> Result<()> {
        todo!()
    }
}

fn sanity_check_cm_config(config: &MachineConfigRef) {
    // Sanity checks
    assert!(
        config.inner().htif.yield_manual,
        "yield manual must be enabled for cmio"
    );
    assert!(
        config.inner().htif.yield_automatic,
        "yield automatic must be enabled for cmio"
    );

    check_cmio_memory_range_config(config.inner().cmio.tx_buffer, "tx_buffer");
    check_cmio_memory_range_config(config.inner().cmio.rx_buffer, "rx_buffer");
}

fn check_cmio_memory_range_config(
    range: cartesi_machine::configuration::CmIoBufferConfig,
    name: &str,
) {
    assert!(!range.shared, "cmio range {} cannot be shared", name);
}

fn get_yield(machine: &cartesi_machine::Machine) -> Result<(isize, u32, u64)> {
    let cmd = machine.read_htif_tohost_cmd()? as isize;
    let data = machine.read_htif_tohost_data()?;

    let reason = data >> 32;
    let m16 = (1 << 16) - 1;
    let reason = reason & m16;
    let m32 = (1 << 32) - 1;
    let length = data & m32;

    Ok((cmd, reason as u32, length))
}

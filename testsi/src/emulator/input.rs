use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolType};

type SolInputMetadata = sol! { (address, uint256, uint256, uint256, uint256) };
type SolInput = sol! { bytes };

#[derive(Clone, Debug)]
pub struct Input {
    pub sender: Address,
    pub block_number: U256,
    pub block_timestamp: U256,
    pub payload: Vec<u8>,
}

impl Input {
    pub fn from_address(sender: Address) -> Self {
        Self {
            sender,
            block_number: U256::ZERO,
            block_timestamp: U256::ZERO,
            payload: Vec::new(),
        }
    }

    pub fn at_block(mut self, block: usize) -> Self {
        self.block_number = block.try_into().unwrap();
        self
    }

    pub fn with_payload(mut self, payload: impl AsRef<str>) -> Self {
        self.payload = SolInput::abi_encode(payload.as_ref());
        self
    }

    pub fn with_timestamp_block(mut self, block_timestamp: usize) -> Self {
        self.block_timestamp = block_timestamp.try_into().unwrap();
        self
    }

    pub fn encoded_metadata(&self, input_index: U256) -> Vec<u8> {
        SolInputMetadata::abi_encode(&(
            self.sender,
            self.block_number,
            self.block_timestamp,
            U256::ZERO,
            input_index,
        ))
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

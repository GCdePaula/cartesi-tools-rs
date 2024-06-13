use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};

// TODO create crate with alloy type definitions in rollup-contracts once this is merged:
// https://github.com/foundry-rs/foundry/pull/7919
sol! {
    function EvmAdvance(
        uint256 chainId,
        address appContract,
        address msgSender,
        uint256 blockNumber,
        uint256 blockTimestamp,
        uint256 index,
        bytes calldata payload
    ) external;

    function Notice(bytes calldata payload) external;

    function Voucher(
        address destination,
        uint256 value,
        bytes calldata payload
    ) external;
}

pub type Input = EvmAdvanceCall;
pub type Voucher = VoucherCall;
pub type Notice = NoticeCall;

#[derive(Clone, Debug)]
pub struct InputBuilder {
    pub sender: Address,
    pub block_number: U256,
    pub block_timestamp: U256,
    pub payload: Vec<u8>,
}

impl InputBuilder {
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
        self.payload = payload.as_ref().into();
        self
    }

    pub fn with_block_timestamp(mut self, block_timestamp: usize) -> Self {
        self.block_timestamp = block_timestamp.try_into().unwrap();
        self
    }

    pub fn encode(self, chain_id: usize, input_index: U256, dapp: Address) -> Vec<u8> {
        let x = EvmAdvanceCall::new((
            U256::from(chain_id),
            dapp,
            self.sender,
            self.block_number,
            self.block_timestamp,
            input_index,
            self.payload,
        ));

        x.abi_encode()
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

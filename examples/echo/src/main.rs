use alloy_primitives::Address;
use alloy_sol_types::{sol, SolType};
use testsi::{emulator::output::AdvanceStatus, Input, Machine};

testsi::testsi_main!();

type SolInput = sol! { bytes };

#[testsi::test_dapp(kind("dapp"))]
pub fn test_echo() -> testsi::TestResult {
    let mut machine = Machine::default();

    // Input 0
    let input = Input::from_address(Address::ZERO).with_payload("hello");
    let status = machine.advance_state(input);
    match status {
        AdvanceStatus::Ok(x) => {
            assert_eq!(x.notices[0].payload, SolInput::abi_encode("hello"));
        }
        AdvanceStatus::Rejected => panic!("should not have reverted"),
    }

    // Input 1
    let input = Input::from_address(Address::ZERO).with_payload("world");
    let status = machine.advance_state(input);
    match status {
        AdvanceStatus::Ok(x) => {
            assert_eq!(x.notices[0].payload, SolInput::abi_encode("world"));
        }
        AdvanceStatus::Rejected => panic!("should not have reverted"),
    }

    Ok(())
}

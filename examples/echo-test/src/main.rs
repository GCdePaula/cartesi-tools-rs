use std::{path::PathBuf, str::FromStr};

use alloy_primitives::Address;
use testsi::Machine;
use types::InputBuilder;

testsi::testsi_main!();

#[testsi::test_dapp(kind("dapp"))]
pub fn test_echo() -> testsi::TestResult {
    let mut machine = Machine::try_new(
        &PathBuf::from_str("../../zz/echo")?,
        Address::ZERO,
        31337,
        0,
    )?;

    // Input 0
    let input = InputBuilder::from_address(Address::ZERO).with_payload("hello");
    let (_outputs, reports) = machine.advance_state(input)?;
    assert_eq!(&reports[0], "hello".as_bytes());

    Ok(())
}

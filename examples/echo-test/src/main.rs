use types::alloy_primitives::Address;

testsi::testsi_main!();

#[testsi::test_dapp(kind("dapp"))]
pub fn test_echo() -> testsi::TestResult {
    let mut machine = testsi::MachineBuilder::load_from("./out/machine-image")
        .at_chain(31337)
        .no_console_putchar(false)
        .try_build()?;

    let payload = b"hello from echo-test".to_vec();
    let input = testsi::InputBuilder::from_address(Address::ZERO).with_payload(&payload);
    let (outputs, _reports) = machine.advance_state(input)?;

    assert_eq!(outputs.notices().len(), 1);
    assert_eq!(
        outputs[0].expect_notice().payload.as_ref(),
        payload.as_slice()
    );

    Ok(())
}

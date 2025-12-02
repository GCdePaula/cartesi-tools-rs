use trolley::cmt;

fn main() {
    let rollup = cmt::RollupCmt::try_new().expect("failed to initialize rollup");
    let app = echo_lib::App::new(rollup);
    app.run();
}

use trolley::{InputMetadata, Rollup, RollupRequest};
use types::Notice;

#[derive(Default)]
pub struct App<R: Rollup> {
    pub rollup: R,
}

impl<R: Rollup> App<R> {
    pub fn new(rollup: R) -> App<R> {
        App { rollup }
    }

    pub fn run(mut self) -> ! {
        loop {
            match self.rollup.next_input() {
                Ok(RollupRequest::Advance { metadata, payload }) => {
                    self.advance(&metadata, &payload)
                }
                Ok(RollupRequest::Inspect { payload }) => self.inspect(&payload),
                Err(err) => panic!("failed to fetch next rollup input: {err}"),
            }
        }
    }

    pub fn advance(&mut self, _metadata: &InputMetadata, payload: &[u8]) {
        self.rollup
            .emit_notice(&Notice {
                payload: payload.to_vec().into(),
            })
            .expect("failed to emit notice");
    }

    pub fn inspect(&mut self, _inspect: &[u8]) {
        todo!()
    }
}

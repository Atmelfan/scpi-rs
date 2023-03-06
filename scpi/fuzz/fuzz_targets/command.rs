#![no_main]
use libfuzzer_sys::fuzz_target;

use scpi::{prelude::*, tree::prelude::*, Root, Leaf};

struct Dummy;
impl<D> Command<D> for Dummy
where
    D: Device,
{}

pub(crate) struct TestDevice;

impl TestDevice {
    pub(crate) fn new() -> Self {
        TestDevice
    }
}

impl Device for TestDevice {
    fn handle_error(&mut self, _err: Error) {}
}


const TREE: Node<TestDevice> = Root![
    // Create default IEEE488 mandated commands
    Leaf!(b"DUMB" => &Dummy)
];

fuzz_target!(|data: &[u8]| {
    let mut dev = TestDevice::new();

    let mut context = Context::default();
    let mut buf = Vec::<u8>::new();
    //Result
    let res = TREE.run(data, &mut dev, &mut context, &mut buf);
    // Don't care if it errors but a ErrorCode::DeviceSpecificError indicates a unexpected parser error
    if let Err(err) = res {
        assert!(err != ErrorCode::DeviceSpecificError)

    }
});

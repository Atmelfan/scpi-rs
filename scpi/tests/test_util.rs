// Util macros to setup context and tree
#[macro_export]
macro_rules! context {
    ($context:ident, $dev:ident) => {
        // Infrastructure
        let mut $dev = TestDevice::new();
        let mut errors = ArrayErrorQueue::<[Error; 10]>::new();
        let mut $context = Context::new(&mut $dev, &mut errors, IEEE488_TREE);
    };
}

#[macro_export]
macro_rules! execute_str {
    ($context:expr, $s:expr => $res:ident, $dat:ident $x:tt) => {
        //Response bytebuffer
        let mut buf = ArrayVecFormatter::<[u8; 256]>::new();
        //Result
        let $res = $context.run($s, &mut buf);
        let $dat = buf.as_slice();
        $x;
    };
}

#[macro_export]
macro_rules! check_esr {
    ($context:ident == $esr:literal) => {
    execute_str!($context, b"*esr?" => result, response {
        assert_eq!(result, Ok(()));
        assert_eq!(response, $esr);
    });
    };
    ($context:ident) => {
    check_esr!($context == b"0\n");
    };
}

// Util macros to make testing  on complete parser simpler

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
        //SCPI tokenizer
        let mut tokenizer = Tokenizer::new($s);
        //Result
        let $res = $context.exec(&mut tokenizer, &mut buf);
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

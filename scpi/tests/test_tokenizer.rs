use scpi::tokenizer::{Tokenizer};



#[test]
fn parse_header(){
    let headers = [
        b":MEASure:VOLTage:DC? 1, .1, 1.0 KHZ, -1.0E2, .43E-6 MV, +.45E+2; MESSage \"POTATO\", 12.6 MOHM".as_ref(),
        b"*IDN?".as_ref(),
        "*RST #s\"åäö\"".as_bytes(),
        b"MEASure:VOLTage? 1.23 KOHM, 5".as_ref()
    ];

    for header in &headers {
        println!("Testing '{}'", String::from_utf8_lossy(header));

        let tokenizer = Tokenizer::from_str(header);

        for token in tokenizer {
            match token {
                Err(err) => {
                    println!("Error {}", err as i32);
                },
                Ok(token) => {
                    println!("{:?}", token)
                }
            }
        }
    }


}
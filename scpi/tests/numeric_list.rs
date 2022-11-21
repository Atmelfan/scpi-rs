mod util;
use scpi::error::Result;
use scpi::prelude::*;
use scpi::prelude::numeric_list::NumericList;
use scpi::qonly;

struct NumericListCommand;

impl Command<util::TestDevice> for NumericListCommand {
    qonly!();

    fn query(
        &self,
        _device: &mut util::TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        let numbers: NumericList = args.next()?.try_into()?;
        for item in numbers {
            match item? {
                numeric_list::Token::Numeric(a) => {
                    response.data(f32::try_from(a)?);
                }
                numeric_list::Token::NumericRange(a, b) => {
                    response.data(f32::try_from(a)?);
                    response.data(f32::try_from(b)?);
                }
            }
        }
        response.finish()
    }
}


#[test]
fn test_numeric_list() {
    const NUMERIC_TREE: Node<util::TestDevice> = Branch {
        name: b"",
        sub: &[Leaf {
            name: b"*NUMLIST",
            default: false,
            handler: &NumericListCommand,
        }],
    };
    let mut dev = util::TestDevice::new();
    util::test_file(&mut dev, &NUMERIC_TREE, "tests/csv/numeric_list.csv");
}
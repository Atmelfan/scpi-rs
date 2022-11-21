mod util;
use scpi::error::Result;
use scpi::prelude::*;
use scpi::prelude::channel_list::ChannelList;
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
        let numbers: ChannelList = args.next()?.try_into()?;
        for item in numbers {
            match item? {
                channel_list::Token::ChannelSpec(t) => {
                    let ch: usize = t.try_into()?;
                    response.data(ch);
                },
                channel_list::Token::ChannelRange(a, b) => {
                    let a: usize = a.try_into()?;
                    let b: usize = b.try_into()?;
                    for i in a..=b {
                        response.data(i);
                    }
                },
                channel_list::Token::PathName(s) => {
                    response.data(s);
                },
                channel_list::Token::ModuleChannel(_, _) => todo!(),
            }
        }
        response.finish()
    }
}


#[test]
fn test_channel_list() {
    const NUMERIC_TREE: Node<util::TestDevice> = Branch {
        name: b"",
        sub: &[Leaf {
            name: b"*CHLIST",
            default: false,
            handler: &NumericListCommand,
        }],
    };
    let mut dev = util::TestDevice::new();
    util::test_file(&mut dev, &NUMERIC_TREE, "tests/csv/channel_list.csv");
}
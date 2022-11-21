mod util;
use scpi::error::Result;
use scpi::parameters::NumericValues;
use scpi::prelude::*;
use scpi::qonly;

struct NumCommand;
impl Command<util::TestDevice> for NumCommand {
    qonly!();

    fn query(
        &self,
        _device: &mut util::TestDevice,
        _context: &mut Context,
        mut args: Arguments,
        mut response: ResponseUnit,
    ) -> Result<()> {
        // Parse a <numeric>
        let x: NumericValues<f32> = args.next()?.try_into()?;

        // Use builder to resolve special values
        let value = x.clone()
            // Specify required max,min values
            .with(100.0, -100.0)
            // Specify the default value
            .default(Default::default())
            // Specify the current value
            // Used to calculate final value when `UP` or `DOWN` is used
            .current(Default::default())
            // Finish the builder and resolve the final value,
            // Provided clousure is called when `AUTO` is used to determine the final value
            .finish_auto(|| 42.0)?;
        
        // Sanity check
        let option = match x {
            NumericValues::Value(_) => 0,
            NumericValues::Maximum => 1,
            NumericValues::Minimum => 2,
            NumericValues::Default => 3,
            NumericValues::Up => 4,
            NumericValues::Down => 5,
            NumericValues::Auto => 6,
        };
        response.data(value).data(option).finish()
    }
}

#[test]
fn test() {
    const NUMERIC_TREE: Node<util::TestDevice> = Branch {
        name: b"",
        sub: &[Leaf {
            name: b"*NUM",
            default: false,
            handler: &NumCommand,
        }],
    };
    let mut dev = util::TestDevice::new();
    util::test_file(&mut dev, &NUMERIC_TREE, "tests/csv/numeric.csv")
}

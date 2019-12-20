# Cortex-m SCPI example

## About this example
This example will 

## Cortex-m-quickstart
This example is based on the excellent [cortex-m-quickstart](https://github.com/rust-embedded/cortex-m-quickstart) guide.

Some changes must be made to the .cargo/config file to successfully compile:
 * The default linker must be changed to `arm-non-eabi-gcc`, the default rust-lld isn't happy about certain missing 
 functions used by the lexical-core dependency (used to parse numerics).
 * Examples folder was removed, again due to compiler complaints. 
 * Build must also be run in release mode (`cargo build|run --release`) or the linker will again, fail. I don't know why.

Hopefully this gets more stable in the future as rust-embedded becomes more mature.


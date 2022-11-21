mod util;
use scpi::{
    ieee488::commands::*, ieee488_cls, ieee488_ese, ieee488_esr, ieee488_idn, ieee488_opc,
    ieee488_rst, ieee488_sre, ieee488_stb, ieee488_tst, ieee488_wai, prelude::*,
    scpi1999::commands::*, scpi_status, scpi_system,
};

const IEEE488_TREE: Node<util::TestDevice> = Branch {
    name: b"",
    sub: &[
        // Create default IEEE488 mandated commands
        ieee488_cls!(),
        ieee488_ese!(),
        ieee488_esr!(),
        ieee488_idn!(b"GPA-Robotics", b"T800-101", b"0", b"0"),
        ieee488_opc!(),
        ieee488_rst!(),
        ieee488_sre!(),
        ieee488_stb!(),
        ieee488_tst!(),
        ieee488_wai!(),
        scpi_status!(),
        scpi_system!(),
    ],
};

#[test]
fn test_common() {
    let mut dev = util::TestDevice::new();
    util::test_file(&mut dev, &IEEE488_TREE, "tests/csv/test.csv")
}

#[test]
fn test_tree_traversal() {
    let mut dev = util::TestDevice::new();
    util::test_file(&mut dev, &IEEE488_TREE, "tests/csv/tree_traversal.csv")
}

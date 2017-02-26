extern crate subprocess;

use std::env;
use subprocess::Exec;

const NAME: &'static str = "envsub";

fn exe(vars: &str) -> Exec {
    let root = env::current_exe()
        .unwrap()
        .parent()
        .expect("failed locating exe dir")
        .to_path_buf();
    let exe = root.join(NAME);
    let exe_str = exe.to_str().unwrap();
    // Abuse shell method until cmd has env var support.
    Exec::shell(format!("{} {}", vars, exe_str))
}

#[test]
fn output_without_replacements() {
    let expected = "foo bar baz";
    let output = exe("")
        .stdin(expected)
        .capture()
        .expect("failed building cmd")
        .stdout_str();
    assert_eq!(output, expected);
}

#[test]
fn output_with_one_replacement() {
    let expected = "foo foo baz";
    let output = exe("VAR=foo")
        .stdin("foo %VAR% baz")
        .capture()
        .expect("failed building cmd")
        .stdout_str();
    assert_eq!(output, expected);
}

#[test]
fn multiple_inputs() {
    /// Test data from kubernetes var expansion proposal doc.
    let cmd_prefix = "ENVSUB_PREFIX='$(' ENVSUB_SUFFIX=')' VAR_A=A VAR_B=B VAR_C=C VAR_EMPTY=''";
    println!("{}", cmd_prefix);
    let table = vec![("$(VAR_A)", "A"),
                     ("___$(VAR_B)___", "___B___"),
                     ("___$(VAR_C)", "___C"),
                     ("$(VAR_A)-$(VAR_A)", "A-A"),
                     ("$(VAR_A)-1", "A-1"),
                     ("$(VAR_A)_$(VAR_B)_$(VAR_C)", "A_B_C"),
                     ("foo\\$(VAR_C)bar", "foo\\Cbar"),
                     ("foo\\\\$(VAR_C)bar", "foo\\\\Cbar"),
                     ("foo\\\\\\\\$(VAR_A)bar", "foo\\\\\\\\Abar"),
                     ("foo$(VAR_EMPTY)bar", "foobar"),
                     ("foo$(VAR_Awhoops!", "foo$(VAR_Awhoops!"),
                     ("f00__(VAR_A)__", "f00__(VAR_A)__"),
                     ("$?_boo_$!", "$?_boo_$!"),
                     ("$VAR_A", "$VAR_A"),
                     ("$(VAR_DNE)", "$(VAR_DNE)"),
                     ("$$$$$$(BIG_MONEY)", "$$$$$$(BIG_MONEY)"),
                     ("$VAR_A)", "$VAR_A)"),
                     ("${VAR_A}", "${VAR_A}"),
                     ("$(VAR_B)_______$(A", "B_______$(A"),
                     ("$(VAR_C)_______$(", "C_______$("),
                     ("$(VAR_A)foobarzab$", "Afoobarzab$"),
                     ("foo-\\$(VAR_A", "foo-\\$(VAR_A"),
                     ("--$($($($($--", "--$($($($($--"),
                     ("$($($($($--foo$(", "$($($($($--foo$("),
                     ("foo0--$($($($(", "foo0--$($($($("),
                     ("$(foo$$var)", "$(foo$$var)")];
    for (input, expected_output) in table {
        let output = exe(cmd_prefix)
            .stdin(input)
            .capture()
            .expect("failed building cmd")
            .stdout_str();
        assert_eq!(output, expected_output);
    }
}

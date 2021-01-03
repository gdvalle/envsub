extern crate subprocess;

use std::env;
use std::path::PathBuf;
use subprocess::Exec;

const NAME: &'static str = "envsub";

pub fn cargo_root() -> PathBuf {
    env::current_exe()
        .unwrap()
        .parent()
        .expect("failed locating exe dir")
        .parent()
        .expect("failed locating parent exe dir")
        .to_path_buf()
}

/// Repurposed from BurntSushi/ripgrep/tests/util.rs.
/// This determines the proper executable when we're in a cross compiler.
fn cross_runner() -> Option<String> {
    for (k, v) in std::env::vars_os() {
        let (k, v) = (k.to_string_lossy(), v.to_string_lossy());
        if !(k.starts_with("CARGO_TARGET_") && k.ends_with("_RUNNER")) {
            continue;
        }
        return Some(v.into_owned());
    }
    None
}

/// Returns the path to the executable.
pub fn cargo_exe(name: &str) -> PathBuf {
    cargo_root().join(format!("{}{}", name, env::consts::EXE_SUFFIX))
}

fn exe() -> Exec {
    let bin_path = cargo_exe(NAME);
    return match cross_runner() {
        None => Exec::cmd(bin_path),
        Some(runner) => Exec::shell(format!(
            "{} {}",
            runner,
            bin_path.into_os_string().into_string().unwrap()
        )),
    };
}

#[test]
fn output_without_replacements() {
    let expected = "foo bar baz";
    let output = exe()
        .stdin(expected)
        .capture()
        .expect("failed running test")
        .stdout_str();
    assert_eq!(output, expected);
}

#[cfg(not(windows))]
#[test]
fn output_with_one_replacement() {
    let expected = "foo foo baz";
    let output = exe()
        .env("VAR", "foo")
        .stdin("foo %VAR% baz")
        .capture()
        .expect("failed running test")
        .stdout_str();
    assert_eq!(output, expected);
}

#[cfg(not(windows))]
#[test]
fn multiple_inputs() {
    // Test data from kubernetes var expansion proposal doc.
    let table = vec![
        ("$(VAR_A)", "A"),
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
        ("$(foo$$var)", "$(foo$$var)"),
    ];
    for (input, expected_output) in table {
        let output = exe()
            .env_extend(&[
                ("ENVSUB_PREFIX", "$("),
                ("ENVSUB_SUFFIX", ")"),
                ("VAR_A", "A"),
                ("VAR_B", "B"),
                ("VAR_C", "C"),
                ("VAR_EMPTY", ""),
            ])
            .stdin(input)
            .capture()
            .expect("failed running test")
            .stdout_str();
        assert_eq!(output, expected_output);
    }
}

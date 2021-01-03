#![warn(clippy::all)]

extern crate aho_corasick;

use aho_corasick::AhoCorasick;
use std::env::{self, VarError};
use std::io::{self, Read, Write};
use std::process;

macro_rules! println_err(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("error: failed printing to stderr");
    } }
);

// Prefix env var namespace.
fn envvar(key: &str) -> Result<String, VarError> {
    env::var(format!("ENVSUB_{}", key))
}

fn replace(patterns: &[String], replacements: &[String], text: &str) -> String {
    let ac = AhoCorasick::new(patterns);
    let mut target_end = 0;
    let mut o: Vec<String> = vec![];

    for m in ac.find_iter(&text) {
        o.push(text[target_end..m.start()].into());
        o.push(replacements[m.pattern()].as_str().into());
        target_end = m.end();
    }

    o.push(text[target_end..].into());
    o.join("")
}

fn get_vars() -> Vec<String> {
    // Gather list of env vars.
    let args = env::args();
    let mut vars = Vec::new();
    if args.len() > 1 {
        // Vars are provided via CLI args.
        for arg in args.skip(1) {
            vars.push(arg);
        }
    } else {
        // Vars discovered from the environment.
        for (key, _) in env::vars() {
            vars.push(key);
        }
    };
    vars
}

fn get_patterns(vars: Vec<String>, prefix: &str, suffix: &str) -> (Vec<String>, Vec<String>) {
    let mut patterns = Vec::new();
    let mut replacements = Vec::new();
    for (i, var) in vars.iter().enumerate() {
        let pattern = format!("{}{}{}", &prefix, &var, &suffix);
        let replacement = match env::var(&vars[i]) {
            Ok(v) => v,
            Err(_) => {
                println_err!("error: env var not found: {}", var);
                process::exit(1)
            }
        };
        patterns.push(pattern);
        replacements.push(replacement);
    }
    (patterns, replacements)
}

fn main() {
    let prefix = match envvar("PREFIX") {
        Ok(v) => v,
        Err(_) => String::from("%"),
    };
    let suffix = match envvar("SUFFIX") {
        Ok(v) => v,
        Err(_) => String::from("%"),
    };

    let vars = get_vars();
    let (patterns, replacements) = get_patterns(vars, &prefix, &suffix);

    let reader = io::stdin();
    let writer = io::stdout();

    // Build an in-memory buffer so users can replace files in-place.
    // i.e. envsub < foo.txt > foo.txt
    let mut buffer = String::new();
    reader
        .lock()
        .read_to_string(&mut buffer)
        .expect("Failed reading stdin");
    let replaced = replace(&patterns, &replacements, &buffer);
    writer
        .lock()
        .write_all(replaced.as_bytes())
        .expect("Failed writing to stdout");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_without_replacements() {
        let string = "just\na\nnormal\nstring\n".to_owned();
        let keys = vec!["x".to_owned()];
        let vals = vec!["xyz".to_owned()];
        let output = replace(&keys, &vals, &string);
        assert_eq!(string, output);
    }

    #[test]
    fn test_replace_with_one_variable() {
        let string = "%VAR%".to_owned();
        let replacement = "moo".to_owned();
        let keys = vec!["%VAR%".to_owned()];
        let vals = vec![replacement.clone()];
        let output = replace(&keys, &vals, &string);
        assert_eq!(output, replacement);
    }

    #[test]
    fn test_replace_with_multiple_variables() {
        let string = "%VAR1%, %VAR2%, %VAR3%".to_owned();
        let keys = vec![
            "%VAR1%".to_owned(),
            "%VAR2%".to_owned(),
            "%VAR3%".to_owned(),
        ];
        let vals = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        let output = replace(&keys, &vals, &string);
        assert_eq!(output, vals.join(", "));
    }

    #[test]
    fn test_replace_with_single_variable_multiple_times() {
        let string = "%VAR1%, %VAR1%, %VAR1%".to_owned();
        let keys = vec![
            "%VAR1%".to_owned(),
            "%VAR2%".to_owned(),
            "%VAR3%".to_owned(),
        ];
        let vals = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        let output = replace(&keys, &vals, &string);
        assert_eq!(output, "foo, foo, foo".to_owned());
    }

    #[test]
    fn test_multiline_replacement() {
        let string = "%VAR3%\n%VAR1\n".to_owned();
        let keys = vec![
            "%VAR1%".to_owned(),
            "%VAR2%".to_owned(),
            "%VAR3%".to_owned(),
        ];
        let vals = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        let output = replace(&keys, &vals, &string);
        assert_eq!(output, "baz\n%VAR1\n".to_owned());
    }
}

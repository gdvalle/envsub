#![feature(alloc_system)]
extern crate alloc_system;
extern crate aho_corasick;

use std::env::{self, VarError};
use std::io::{self, Read, Write};
use std::process;
use aho_corasick::{Automaton, AcAutomaton};

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

fn replace(keys: &Vec<String>, values: &Vec<String>, target: &String) -> String {
    let aut = AcAutomaton::new(keys);
    let matches = aut.find(&target);
    let mut target_end = 0;
    let mut o: Vec<String> = vec![];

    for m in matches {
        o.push(target[target_end..m.start].into());
        o.push(values[m.pati].as_str().into());
        target_end = m.end;
    }

    o.push(target[target_end..].into());
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

fn get_kvs(vars: Vec<String>, prefix: &str, suffix: &str) -> (Vec<String>, Vec<String>) {
    // Build two vecs, one for search patterns, one for replacement text.
    let mut keys = Vec::new();
    let mut vals = Vec::new();
    for (i, var) in vars.iter().enumerate() {
        // Form the search key.
        let key = format!("{}{}{}", &prefix, &var, &suffix);
        let val = match env::var(&vars[i]) {
            Ok(v) => v,
            Err(_) => {
                println_err!("error: env var not found: {}", var);
                process::exit(1)
            },
        };
        keys.push(key);
        vals.push(val);
    }
    (keys, vals)
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

    // Gather list of env vars.
    let vars = get_vars();
    // Build search keys and values.
    let (keys, vals) = get_kvs(vars, &prefix, &suffix);

    let reader = io::stdin();
    let writer = io::stdout();

    // Build an in-memory buffer so users can replace files in-place.
    // i.e. envsub < foo.txt > foo.txt
    let mut buffer = String::new();
    let _ = reader.lock().read_to_string(&mut buffer);
    let replaced = replace(&keys, &vals, &buffer);
    let _ = writer.lock().write(&replaced.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_without_replacements() {
        let string = String::from("just\na\nnormal\nstring\n");
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
        assert_eq!(output, replacement.clone());
    }

    #[test]
    fn test_replace_with_multiple_variables() {
        let string = "%VAR1%, %VAR2%, %VAR3%".to_owned();
        let keys = vec!["%VAR1%".to_owned(), "%VAR2%".to_owned(), "%VAR3%".to_owned()];
        let vals = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        let output = replace(&keys, &vals, &string);
        assert_eq!(output, vals.join(", "));
    }

    #[test]
    fn test_replace_with_single_variable_multiple_times() {
        let string = "%VAR1%, %VAR1%, %VAR1%".to_owned();
        let keys = vec!["%VAR1%".to_owned(), "%VAR2%".to_owned(), "%VAR3%".to_owned()];
        let vals = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        let output = replace(&keys, &vals, &string);
        assert_eq!(output, "foo, foo, foo".to_owned());
    }

    #[test]
    fn test_multiline_replacement() {
        let string = "%VAR3%\n%VAR1\n".to_owned();
        let keys = vec!["%VAR1%".to_owned(), "%VAR2%".to_owned(), "%VAR3%".to_owned()];
        let vals = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        let output = replace(&keys, &vals, &string);
        assert_eq!(output, "baz\n%VAR1\n".to_owned());
    }
}

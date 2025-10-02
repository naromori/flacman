use crate::args::{build_cli, handle_matches};

mod args;

#[test]
fn test() {
    let argsz = build_cli().get_matches();
    handle_matches(&argsz);
}
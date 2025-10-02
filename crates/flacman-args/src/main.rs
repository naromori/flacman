use crate::args::handle_matches;

mod args;
fn main() {
    let matches = args::build_cli().get_matches();
    handle_matches(&matches);
}
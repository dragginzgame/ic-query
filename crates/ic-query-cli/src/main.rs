use ic_query::run_from_env;

// Run the IC query CLI and report errors in a shell-friendly form.
fn main() {
    if let Err(err) = run_from_env() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

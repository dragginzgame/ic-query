use ic_query_cli::run_from_env;

// Run the IC query CLI and report errors in a shell-friendly form.
fn main() {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("failed to create CLI query runtime: {error}");
            std::process::exit(1);
        }
    };
    if let Err(err) = runtime.block_on(run_from_env()) {
        if err.is_broken_pipe() {
            return;
        }
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }
}

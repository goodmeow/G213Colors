mod app;
mod cli;
mod command;
mod config;
mod device;
mod error;
mod product;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cli::has_cli_args() {
        cli::run()?;
        return Ok(());
    }

    app::run()?;
    Ok(())
}

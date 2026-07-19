use clap::{Parser, Subcommand};

use crate::command::{segment_command, Effect, Rgb};
use crate::config::{self, save_user_config, user_config_path};
use crate::device;
use crate::error::Result;
use crate::product::{spec_for, Product};

pub fn has_cli_args() -> bool {
    std::env::args_os().len() > 1
}

#[derive(Debug, Parser)]
#[command(name = "g213colors")]
#[command(about = "Logitech G213 lighting controller")]
struct Cli {
    #[arg(short = 't', long = "apply-system-default")]
    apply_system_default: bool,

    #[arg(long, num_args = 0..=1, default_missing_value = "G213")]
    apply_user_config: Option<Product>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Detect,
    SetStatic { color: String },
    SetBreathe { color: String, speed_ms: u32 },
    SetCycle { speed_ms: u32 },
    SetSegment { segment: u8, color: String },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.apply_system_default {
        device::apply_config_file(config::system_default_path())?;
        println!("Applied system default config for G213.");
        return Ok(());
    }

    if let Some(product) = cli.apply_user_config {
        device::apply_config_file(user_config_path(product)?)?;
        println!("Applied user config for {product}.");
        return Ok(());
    }

    match cli.command {
        Some(Command::Detect) => {
            if device::detect(Product::G213) {
                println!("{} detected.", spec_for(Product::G213).display_name);
            } else {
                println!("G213 not detected.");
            }
        }
        Some(Command::SetStatic { color }) => {
            let color = Rgb::parse_hex(&color)?;
            device::apply_effect(Product::G213, &Effect::Static(color))?;
            println!("Applied G213 static color.");
        }
        Some(Command::SetBreathe { color, speed_ms }) => {
            let color = Rgb::parse_hex(&color)?;
            device::apply_effect(Product::G213, &Effect::Breathe { color, speed_ms })?;
            println!("Applied G213 breathe effect.");
        }
        Some(Command::SetCycle { speed_ms }) => {
            device::apply_effect(Product::G213, &Effect::Cycle { speed_ms })?;
            println!("Applied G213 cycle effect.");
        }
        Some(Command::SetSegment { segment, color }) => {
            let color = Rgb::parse_hex(&color)?;
            let spec = spec_for(Product::G213);
            let commands = vec![segment_command(spec, segment, color)?];
            device::apply_commands(spec, &commands)?;
            save_user_config(Product::G213, &commands)?;
            println!("Applied G213 segment {segment} color.");
        }
        None => {
            println!("No CLI action requested. Run without arguments to launch the GUI.");
        }
    }

    Ok(())
}

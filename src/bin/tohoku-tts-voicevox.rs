
use tohoku_tts_voicevox::{self as tohoku, SynthesisVariant, EngineHandle};

use std::io::Write;
use std::io::Read;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "A simple CLI for Voicevox Core", long_about = None, version)]
struct Cli {
    #[command(subcommand)]
    subcommand: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// One-shot synthesis
    #[command(arg_required_else_help = true)]
    TestSynthesis {
        #[arg(long, value_enum)]
        variant: SynthesisVariant,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Cli::parse();

    match args.subcommand {
        Command::TestSynthesis { variant } => test_synthesis(variant)?,
    }

    Ok(())
}

fn test_synthesis(variant: SynthesisVariant) -> anyhow::Result<()> {
    let dir = "./voicevox_core/open_jtalk_dic_utf_8-1.11";
    
    tohoku::initialize(dir)?;

    let handle = EngineHandle::new()?;

    let mut text = String::new();
    let _ = std::io::stdin().read_to_string(&mut text)?;
    let wav = handle.synthesize_blocking(text, variant)?;
    std::io::stdout().write_all(wav.as_slice())?;

    Ok(())
}

use tohoku_tts_voicevox::{self as tohoku, SynthesisVariant, SynthesisParams, SynthesisOptions, EngineHandle};

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
        /// Dialect variant
        #[arg(long, value_enum)]
        variant: SynthesisVariant,

        /// Pitch offset
        #[arg(long, default_value_t = SynthesisParams::default().pitch_offset())]
        pitch_offset: f64,

        /// Pitch range
        #[arg(long, default_value_t = SynthesisParams::default().pitch_range())]
        pitch_range: f64,

        /// Speed scale
        #[arg(long, default_value_t = SynthesisParams::default().speed_scale())]
        speed_scale: f64,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Cli::parse();

    match args.subcommand {
        Command::TestSynthesis { variant, pitch_offset, pitch_range, speed_scale } => {
            let params = SynthesisParams::new(pitch_offset, pitch_range, speed_scale)?;
            let options = SynthesisOptions {
                params,
                variant,
            };
            test_synthesis(options)?;
        },
    }

    Ok(())
}

fn test_synthesis(options: SynthesisOptions) -> anyhow::Result<()> {
    let dir = "./voicevox_core/open_jtalk_dic_utf_8-1.11";
    
    tohoku::initialize(dir)?;

    let handle = EngineHandle::new()?;

    let mut text = String::new();
    let _ = std::io::stdin().read_to_string(&mut text)?;
    let wav = handle.synthesize_blocking(text, options)?;
    std::io::stdout().write_all(wav.as_slice())?;

    Ok(())
}

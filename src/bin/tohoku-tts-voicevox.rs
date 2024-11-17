
use tohoku_tts_voicevox::{self as tohoku, SynthesisVariant, SynthesisParams, SynthesisOptions, EngineHandle};

use std::io::Write;
use std::io::Read;

use clap::{Parser, Subcommand};

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const PKG_NAME_JA: &str = "ジェネリック東北共通語読み上げソフト";

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

        /// Speak the program version and sample text (ignores input)
        #[arg(long)]
        speak_sample_text: bool,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Cli::parse();

    match args.subcommand {
        Command::TestSynthesis { variant, pitch_offset, pitch_range, speed_scale, speak_sample_text } => {
            let params = SynthesisParams::new(pitch_offset, pitch_range, speed_scale)?;
            let options = SynthesisOptions {
                params,
                variant,
            };

            if speak_sample_text {
                let text = sample_text();
                test_synthesis(options, &text)?;
            } else {
                let mut text = String::new();
                let _ = std::io::stdin().read_to_string(&mut text)?;
                test_synthesis(options, &text)?;
            }
        },
    }

    Ok(())
}

fn test_synthesis(options: SynthesisOptions, text: &str) -> anyhow::Result<()> {
    let dir = "./voicevox_core/open_jtalk_dic_utf_8-1.11";
    
    tohoku::initialize(dir)?;

    let handle = EngineHandle::new()?;

    let wav = handle.synthesize_blocking(text.to_owned(), options)?;
    std::io::stdout().write_all(wav.as_slice())?;

    Ok(())
}

fn sample_text() -> String {
    format!(r#"
これは、{}、バージョン{}です。
これは、ジェネリックな東北共通語っぽい音声合成ができるソフトです。 
このように、一般的な現代日本語の任意の文章を方言風のアクセント・イントネーションで読みあげさせることができます。
いわゆる標準語を訛らせて発話させることを想定したもので、伝統的な方言、例えば津軽弁、南部弁、ケセン語、会津弁などを再現することを目的としたものではありません。
小規模な簡易ネイティブチェックを行い、合成音声の範囲内で自然さには配慮しておりますが、精密に特定の場所の方言に準じてつくっているわけではありません。
また、方言の参照用として使うことを想定したものではありません。
ご注意ください。
このソフトウェアは、アパッチライセンス・バージョン2.0のもとでライセンスされています。
また、このソフトウェアは、ボイスボックス・コアを使用しております。
"#, PKG_NAME_JA, VERSION.replace(".", "てん"))
}

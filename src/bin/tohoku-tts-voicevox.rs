
use tohoku_tts_voicevox::{self as tohoku, SynthesisVariant, EngineHandle};

use std::io::Write;
use std::io::Read;


fn main() -> anyhow::Result<()> {
    env_logger::init();

    let dir = "./voicevox_core/open_jtalk_dic_utf_8-1.11";
    
    tohoku::initialize(dir)?;

    let variant = SynthesisVariant::Northern;

    let handle = EngineHandle::new()?;

    let mut text = String::new();
    let _ = std::io::stdin().read_to_string(&mut text)?;
    let wav = handle.synthesize_blocking(text, variant)?;
    std::io::stdout().write_all(wav.as_slice())?;

    Ok(())
}

use std::io::Write;
use vvcore::*;

fn main() -> anyhow::Result<()> {
    let dir = std::ffi::CString::new("./voicevox_core/open_jtalk_dic_utf_8-1.11").unwrap();
    let vvc = VoicevoxCore::new_from_options(AccelerationMode::Auto, 0, true, dir.as_c_str()).unwrap();
    let text = std::io::read_to_string(std::io::stdin())?;
    let speaker_id = 2u32;
    let json = match vvc.audio_query(&text, speaker_id, AudioQueryOptions { kana: false }) {
        Ok(json) => {
            json.as_str().to_owned()
        }
        Err(e) => {
            let msg = VoicevoxCore::error_result_to_message(e);
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    eprintln!("{}", &json);

    let wav = match vvc.synthesis(&json, speaker_id, SynthesisOptions { enable_interrogative_upspeak: false }) {
        Ok(wav) => {
            wav
        }
        Err(e) => {
            let msg = VoicevoxCore::error_result_to_message(e);
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };
    
    std::io::stdout().write_all(wav.as_slice())?;

    Ok(())
}

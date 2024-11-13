
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioQuery {
    pub accent_phrases: Vec<AccentPhrase>,
    pub speed_scale: f64,
    pub pitch_scale: f64,
    pub intonation_scale: f64,
    pub volume_scale: f64,
    pub pre_phoneme_length: f64,
    pub post_phoneme_length: f64,
    pub output_sampling_rate: i32,
    pub output_stereo: bool,
    pub kana: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccentPhrase {
    pub moras: Vec<Mora>,
    pub accent: i32,
    pub pause_mora: Option<Mora>,
    pub is_interrogative: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mora {
    pub text: String,
    pub vowel: String,
    pub vowel_length: f64,
    pub pitch: f64,
    pub consonant: Option<String>,
    pub consonant_length: Option<f64>,
}

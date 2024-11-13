
use vvcore::*;
use std::sync::OnceLock;
use std::ffi::CString;

use tokio::sync::mpsc;
use tokio::sync::oneshot;
use std::path::Path;
use std::fmt::Debug;

use clap::ValueEnum;

use crate::types;
use crate::error::*;
use crate::EngineErrorDescription;
use crate::EngineError;

static ENGINE: OnceLock<EngineHandle> = OnceLock::new();

type InternalError = GenericError<&'static str>;

struct EngineRequestData<Req, Res> {
    req: Req,
    res_sender: oneshot::Sender<Res>,
}

impl<Req, Res> EngineRequestData<Req, Res>
where 
    Req: Send + 'static,
    Res: Send + 'static,
{
    pub fn new(req: Req) -> (Self, oneshot::Receiver<Res>) {
        let (res_sender, res_receiver) = oneshot::channel();
        (Self {
            req,
            res_sender,
        }, res_receiver)
    }
}

impl<Req, Res> Debug for EngineRequestData<Req, Res>
where 
    Req: Debug,
    Res: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineRequestData")
            .field("req", &self.req)
            .field("res_sender", &self.res_sender)
            .finish()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum SynthesisVariant {
    Northern,
    Southern,
}

impl SynthesisVariant {
    fn make_voiced_consonant(&self, consonant: &str) -> Option<String> {
        match consonant {
            "k" => Some("g".to_string()),
            "s" => Some("z".to_string()),
            "t" => Some("d".to_string()),
            _ => Some(consonant.to_string()),
        }
    }
    fn preprocess_audio_query(&self, query: types::AudioQuery) -> types::AudioQuery {
        let unvoiced_vowels = vec!["A", "I", "U", "E", "O"];
        let voiced_consonants = vec!["g", "z", "d", "b", "n"];

        let mut query = query.clone();
        query.speed_scale = 1.1;
        query.pitch_scale = 0.0;
        query.intonation_scale = 0.9;
        query.volume_scale = 1.0;

        match self {
            Self::Northern => {
                let mut accent_phrases = query.accent_phrases.clone();
                for i in 0..accent_phrases.len() {
                    let accent_phrases_len = accent_phrases.len();
                    let accent_phrase = &mut accent_phrases[i];
                    assert!(accent_phrase.moras.len() > 0);
                    let pitches = accent_phrase.moras.iter().map(|m| m.pitch).filter(|pitch| *pitch != 0.0).collect::<Vec<f64>>();
                    assert!(pitches.len() > 0);
                    let avg_pitch = pitches.iter().sum::<f64>() / pitches.len() as f64;

                    let accent = (accent_phrase.accent - 1) as usize; // originally 1-indexed
                    for j in 0..accent_phrase.moras.len() {
                        let mut mora = accent_phrase.moras[j].clone();
                        if unvoiced_vowels.contains(&mora.vowel.as_str()) {
                            mora.vowel = mora.vowel.to_lowercase();
                            mora.vowel_length *= 1.5;
                            if let Some(len) = &mora.consonant_length {
                                mora.consonant_length = Some(len * 0.6);
                            }
                        }
                        let next_mora = j + 1;
                        if next_mora < accent_phrase.moras.len() {
                            let next_mora = accent_phrase.moras[next_mora].clone();
                            if let Some(consonant) = &next_mora.consonant {
                                if voiced_consonants.contains(&consonant.as_str()) {
                                   if let Some(consonant) = &mora.consonant {
                                       mora.consonant = self.make_voiced_consonant(consonant);
                                   }
                                }
                            }
                            if voiced_consonants.contains(&next_mora.vowel.as_str()) {
                                mora.vowel = mora.vowel.to_uppercase();
                            }
                        }
                        let last_mora = (accent_phrase.moras.len() - 1) == j;
                        if last_mora {
                            mora.vowel_length *= 1.75;
                        }

                        if j < accent {
                            mora.pitch = avg_pitch * 0.95;
                        } else if j == accent {
                            mora.pitch = avg_pitch * 1.07;
                        } else {
                            mora.pitch = avg_pitch;
                        }
                        if i == accent_phrases_len - 1 && last_mora {
                            mora.vowel_length *= 1.25;
                            if accent_phrase.is_interrogative {
                                mora.pitch *= 1.02;
                            } else {
                                mora.pitch *= 0.96;
                            }
                        }
                        accent_phrase.moras[j] = mora;
                    }
                }
                query.accent_phrases = accent_phrases;
                query
            },
            Self::Southern => {
                let mut accent_phrases = query.accent_phrases.clone();
                for i in 0..accent_phrases.len() {
                    let accent_phrases_len = accent_phrases.len();
                    let accent_phrase = &mut accent_phrases[i];
                    assert!(accent_phrase.moras.len() > 0);
                    let pitches = accent_phrase.moras.iter().map(|m| m.pitch).filter(|pitch| *pitch != 0.0).collect::<Vec<f64>>();
                    assert!(pitches.len() > 0);
                    let avg_pitch = pitches.iter().sum::<f64>() / pitches.len() as f64;

                    let accent = (accent_phrase.moras.len() - 1) as usize;
                    for j in 0..accent_phrase.moras.len() {
                        let mut mora = accent_phrase.moras[j].clone();
                        let next_mora = j + 1;
                        let mut next_unvoiced = false;
                        if next_mora < accent_phrase.moras.len() {
                            let next_mora = accent_phrase.moras[next_mora].clone();
                            if let Some(consonant) = &next_mora.consonant {
                                if voiced_consonants.contains(&consonant.as_str()) {
                                   if let Some(consonant) = &mora.consonant {
                                       mora.consonant = self.make_voiced_consonant(consonant);
                                   }
                                }
                            }
                            if unvoiced_vowels.contains(&next_mora.vowel.as_str()) {
                                next_unvoiced = true;
                            }
                            if voiced_consonants.contains(&next_mora.vowel.as_str()) {
                                mora.vowel = mora.vowel.to_uppercase();
                            }
                        }

                        let last_mora = j == (accent_phrase.moras.len() - 1) || next_unvoiced && j == accent_phrase.moras.len() - 2;
                        if last_mora {
                            mora.vowel_length *= 1.75;
                        }

                        if j == 0 {
                            mora.pitch = avg_pitch * 0.95;
                        } else if j < accent {
                            mora.pitch = avg_pitch * 1.04;
                        } else if j == accent {
                            mora.pitch = avg_pitch * 1.05;
                        } else {
                            mora.pitch = avg_pitch * 0.95;
                        }
                        if i == accent_phrases_len - 1 && last_mora {
                            mora.vowel_length *= 1.25;
                            if accent_phrase.is_interrogative {
                                mora.pitch *= 1.02;
                            } else {
                                mora.pitch *= 0.99;
                            }
                        }
                        accent_phrase.moras[j] = mora;
                    }
                }
                query.accent_phrases = accent_phrases;
                query
            },
        }
    }
}

#[derive(Debug)]
enum EngineRequest {
    Synthesis(EngineRequestData<(String, SynthesisVariant), Result<Vec<u8>, InternalError>>),
}

struct Runner {
    vvc: VoicevoxCore,
    receiver: mpsc::Receiver<EngineRequest>,
}

impl Runner {
    fn start<P: AsRef<Path>>(open_jtalk_dict_dir: P) -> Result<EngineHandle, InternalError> {
        let dir = if let Ok(dir) = CString::new(open_jtalk_dict_dir.as_ref().to_str().unwrap()) {
            dir
        } else {
            return Err(InternalError::new("Failed to convert path to CString"));
        };
        let vvc = VoicevoxCore::new_from_options(AccelerationMode::Auto, 0, true, dir.as_c_str()).unwrap();
        
        let (req_sender, req_receiver) = mpsc::channel(100);

        let runner = Runner {
            vvc,
            receiver: req_receiver,
        };

        std::thread::spawn(move || {
            runner.run();
        });

        Ok(EngineHandle {
            sender: req_sender,
        })
    }

    fn run(self) {
        let vvc = self.vvc;
        let mut receiver = self.receiver;

        loop {
            match receiver.blocking_recv() {
                Some(EngineRequest::Synthesis(data)) => {
                    let (text, variant) = data.req;

                    let speaker_id = 2u32;
                    let query: types::AudioQuery = match vvc.audio_query(&text, speaker_id, AudioQueryOptions { kana: false }) {
                        Ok(json) => {
                            match serde_json::from_str(json.as_str()) {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("Failed to parse JSON: {}", e);
                                    let err = InternalError::new("Failed to parse JSON");
                                    let _ = data.res_sender.send(Err(err));
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            let msg = VoicevoxCore::error_result_to_message(e);
                            let err = InternalError::new(msg);
                            let _ = data.res_sender.send(Err(err));
                            continue;
                        }
                    };

                    let query = variant.preprocess_audio_query(query);

                    let json = serde_json::to_string(&query).unwrap();

                    log::debug!("Synthesizing with JSON: {}", json);

                    let res = vvc.synthesis(&json, speaker_id, SynthesisOptions { enable_interrogative_upspeak: false });

                    match res {
                        Ok(wav) => {
                            let _ = data.res_sender.send(Ok(wav.as_slice().to_owned()));
                        },
                        Err(e) => {
                            let msg = VoicevoxCore::error_result_to_message(e);
                            let err = InternalError::new(msg);
                            let _ = data.res_sender.send(Err(err));
                        },
                    }
                },
                None => break,
            }
        }

        log::warn!("Runner thread exited");
    }
}

#[derive(Debug, Clone)]
pub struct EngineHandle {
    sender: mpsc::Sender<EngineRequest>,
}

impl EngineHandle {
    pub fn new() -> Result<EngineHandle, EngineError> {
        ENGINE.get().map(|handle| handle.clone()).ok_or(EngineError::new(EngineErrorDescription::NotInitialized))
    }

    pub fn synthesize_blocking(&self, text: String, variant: SynthesisVariant) -> Result<Vec<u8>, InternalError> {
        let (data, receiver) = EngineRequestData::new((text, variant));
        self.sender.blocking_send(EngineRequest::Synthesis(data)).unwrap();
        receiver.blocking_recv().unwrap()
    }

    pub async fn synthesize(&self, text: String, variant: SynthesisVariant) -> Result<Vec<u8>, InternalError> {
        let (data, receiver) = EngineRequestData::new((text, variant));
        self.sender.send(EngineRequest::Synthesis(data)).await.unwrap();
        receiver.await.unwrap()
    }
}

pub fn initialize<P: AsRef<Path>>(dir: P) -> Result<(), EngineError> {
    let handle = Runner::start(dir);
    let handle = if let Ok(handle) = handle {
        handle
    } else {
        return Err(EngineError::new(EngineErrorDescription::InitializationFailed));
    };
    ENGINE.set(handle).map_err(|_| EngineError::new(EngineErrorDescription::AlreadyInitialized))
}

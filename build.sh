#!/bin/sh

export LD_LIBRARY_PATH=./voicevox_core:$LD_LIBRARY_PATH

dir=$( pwd -P )

export RUSTFLAGS="-C link-args=-Wl,-rpath,${dir}/voicevox_core"

cargo build --release

cp target/release/tohoku-tts-voicevox ./

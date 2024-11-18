# tohoku-tts-voicevox

**ジェネリック東北共通語読み上げソフト** (VoiceVox Core 版)

ジェネリックな東北共通語 (他所行きの発話) 風の音声合成ができるソフトです。

いわゆる「標準語」を訛らせて発話させることを想定したもので，伝統的な方言 (津軽弁、南部弁、ケセン語、会津弁など) を再現することを目的としたものではありません。

以下のバージョンを実装しています。

- 北東北 (北奥羽アクセント圏)
- 南東北 (無アクセント圏)

小規模な簡易ネイティブチェックを行い、合成音声の範囲内で自然さには配慮しておりますが、精密に特定の場所の方言に準じてつくっているわけではございません。

また、方言の参照用として使うことを想定したものではありませんので，ご注意ください。

このソフトウェアは [VoiceVox Core](https://github.com/VOICEVOX/voicevox_core) を使用しております。
VoiceVox Core 側が配布している音源モデルは自由なライセンス (OSS的な意味で) ではありませんので，利用規約にご注意ください。

なお，このソフトウェアは， VoiceVox 公式のものではございません。

## Usage

設定：

```bash
git clone https://github.com/metastable-void/tohoku-tts-voicevox.git
cd tohoku-tts-voicevox
./download-deps.sh
./build.sh
```

北東北版の例：

```bash
./tohoku-tts-voicevox play-synthesis --variant=northern --pitch-offset=-2 --speed-scale=0.9 --speak-sample-text
```

南東北版の例：

```bash
./tohoku-tts-voicevox play-synthesis --variant=southern --speed-scale=0.9 --speak-sample-text
```

詳しくは， `./tohoku-tts-voicevox help` を実行してください。

## License

Apache License, version 2.0.

use {
    kokoro_tts::{KokoroTts, Voice},
    voxudio::AudioPlayer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tts = KokoroTts::new("kokoro-v1.0.int8.onnx", "voices.bin").await?;
    let (audio, took) = tts
        .synth(
            "Hello, world!你好，我们是一群追逐梦想的人。我正在使用qq。",
            Voice::ZfXiaoxiao(1.2),
        )
        .await?;
    println!("Synth took: {:?}", took);
    let mut player = AudioPlayer::new()?;
    player.play()?;
    player.write::<24000>(&audio, 1).await?;

    Ok(())
}

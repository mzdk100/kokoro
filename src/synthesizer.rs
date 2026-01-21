use {
    crate::{KokoroError, Voice, g2p, get_token_ids},
    ndarray::Array,
    ort::{
        inputs,
        session::{RunOptions, Session},
        value::TensorRef,
    },
    std::{
        cmp::min,
        sync::Weak,
        time::{Duration, SystemTime},
    },
    tokio::sync::Mutex,
};

async fn synth_v10<P, S>(
    model: Weak<Mutex<Session>>,
    phonemes: S,
    pack: P,
    speed: f32,
) -> Result<(Vec<f32>, Duration), KokoroError>
where
    P: AsRef<Vec<Vec<Vec<f32>>>>,
    S: AsRef<str>,
{
    let model = model.upgrade().ok_or(KokoroError::ModelReleased)?;
    let phonemes = get_token_ids(phonemes.as_ref(), false);
    let phonemes = Array::from_shape_vec((1, phonemes.len()), phonemes)?;
    let ref_s = pack.as_ref()[phonemes.len() - 1]
        .first()
        .cloned()
        .unwrap_or_default();

    let style = Array::from_shape_vec((1, ref_s.len()), ref_s)?;
    let speed = Array::from_vec(vec![speed]);
    let options = RunOptions::new()?;
    let mut model = model.lock().await;
    let t = SystemTime::now();
    let kokoro_output = model
        .run_async(
            inputs![
                "tokens" => TensorRef::from_array_view(&phonemes)?,
                "style" => TensorRef::from_array_view(&style)?,
                "speed" => TensorRef::from_array_view(&speed)?,
            ],
            &options,
        )?
        .await?;
    let elapsed = t.elapsed()?;
    let (_, audio) = kokoro_output["audio"].try_extract_tensor::<f32>()?;

    Ok((audio.to_owned(), elapsed))
}

async fn synth_v11<P, S>(
    model: Weak<Mutex<Session>>,
    phonemes: S,
    pack: P,
    speed: i32,
) -> Result<(Vec<f32>, Duration), KokoroError>
where
    P: AsRef<Vec<Vec<Vec<f32>>>>,
    S: AsRef<str>,
{
    let model = model.upgrade().ok_or(KokoroError::ModelReleased)?;
    let mut phonemes = get_token_ids(phonemes.as_ref(), true);

    let mut ret = Vec::new();
    let mut elapsed = Duration::ZERO;
    while let p = phonemes.drain(..min(pack.as_ref().len(), phonemes.len()))
        && p.len() != 0
    {
        let phonemes = Array::from_shape_vec((1, p.len()), p.collect())?;
        let ref_s = pack.as_ref()[phonemes.len() - 1]
            .first()
            .cloned()
            .unwrap_or(vec![0.; 256]);

        let style = Array::from_shape_vec((1, ref_s.len()), ref_s)?;
        let speed = Array::from_vec(vec![speed]);
        let options = RunOptions::new()?;
        let mut model = model.lock().await;
        let t = SystemTime::now();
        let kokoro_output = model
            .run_async(
                inputs![
                    "input_ids" => TensorRef::from_array_view(&phonemes)?,
                    "style" => TensorRef::from_array_view(&style)?,
                    "speed" => TensorRef::from_array_view(&speed)?,
                ],
                &options,
            )?
            .await?;
        elapsed = t.elapsed()?;
        let (_, audio) = kokoro_output["waveform"].try_extract_tensor::<f32>()?;
        let (_, _duration) = kokoro_output["duration"].try_extract_tensor::<i64>()?;
        // let _ = dbg!(duration.len());
        ret.extend_from_slice(audio);
    }

    Ok((ret, elapsed))
}

pub(super) async fn synth<P, S>(
    model: Weak<Mutex<Session>>,
    text: S,
    pack: P,
    voice: Voice,
) -> Result<(Vec<f32>, Duration), KokoroError>
where
    P: AsRef<Vec<Vec<Vec<f32>>>>,
    S: AsRef<str>,
{
    let phonemes = g2p(text.as_ref(), voice.is_v11_supported())?;
    // #[cfg(debug_assertions)]
    // println!("{}", phonemes);
    match voice {
        v if v.is_v11_supported() => synth_v11(model, phonemes, pack, v.get_speed_v11()?).await,
        v if v.is_v10_supported() => synth_v10(model, phonemes, pack, v.get_speed_v10()?).await,
        v => Err(KokoroError::VoiceVersionInvalid(v.get_name().to_owned())),
    }
}

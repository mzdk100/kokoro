use {
    crate::{KokoroError, Voice},
    futures::{Sink, SinkExt, Stream},
    pin_project::pin_project,
    std::{
        pin::Pin,
        task::{Context, Poll},
        time::Duration,
    },
    tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};

struct Request<S> {
    voice: Voice,
    text: S,
}

struct Response {
    data: Vec<f32>,
    took: Duration,
}

/// 语音合成流
///
/// 该结构体用于通过流式合成来处理更长的文本。它实现了`Stream` trait，可以用于异步迭代合成后的音频数据。
#[pin_project]
pub struct SynthStream {
    #[pin]
    rx: UnboundedReceiver<Response>,
}

impl Stream for SynthStream {
    type Item = (Vec<f32>, Duration);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.project().rx)
            .poll_recv(cx)
            .map(|i| i.map(|Response { data, took }| (data, took)))
    }
}

/// 语音合成发送端
///
/// 该结构体用于发送语音合成请求。它实现了`Sink` trait，可以用于异步发送合成请求。
#[pin_project]
pub struct SynthSink<S> {
    tx: UnboundedSender<Request<S>>,
    voice: Voice,
}

impl<S> SynthSink<S> {
    /// 设置语音名称
    ///
    /// 该方法用于设置要合成的语音名称。
    ///
    /// # 参数
    ///
    /// * `voice_name` - 语音名称，用于选择要合成的语音。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use kokoro_tts::{KokoroTts, Voice};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let Ok(tts) = KokoroTts::new("../kokoro-v1.0.int8.onnx", "../voices.bin").await else {
    ///         return;
    ///     };
    ///     // speed: 1.0
    ///     let (mut sink, _) = tts.stream::<&str>(Voice::ZfXiaoxiao(1.0));
    ///     // speed: 1.8
    ///     sink.set_voice(Voice::ZmYunxi(1.8));
    /// }
    /// ```
    ///
    pub fn set_voice(&mut self, voice: Voice) {
        self.voice = voice
    }

    /// 发送合成请求
    ///
    /// 该方法用于发送语音合成请求。
    ///
    /// # 参数
    ///
    /// * `text` - 要合成的文本内容。
    ///
    /// # 返回值
    ///
    /// 如果发送成功，将返回`Ok(())`；如果发送失败，将返回一个`KokoroError`类型的错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use kokoro_tts::{KokoroTts, Voice};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let Ok(tts) = KokoroTts::new("../kokoro-v1.1-zh.onnx", "../voices-v1.1-zh.bin").await else {
    ///         return;
    ///     };
    ///     let (mut sink, _) =tts.stream(Voice::Zf003(2));
    ///     let _ = sink.synth("hello world.").await;
    /// }
    /// ```
    ///
    pub async fn synth(&mut self, text: S) -> Result<(), KokoroError> {
        self.send((self.voice, text)).await
    }
}

impl<S> Sink<(Voice, S)> for SynthSink<S> {
    type Error = KokoroError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, (voice, text): (Voice, S)) -> Result<(), Self::Error> {
        self.tx
            .send(Request { voice, text })
            .map_err(|e| KokoroError::Send(e.to_string()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

pub(super) fn start_synth_session<F, R, S>(
    voice: Voice,
    synth_request_callback: F,
) -> (SynthSink<S>, SynthStream)
where
    F: Fn(S, Voice) -> R + Send + 'static,
    R: Future<Output = Result<(Vec<f32>, Duration), KokoroError>> + Send,
    S: AsRef<str> + Send + 'static,
{
    let (tx, mut rx) = unbounded_channel::<Request<S>>();
    let (tx2, rx2) = unbounded_channel();
    tokio::spawn(async move {
        while let Some(req) = rx.recv().await {
            let (data, took) = synth_request_callback(req.text, req.voice).await?;
            tx2.send(Response { data, took })
                .map_err(|e| KokoroError::Send(e.to_string()))?;
        }

        Ok::<_, KokoroError>(())
    });

    (SynthSink { tx, voice }, SynthStream { rx: rx2 })
}

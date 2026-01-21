# Kokoro TTS的rust推理实现

[Kokoro](https://github.com/hexgrad/kokoro)

> **Kokoro**是具有8200万参数的开放式TTS型号。
> 尽管具有轻巧的体系结构，但它的质量与大型型号相当，同时更快，更具成本效益。使用Apache许可的权重，可以将Kokoro部署从生产环境到个人项目的任何地方。


## 概述

本项目包含幾个示例脚本，展示了如何使用Kokoro库进行语音合成。这些示例展示了如何直接合成语音和通过流式合成来处理更长的文本。

## 前置条件

- Rust编程语言
- tokio异步运行时
- voxudio音频处理和播放的库（可选）
- 下载模型资源，在這裡可以找到[1.0模型](https://github.com/mzdk100/kokoro/releases/tag/V1.0)和[1.1模型](https://github.com/mzdk100/kokoro/releases/tag/V1.1)

## 特点
- 跨平台，可以轻松在Windows、Mac OS上构建，也可以轻松交叉编译到安卓和iOS。
- 离线推理，不依赖网络。
- 足够轻量级，有不同尺寸的模型可以选择（最小的模型仅88M）。
- 发音人多样化，跨越多国语言。

## 使用方法

1. 运行示例，克隆或下载本项目到本地。在项目根目录下运行：
    ```shell
    cargo run --example synth_directly_v10
    cargo run --example synth_directly_v11
    ```
2. 集成到自己的项目中：
    ```shell
    cargo add kokoro-tts
    ```
3. Linux依赖项
    ```shell
    sudo apt install libasound2-dev
    ```
参考[examples](examples)文件夹中的示例代码进行开发。


## 许可证

本项目采用Apache-2.0许可证。请查看项目中的LICENSE文件了解更多信息。

## 注意

- 请确保在运行示例之前已经正确加载了模型和语音数据。
- 示例中的语音合成参数（如语音名称、文本内容、速度等）仅作为示例，实际使用时请根据需要进行调整。

## 贡献

如果您有任何改进意见或想要贡献代码，请随时提交Pull Request或创建Issue。

## 免责声明

本项目中的示例代码仅用于演示目的。在使用本项目中的代码时，请确保遵守相关法律法规和社会主义核心价值观。开发者不对因使用本项目中的代码而导致的任何后果负责。
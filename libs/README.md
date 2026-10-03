# 本机运行资源

模型与第三方运行二进制不会提交到 GitHub（见仓库根目录 `.gitignore`）。需要完整运行桌面截图识别和本机引擎时，请自行准备并放置有权使用的对应版本：

- `models/board.onnx`：本项目使用的 15 类象棋识别 ONNX 模型。若可依法使用用户本机已安装的 xqlink 模型，可运行 `node scripts/extract-model.mjs <xqlink.exe 路径>` 提取到此目录。
- `runtime/onnxruntime.dll` 与 `runtime/onnxruntime_providers_shared.dll`：与 `ort` 依赖兼容的 ONNX Runtime Windows DLL。
- `pikafish/pikafish-windows.exe`、`pikafish/pikafish.nnue`：Windows Pikafish 引擎和网络文件；对外分发时请遵循对应版本的 GPL-3.0 条款，并提供所需源代码及许可文本。

`desktop:build` 的 Tauri 资源配置会从这些固定路径打包文件。未准备相应资源时，可继续开发前端；桌面打包或相关识别/引擎功能会因缺少文件而失败。

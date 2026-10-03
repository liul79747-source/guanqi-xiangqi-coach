# 第三方资源

本项目为用户本机研习工具。以下原始来源与许可保留：

- **atopx/chessboard**，作者 atopx，Apache-2.0，https://github.com/atopx/chessboard 。`LICENSE` 保留其完整许可证。识别标签、预处理及棋盘裁剪思路参考该项目；`server/src/vision.rs` 已重新实现并标注修改。图标来自该项目。
- **识别模型**：从用户已安装的 xqlink 0.1.2 CPU 程序内提取原始 ONNX 模型，未修改权重。提取脚本为 `scripts/extract-model.mjs`。标签顺序以参考项目源码为准；不假设任意 YOLO 模型可直接替换。
- **Pikafish**：https://github.com/official-pikafish/Pikafish ，GPL-3.0。引擎和 NNUE 来自用户提供的参考项目 `libs/pikafish`，本软件经独立子进程 UCI 接口调用。引擎源码及构建说明在上游仓库。若对外分发含引擎的版本，需同时履行其对应版本源代码及许可证的分发义务。
- **ONNX Runtime**：Microsoft，MIT，https://github.com/microsoft/onnxruntime 。DLL 来自用户本机 xqlink CPU 安装目录。
- **Vue / Vite / Naive UI / TypeScript / Tauri / xcap / ort**：各项目的原有许可证仍适用。具体锁定版本见 package-lock.json 与 server/Cargo.lock。

该软件的新界面、规则校验、棋谱记录与运行适配于 2026-10-02 编写。

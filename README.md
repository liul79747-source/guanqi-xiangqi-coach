# 观棋 · 象棋研习室

按照提供的 Vue 3 + TypeScript + Vite + Naive UI / Tauri 2 + Rust 架构制作的本机象棋学习软件。

## 运行

双击项目根目录的 **启动观棋.cmd**。存在 `output/观棋/guanqi.exe` 时直接打开桌面版；否则启动本机浏览器版。桌面便携版需要保持 `guanqi.exe` 与同目录 `libs` 文件夹一起，不能单独移动 exe。

也可以直接双击 `output/观棋/guanqi.exe`。本次交付为便携版，无需安装。程序支持命令行 `guanqi.exe --diagnose`，可检查随附模型与引擎是否可用。

浏览器版依赖 Node.js 24，访问地址为 `http://127.0.0.1:1422`。关闭启动窗口会停止本地服务。此版支持摆棋与真实 Pikafish 分析，窗口截图请使用桌面版。

## 已实现功能

- 棋盘点击走棋、合法落点、翻转、悔棋、棋谱回看与自动保存。
- 自由摆棋、选择行棋方、FEN 导入与复制、JSON 棋谱导出。
- 先查询 ChessDB 云库；未命中、超时或失败后自动使用本机 Pikafish。
- 中文推荐着法、箭头、评分、搜索深度、耗时与最多 12 个半回合的主变化。
- Tauri 桌面窗口列表、xcap 截图、ONNX 棋盘棋子识别、连续帧投票确认、连续识别与暂停。
- 识别后以 10×9 棋盘生成 FEN。用户设置首次行棋方；跟踪变化时优先回放最多 4 手合法着法，自动修复短暂漏帧并提示核对棋谱。
- 深度、思考时间、线程、内存、云库开关与超时设置。

默认采用较快分析参数（本机最多思考 1 秒，云库等待最多 1 秒）；可在“分析设置”中提高时限和搜索深度，以换取更深入的计算。

评分以当前行棋方为视角，正数有利、负数不利；`杀 N` 是引擎的将杀距离。主变化来自引擎/云库。“研习提示”为着法规则说明，不是模型生成的策略解释。

## 窗口识别

打开桌面版 → 窗口识别 → 选择窗口 → 选定首次行棋方 → 识别一次 / 连续识别。

目标窗口需要显示完整棋盘，不能最小化。识别沿用参考项目模型的 15 类标签；不同棋盘皮肤、遮挡、动画可能影响准确率。连续识别不会操作目标软件，只读取截图。程序对最近 3 帧逐格多数投票，并要求棋盘连续稳定后才更新；漏掉 1–4 手时尝试按合法走棋序列恢复行棋方，超过范围或局面异常时会自动重置跟踪基准、继续识别并提示核对行棋方，不会困在旧局面反复要求校正。目标画面暂时识别失败时，会在恢复识别后继续跟踪。

## 开发与验证

```powershell
npm install
npm test
npm run build
npm start
```

前端开发时另开一个终端运行 `npm start`（本地分析接口），然后 `npm run dev`。

桌面开发与打包：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/desktop.ps1 -Dev
powershell -ExecutionPolicy Bypass -File scripts/desktop.ps1
```

编译需要 Rust MSVC 工具链、Microsoft C++ Build Tools 与 Windows SDK。`scripts/desktop.ps1` 会优先使用本机 `.tools` 中的工具链（若已准备），否则使用系统安装的 Cargo。首次构建需要联网下载依赖；本机 Cargo 镜像设置不纳入仓库。

Rust 测试：`cargo test --manifest-path server/Cargo.toml --lib`。在 Windows 上运行桌面版/完整测试还需要 MSVC 工具链及相应的模型和引擎资源。

测试覆盖基础棋规、FEN 校验、真实引擎搜索、中文着法、界面状态更新、旧分析结果隔离、ONNX 模型加载与参考截图完整局面识别。内存 DOM 测试不等于屏幕布局检查；本次浏览器可视检查被保存的访问权限设置阻止。

## 目录

- `src/`：Vue 界面与象棋规则。
- `server/`：Tauri / Rust 后端，含窗口截图、模型推理与 UCI 分析。
- `scripts/`：本机浏览器适配、启动与模型提取工具。
- `libs/`：Pikafish、NNUE、ONNX 模型和运行库。
- `tests/`：规则、引擎、界面测试；参考截图来源见第三方声明。

参考项目提供的源码未被修改。原项目、Pikafish 与 ONNX Runtime 的来源和许可证见 `THIRD_PARTY_NOTICES.md`。资源文件较大，已在 `.gitignore` 中单独排除；备份时请连同 `libs` 一起保存。

克隆仓库后，`libs` 下的引擎、NNUE、ONNX 模型和运行库不会自动下载。请按 [`libs/README.md`](libs/README.md) 放入有权使用的资源；缺少这些文件时，仍可修改前端源码，但完整桌面版打包和棋盘识别不可用。

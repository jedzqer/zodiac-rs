# 十二生肖 (Rust + Web)

基于中国传统十二生肖的棋盘游戏，使用 Rust 后端 + Web 前端架构重构自 Python/Pygame 版本。

## 项目状态

**当前阶段**: 核心功能已完成，可运行可测试

### 已完成

- [x] Rust 游戏逻辑引擎（12 种生肖棋子规则完整移植）
- [x] 6×4 棋盘数据结构，含猴占位机制
- [x] 启发式 AI 对手（评估函数：子力、位置、翻子收益）
- [x] Axum WebSocket 服务端（实时双向通信）
- [x] 前后端 JSON 协议定义（含 `move_count`、结构化 `AiAction` 坐标）
- [x] HTML5 Canvas 前端（棋盘渲染、棋子显示、点击交互）
- [x] 双人模式 (PVP) / 单人模式 (PVE)
- [x] 13 项单元测试全部通过
- [x] 翻子动画（Card-flip 效果，前半程显示背面，后半程显示正面）
- [x] 移动/吃子/消失动画（基于棋盘 diff 驱动，支持 Pong/Dog/Sheep 等特殊规则）
- [x] 选中棋子悬浮效果（`requestAnimationFrame` 循环，持续上浮 lift 偏移）
- [x] AI 动作动画（服务端 `AiAction` 携带结构化坐标，前端同步触发）
- [x] 猴占位机制修复（猴离开未翻开棋子后可正常移动；爬上时不再触发消失动画）
- [x] 选中棋子时面板显示该棋子技能说明
- [x] 信息面板右移至棋盘外侧，不再遮挡棋盘

### 待完成

- [ ] 神经网络 AI（需导出 ONNX 模型，集成 `ort` crate）
- [ ] 多房间/多人在线对战
- [ ] 断线重连
- [ ] 移动端触控适配
- [ ] 音效

## 快速开始

```bash
cd zodiac-rs
cargo run
```

浏览器打开 http://localhost:3000

可选环境变量：
```bash
PORT=8080 cargo run   # 自定义端口
```

启用 ONNX 神经网络 AI：
```bash
ZODIAC_AI_BACKEND=neural cargo run
```

若模型不在默认位置 `models/zooformer.onnx`，可显式指定：
```bash
ZODIAC_AI_BACKEND=neural \
ZODIAC_ONNX_MODEL=/abs/path/to/zooformer.onnx \
cargo run
```

导出 ONNX 模型：
```bash
python tools/export_zooformer_onnx.py \
  --checkpoint ../Python-old/checkpoints/zooformer.pt \
  --output models/zooformer.onnx
```

说明：导出脚本默认使用 ONNX opset `18`，这是当前 PyTorch 导出 ZooFormer 时更稳定的目标版本。若强制使用 `17`，PyTorch/ONNX 可能会在包含 `ScatterElements` 的图上降版本失败。

## 运行测试

```bash
cargo test
```

## 项目结构

```
zodiac-rs/
├── Cargo.toml
├── src/
│   ├── main.rs                 # 入口：启动 Axum 服务
│   ├── game/
│   │   ├── mod.rs              # GameState（回合管理、胜负判定）
│   │   ├── board.rs            # Board（6×4 棋盘、移动执行、13 项测试）
│   │   └── piece.rs            # 12 种生肖棋子 + MoveResult 枚举
│   ├── ai/
│   │   ├── mod.rs
│   │   └── heuristic.rs        # 启发式 AI（搜索 + 评估）
│   ├── server/
│   │   └── mod.rs              # Axum WebSocket handler + 游戏会话管理
│   └── protocol.rs             # ClientMessage / ServerMessage 定义
├── frontend/
│   └── index.html              # 单页应用（Canvas 渲染 + WebSocket 通信，无外部资源依赖）
└── Python-old/                 # 原 Python 版本（参考用）
```

## 技术栈

| 层 | 技术 |
|---|------|
| 后端框架 | Axum 0.8 + Tokio |
| 通信 | WebSocket (JSON) |
| 序列化 | Serde |
| 静态文件 | tower-http ServeDir |
| AI | 自研启发式搜索 |
| 前端 | HTML5 Canvas + 原生 JS |

## 架构说明

```
浏览器 (Canvas)  ←──WebSocket──→  Axum Server
   │                                  │
   ├─ 棋盘渲染                       ├─ GameState (Board + 规则引擎)
   ├─ 点击事件 → ClientMessage       ├─ AIPlayer (启发式)
   └─ ServerMessage → 更新画面       └─ 协议序列化/反序列化
```

- 前端仅负责渲染和用户输入，所有游戏逻辑在服务端
- 每次操作通过 WebSocket 发送 `ClientMessage`，服务端返回 `ServerMessage`
- PVE 模式下 AI 在服务端同步执行，前端收到结果后更新

## 对照原版

| 特性 | Python-old | zodiac-rs |
|------|-----------|-----------|
| 语言 | Python 3.8+ | Rust 2024 edition |
| UI | Pygame 桌面窗口 | HTML5 Canvas 浏览器 |
| 网络 | 无 | WebSocket |
| AI (基础) | ✅ 启发式 | ✅ 启发式 |
| AI (神经网络) | ✅ PyTorch ZooFormer | ❌ 待集成 ONNX |
| 动画 | ✅ Pygame 动画 | ✅ 翻子/移动/消失/悬浮动画 |
| AI 延迟 | — | 750ms（拟人感） |
| 棋子技能说明 | — | ✅ 选中棋子后面板显示 |
| 代码量 | ~1750 行 | ~1730 行 (Rust) + ~800 行 (HTML/JS) |

## 协议格式

客户端发送：
```json
{"type": "new_game", "data": {"mode": "pve"}}
{"type": "flip", "data": {"x": 2, "y": 1}}
{"type": "move", "data": {"from_x": 2, "from_y": 1, "to_x": 3, "to_y": 1}}
```

服务端返回：
```json
{"type": "game_started", "data": {"board": {...}, "current_player": "black", "mode": "pve"}}
{"type": "board_update", "data": {"board": {...}, "current_player": "red", "message": "..."}}
{"type": "game_over", "data": {"winner": "red", "board": {...}}}
{"type": "ai_thinking", "data": null}
{"type": "ai_action", "data": {"description": "AI 翻开了 (2, 1) 的棋子", "action_type": "flip", "from_x": 2, "from_y": 1, "to_x": null, "to_y": null}}
{"type": "error", "data": {"message": "Invalid move"}}
```

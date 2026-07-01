# Lumina

**AI 驱动的 Git 工作台** —— 用 AI 理解你的代码变更，一键生成规范的 Commit Message。

Lumina 是一款基于 Tauri 2 的轻量桌面应用，专为日常 Git 工作流设计。它帮你快速浏览变更、选择要提交的文件，然后用 AI 生成准确的 conventional commit 信息。

> [!NOTE]
> 项目当前为 v0.1.0，欢迎试用和反馈。

## 为什么需要 Lumina？

每次提交代码时，你可能面对这样的场景：改了十几个文件，手动写 commit message 要逐个回顾变更，还担心漏掉关键改动或误提交无关文件。

Lumina 解决三个问题：

1. **看不清** —— 变更太多，不知道改了什么 → 文件关注度评分帮你快速定位重点
2. **写不好** —— commit message 不够规范 → AI 基于你选中的文件精准生成
3. **怕误提交** —— 工作区有多个改动混在一起 → 只提交你勾选的文件

## 截图

<!-- TODO: 添加应用截图 -->

## 核心功能

### Git AI 助手

- 📂 选择并记住最近打开的 Git 仓库
- 🔍 一目了然的分支状态、暂存状态、冲突文件统计
- 📊 文件关注度评分，推荐优先检查的文件
- 🤖 勾选文件 → AI 生成 conventional commit message
- 🔎 Prompt 可视化：展示 AI 看到了什么、忽略了什么
- 💾 Commit message 历史，一键恢复

### 远端同步

- Fetch / Pull / Push，自动检测 ahead/behind 和分叉状态
- 冲突处理：标记 resolved、中止 merge/rebase
- 无 upstream 时自动设置追踪分支

### 项目控制台（DevDock）

- 多项目管理：添加、扫描、移除项目目录
- 读取 `package.json`，自动识别项目类型和包管理器
- 启动 / 停止 / 重启 dev server，查看实时日志（支持 ANSI 颜色）
- 一键打开浏览器访问运行端口

### AI 模型支持

- **Ollama**：本地模型，代码变更不出内网
- **OpenAI-compatible**：通义千问、OpenAI 及任何兼容 `/chat/completions` 的服务
- 不同任务可指定不同模型（commit message / change summary / light review）

## 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) 工具链
- [Git](https://git-scm.com/)
- [Tauri 2 系统依赖](https://v2.tauri.app/start/prerequisites/)

### 安装与运行

```bash
# 克隆项目
git clone <your-repo-url> lumina
cd lumina

# 安装依赖
npm install

# 启动桌面应用（开发模式）
npm run tauri:dev
```

### 构建安装包

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`。

> **Windows 提示**：如果构建时遇到 NSIS/WiX 工具下载失败，运行 `powershell -ExecutionPolicy Bypass -File fix-tauri-tools.ps1`。

## AI 模型配置

首次使用需要配置至少一个 AI 模型：

1. 打开应用 → 设置 → AI 模型管理
2. 添加模型：选择 Provider（Ollama 或 OpenAI-compatible），填写 Base URL 和模型名称
3. 点击"测试连接"确认可用
4. 为不同任务指定模型（可选，不指定则使用默认模型）

**Ollama 示例**：Base URL 填 `http://localhost:11434`，模型名填你在 Ollama 中拉取的模型（如 `qwen2.5`）。

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面容器 | Tauri 2 |
| 前端 | Vue 3 · TypeScript · Vite · Naive UI |
| 后端 | Rust · reqwest · tokio |
| AI | Ollama / OpenAI-compatible API |
| 样式 | SCSS 设计令牌体系，明暗主题切换 |

## 项目结构

```
Lumina/
├── src/                          # Vue 前端
│   ├── views/git-assistant/      # Git AI 助手（核心功能）
│   ├── views/devdock/            # 项目控制台
│   ├── views/settings/           # 设置
│   ├── services/git/             # Tauri 命令调用封装
│   ├── components/               # 通用组件
│   ├── stores/                   # Pinia 状态管理
│   ├── styles/                   # 设计令牌与主题
│   └── i18n/                     # 中/英双语
├── src-tauri/                    # Rust 后端
│   └── src/
│       ├── commands/             # Tauri command 暴露层
│       ├── git/                  # Git 核心逻辑
│       │   ├── runner.rs         # Git 命令执行
│       │   ├── prompt.rs         # AI Prompt 构建引擎
│       │   └── analyzer.rs       # 文件分析与分类
│       └── ai/                   # AI 模型调用
├── public/                       # 静态资源
└── package.json
```

## 开发命令

```bash
npm run tauri:dev        # 启动桌面应用（开发模式）
npm run dev              # 仅启动前端 Vite
npm run tauri:build      # 构建桌面应用
npm run build            # 构建前端产物
npm run lint             # 代码检查
npm run lint:css         # 样式检查
```

## 本地数据

应用数据存储在两个位置：

- **浏览器 localStorage**：仓库列表、commit 历史、AI 模型配置、主题偏好
- **项目内 `.lumina/` 目录**：项目画像（`git-profile.json`）、Prompt 调试文件

`.lumina/` 已被 `.gitignore` 忽略，不会污染你的仓库。

## License

<!-- TODO: 添加 License -->

# Lumina

**中文** | [English](#english)

Lumina 是一个面向日常开发的桌面工作台：在同一个应用中管理 Git 变更、生成提交说明、运行多个本地项目，并查看进程与日志。

> 当前版本：`0.1.0-beta.1`。这是一个公开测试版本，欢迎提交 Issue 和反馈。

## 功能

### Git 工作台

- 浏览工作区变更、暂存状态、冲突文件和分支同步状态。
- 选择部分文件提交，同时保留未选择文件原有的暂存状态。
- 支持 Fetch、Pull、Push、Rebase、冲突标记和单文件 Revert。
- 查看文件 Diff、提交历史与 Git 命令实时进度。
- 记录最近项目，并支持为项目设置本地别名。

### AI Commit Assistant

- 基于选中文件生成 conventional commit 标题和正文。
- 查看生成 Prompt、文件筛选策略与提交历史。
- 支持 Ollama 及 OpenAI-compatible API，并可按任务选择模型。

### DevDock

- 管理多个本地项目，解析 `package.json` 中的 scripts。
- 启动、停止、重启开发进程，查看 PID、端口、URL 和日志。
- 支持命令搜索、置顶、最近命令和浏览器快速打开。
- 退出应用时可选择最小化到系统托盘，或停止全部受管进程后退出。

## 快速开始

### 环境要求

- Node.js 20+
- Rust 1.77.2+
- Git
- [Tauri 2 系统依赖](https://v2.tauri.app/start/prerequisites/)

### 本地运行

```bash
git clone https://github.com/<your-github-id>/lumina.git
cd lumina
npm install
npm run tauri:dev
```

### 构建安装包

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`。Windows 下如果 NSIS 或 WiX 下载失败，可运行：

```powershell
powershell -ExecutionPolicy Bypass -File fix-tauri-tools.ps1
```

## AI 模型配置

打开 **设置 → AI 模型管理**，添加模型后先测试连接，再按需指定给 Commit Message、Change Summary 或 Light Review。

- **Ollama**：Base URL 例如 `http://localhost:11434`，适合完全本地的工作流。
- **OpenAI-compatible**：填写服务地址、模型名和 API Key，兼容支持 `/chat/completions` 的服务。

## 数据与安全边界

- AI 模型配置、最近仓库、主题和提交历史保存在本机应用数据中。
- 项目画像和 Prompt 调试数据写入项目内的 `.lumina/`，该目录已被忽略，不会被 Lumina 自动提交。
- DevDock 会执行你选择项目中的 `package.json` scripts。只添加和运行你信任的项目目录。
- 当使用远程 AI 服务时，选中的代码变更和生成 Prompt 会发送给你配置的服务提供方。

## 开发

```bash
npm run tauri:dev   # 启动桌面应用
npm run dev         # 仅启动 Vite
npm run lint        # TypeScript、ESLint、Stylelint
npm run build       # 构建前端
npm run tauri:build # 构建桌面安装包
```

## 项目结构

```text
src/                         Vue 前端
  views/git-assistant/       Git 工作台
  views/devdock/             多项目进程管理
  views/git-log/             提交历史窗口
  components/workbench/      共享工作台组件
  services/                  Tauri 命令调用封装
src-tauri/                   Rust / Tauri 后端
  commands/                  Tauri command 层
  git/                       Git 操作与 Prompt 构建
  ai/                        AI 服务调用
```

## 贡献

欢迎提交 Issue 和 Pull Request。提交前请运行：

```bash
npm run lint
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

## 许可证

[MIT License](LICENSE)

---

<a id="english"></a>

# Lumina

[中文](#lumina) | **English**

Lumina is a desktop workbench for everyday development: manage Git changes, generate commit messages, run multiple local projects, and inspect processes and logs from one application.

> Current version: `0.1.0-beta.1`. This is a public beta. Issues and feedback are welcome.

## Features

### Git Workbench

- Inspect working-tree changes, staged files, conflicts, branches, and sync status.
- Commit selected files without changing the staged state of files that were not selected.
- Fetch, pull, push, rebase, conflict resolution, and per-file revert.
- File diffs, commit history, and live Git command progress.
- Recent repositories with local aliases.

### AI Commit Assistant

- Generate conventional commit titles and bodies from selected files.
- Review generated prompts, file-selection strategy, and commit-message history.
- Use Ollama or OpenAI-compatible APIs, with model routing per task.

### DevDock

- Manage multiple local projects and parse scripts from `package.json`.
- Start, stop, and restart development processes; inspect PID, ports, URLs, and logs.
- Search and pin commands, reopen recent commands, and open running services in a browser.
- Minimize to the system tray or stop all managed processes before quitting.

## Getting Started

### Prerequisites

- Node.js 20+
- Rust 1.77.2+
- Git
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)

### Run Locally

```bash
git clone https://github.com/<your-github-id>/lumina.git
cd lumina
npm install
npm run tauri:dev
```

### Build an Installer

```bash
npm run tauri:build
```

Artifacts are written to `src-tauri/target/release/bundle/`. On Windows, if NSIS or WiX downloads fail, run:

```powershell
powershell -ExecutionPolicy Bypass -File fix-tauri-tools.ps1
```

## AI Model Setup

Open **Settings → AI Models**, add a model, test the connection, then optionally assign it to Commit Message, Change Summary, or Light Review.

- **Ollama**: use a Base URL such as `http://localhost:11434` for a local workflow.
- **OpenAI-compatible**: provide the service URL, model name, and API key. Services exposing `/chat/completions` are supported.

## Data and Security

- AI configuration, recent repositories, themes, and commit history stay in local application data.
- Project profiles and prompt-debug data are stored in `.lumina/` within each project. The directory is ignored by Git.
- DevDock executes scripts from the selected project's `package.json`. Only add projects you trust.
- When using a remote AI provider, selected code changes and generated prompts are sent to the provider you configure.

## Development

```bash
npm run tauri:dev   # Run the desktop app
npm run dev         # Run Vite only
npm run lint        # TypeScript, ESLint, and Stylelint
npm run build       # Build the frontend
npm run tauri:build # Build desktop installers
```

## Project Layout

```text
src/                         Vue frontend
  views/git-assistant/       Git workbench
  views/devdock/             Multi-project process management
  views/git-log/             Commit-history window
  components/workbench/      Shared workbench components
  services/                  Tauri command wrappers
src-tauri/                   Rust / Tauri backend
  commands/                  Tauri command layer
  git/                       Git operations and prompt construction
  ai/                        AI service clients
```

## Contributing

Issues and pull requests are welcome. Before opening a pull request, run:

```bash
npm run lint
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

## License

[MIT License](LICENSE)

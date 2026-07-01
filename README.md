# Lumina

Lumina 是一个基于 Tauri 2、Vue 3 和 Rust 的桌面端 Git AI 助手。它把常见 Git 工作流、变更查看、AI Commit Message 生成、远端同步和冲突处理集中在一个本地应用里，适合在开发过程中快速理解改动、选择文件并生成更准确的提交说明。

## 核心能力

- 选择并记住最近打开的 Git 仓库。
- 展示当前分支、工作区状态、暂存状态、未跟踪文件和冲突文件。
- 按文件查看 staged、unstaged 和上一版本 Diff。
- 根据文件类型、路径和变更状态计算关注分数，并推荐优先检查的文件。
- 勾选指定文件生成 AI Commit Message。
- 查看 AI Prompt 构建过程，包括文件分类、证据行数、清洗前后字符数和省略原因。
- 保存最近生成或手写的 Commit Message 历史。
- 只提交用户勾选的文件，避免误提交其他工作区改动。
- 支持 fetch、pull、push，并提示 remote、upstream、ahead、behind 和分叉状态。
- 支持配置 origin、修复 upstream、发布当前分支。
- 支持冲突文件标记 resolved 和中止 merge。
- 支持查看 Git Log、提交详情、提交内文件列表和单文件提交 Diff。
- 支持 Ollama 和 OpenAI-compatible 模型接口。

## 技术栈

- 桌面容器：Tauri 2
- 前端：Vue 3、TypeScript、Vite
- UI：Naive UI、Iconify
- 状态管理：Pinia
- 国际化：vue-i18n
- Diff 渲染：diff2html
- 后端：Rust
- Git 操作：Rust 侧调用本机 `git` 命令
- AI 请求：Rust 侧通过 `reqwest` 调用本地或远程模型服务

## 目录结构

```text
.
├── src/                              # Vue 前端
│   ├── main.ts                       # 前端入口
│   ├── router/                       # 路由，默认进入 Git Assistant
│   ├── views/git-assistant/          # Git 助手主界面
│   ├── services/git/                 # Tauri Git/AI 命令调用封装
│   ├── stores/                       # Pinia 状态
│   ├── i18n/                         # 中英文文案
│   └── styles/                       # 主题、token、基础样式
├── src-tauri/                        # Tauri/Rust 后端
│   ├── src/commands/                 # Tauri command 暴露层
│   ├── src/git/                      # Git 状态、Diff、提交、Prompt、项目画像
│   ├── src/ai/                       # Ollama 和 OpenAI-compatible 调用
│   └── tauri.conf.json               # Tauri 配置
├── public/                           # 静态资源
├── package.json                      # 前端脚本和依赖
└── README.md
```

## 运行环境

本地需要安装：

- Node.js 20.19.3 或兼容版本
- npm
- Rust 工具链
- Tauri 2 所需系统依赖
- Git

如果需要使用 AI 生成能力，还需要至少配置一种模型服务：

- Ollama，例如 `http://localhost:11434`
- OpenAI-compatible API，例如通义千问、OpenAI 兼容网关或其他兼容 `/chat/completions` 的服务

## 安装依赖

```bash
npm install
```

## 开发运行

只启动前端 Vite：

```bash
npm run dev
```

启动完整 Tauri 桌面应用：

```bash
npm run tauri:dev
```

## 构建

构建前端产物：

```bash
npm run build
```

构建桌面应用：

```bash
npm run tauri:build
```

## 验证命令

前端类型检查和构建：

```bash
npm run build
```

Rust 测试：

```bash
cd src-tauri
cargo test
```

样式检查：

```bash
npm run lint:css
```

完整 lint：

```bash
npm run lint
```

## AI 模型配置

在应用设置中可以维护模型配置，并为不同任务指定模型。目前任务包括：

- Commit Message
- Change Summary
- Light Review

模型配置存储在浏览器 localStorage 中，key 为 `lumina.ai.settings.v1`。每个模型包含：

- 名称
- Provider：`ollama` 或 `openai-compatible`
- Base URL
- API Key，可选
- 模型名称
- 是否启用

连接测试会调用：

- Ollama：`GET /api/tags`
- OpenAI-compatible：`GET /models`

生成 Commit Message 时，后端会要求模型返回结构化 JSON：

```json
{
  "title": "feat(scope): concise English title",
  "body": "- English bullet line",
  "summary": "中文摘要",
  "risks": ["中文风险提示"]
}
```

## Git 提交流程

Lumina 的提交逻辑以“用户勾选的文件”为准：

1. 前端加载 Git 快照并展示所有变更。
2. 用户勾选要纳入本次提交的文件。
3. AI Prompt 只基于选中文件构建。
4. 用户确认或编辑 title/body。
5. Rust 后端先清空 index。
6. Rust 后端只 stage 选中文件。
7. Rust 后端执行 `git commit`。

这种方式可以避免当前工作区存在其他改动时被一起提交。

## Prompt 构建逻辑

Prompt 构建位于 `src-tauri/src/git/prompt.rs`，主要做这些事：

- 根据 `.lumina/git-profile.json` 或默认规则识别文件角色和 scope。
- 合并 staged 与 unstaged diff。
- 对 untracked 文件使用 `git diff --no-index` 生成类似新增文件的 diff。
- 跳过 lockfile、资产文件和内部调试文件等低价值内容。
- 对大文件、新文件、删除和重命名做结构化摘要。
- 对普通 diff 选择高信息量证据行。
- 对疑似密钥、token、password 等敏感片段做脱敏。
- 按文件组和总字符预算压缩 prompt。
- 写入 `.lumina/commit-prompt-debug.json` 供调试查看。

`.lumina` 默认在 `.gitignore` 中被忽略，不应提交到仓库。

## 项目画像

首次打开仓库时，后端会确保存在 `.lumina/git-profile.json`。它用于描述当前仓库的：

- 项目类型
- 语言和文件扩展名
- scope 匹配规则
- primary、secondary、generated、tooling 文件角色
- commit 风格
- review 权重

如果项目结构特殊，可以在应用设置中调整项目画像，让 AI Prompt 更贴合真实业务结构。

## 远端同步策略

Lumina 会读取：

- 是否已有提交
- 是否存在 `origin`
- 当前分支 upstream
- upstream 是否已消失
- ahead/behind 数量

Push 前会先执行 `fetch --prune`。如果远端有本地没有的提交，或者本地和远端已经分叉，应用会阻止直接 push，并提示先 pull 或处理分叉。

当当前分支没有 upstream 时，push 会使用：

```bash
git push -u origin <branch>
```

## 本地数据

应用会在 localStorage 保存部分本地偏好：

- 最近打开的仓库
- Commit Message 历史
- AI 模型配置
- 主题、语言等偏好

仓库内的 Lumina 调试和项目画像文件保存在 `.lumina/`，该目录已被 `.gitignore` 忽略。

## 网络和代理

Rust 后端运行 Git 命令时会处理代理环境：

- 优先继承 `HTTP_PROXY`、`HTTPS_PROXY`、`ALL_PROXY` 等环境变量。
- Windows 下会尝试读取系统代理。
- 对 localhost、局域网 IP、`.local`、无点号主机名和 `NO_PROXY` 命中的 host 会绕过代理。

这部分主要用于减少桌面应用中 Git 访问远端时代理环境不一致的问题。

## 当前注意事项

- 浏览器端 UI 自动化测试未配置。
- README 描述的是当前代码中已存在的能力，不包含未来规划功能。
- Windows 下运行 `cargo test` 时可能出现增量编译目录 `拒绝访问` warning；如果测试结果为 passed，通常不影响功能验证。
- `dist/`、`node_modules/`、`.lumina/` 都不应提交。


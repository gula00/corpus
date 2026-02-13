## Antigravity-Manager

这是一个专业的 AI 账号管理与协议反代系统，基于 Tauri v2 框架构建的桌面应用。

### 项目概览

- 项目名称: Antigravity Tools
- 当前版本: 4.1.12
- 技术栈: Tauri v2 + React + Rust + TypeScript
- 主要功能: AI 账号管理、协议转换、智能请求调度

### 目录结构

```
Antigravity-Manager/
├── src/                    # React 前端源码
│   ├── App.tsx            # 主应用组件
│   ├── components/        # React 组件
│   ├── pages/             # 页面组件
│   ├── stores/            # 状态管理 (Zustand)
│   ├── services/          # API 服务层
│   ├── hooks/             # 自定义 React Hooks
│   ├── utils/             # 工具函数
│   ├── types/             # TypeScript 类型定义
│   ├── locales/           # 国际化语言文件
│   ├── config/            # 配置文件
│   └── i18n.ts            # 国际化配置
│
├── src-tauri/             # Rust 后端源码
│   ├── src/               # Rust 源代码
│   ├── Cargo.toml         # Rust 依赖配置
│   ├── tauri.conf.json    # Tauri 应用配置
│   ├── icons/             # 应用图标
│   └── capabilities/      # 权限配置
│
├── public/                # 静态资源
├── docs/                  # 项目文档
├── scripts/               # 构建和部署脚本
├── docker/                # Docker 相关配置
└── dist/                  # 构建输出目录
```

### 技术栈详情

**前端技术:**

- 框架: React 19.1.0
- 路由: React Router 7.x
- 状态管理: Zustand 5.0.9
- UI 框架:
  - Ant Design 5.24.6
  - @lobehub/ui 4.33.4
  - DaisyUI 5.5.13
- 样式: Tailwind CSS 3.4.19 + Emotion
- 动画: Framer Motion 11.13.1
- 国际化: i18next 25.7.2
- 图表: Recharts 3.5.1
- 拖拽: @dnd-kit
- 构建工具: Vite 7.0.4

**后端技术:**

- 框架: Tauri v2
- 语言: Rust
- 插件:
  - 自动启动 (autostart)
  - 文件系统 (fs)
  - 对话框 (dialog)
  - 自动更新 (updater)

### 核心功能模块

1. **智能账号仪表盘** - 实时监控 AI 账号配额
2. **账号管理** - OAuth 2.0 授权、批量导入、网格视图
3. **协议转换** - 支持 OpenAI/Anthropic/Gemini 多种格式
4. **模型路由** - 智能分级路由、自动降级
5. **多模态支持** - Imagen 3 绘图支持

### 关键配置文件

- `package.json` - Node.js 依赖和脚本
- `src-tauri/Cargo.toml` - Rust 依赖
- `src-tauri/tauri.conf.json` - Tauri 应用配置
- `vite.config.ts` - Vite 构建配置
- `tailwind.config.js` - Tailwind CSS 配置
- `tsconfig.json` - TypeScript 配置

### 开发脚本

```bash
"dev": "vite"                                      # 开发模式
"build": "tsc && vite build"                       # 构建前端
"tauri": "tauri"                                   # Tauri CLI
"tauri:debug": "RUST_LOG=debug npm run tauri dev"  # 调试模式
```

### 项目特点

1. **跨平台**: 基于 Tauri，支持 Windows/macOS/Linux
2. **高性能**: Rust 后端保证执行效率
3. **现代化**: 使用最新的 React 19 和 TypeScript 5.8
4. **国际化**: 内置多语言支持
5. **可部署**: 提供 Docker 配置和部署脚本

这是一个企业级的桌面应用项目，架构清晰、技术栈现代化，专注于 AI 服务的本地化管理和协议转换。

---

### 反代理功能架构

反代理功能分为前端配置界面和后端实现两部分：

**前端 - 配置界面**

主要文件: `src/pages/ApiProxy.tsx` (178KB)，用户在此进行反代理设置。

**后端 - Rust 核心实现**

核心目录: `src-tauri/src/proxy/`

```
src-tauri/src/proxy/
├── mod.rs                    # 模块入口
├── server.rs                 # HTTP 服务器
├── config.rs                 # 代理配置
├── proxy_pool.rs             # 代理池管理
├── token_manager.rs          # Token 管理
├── session_manager.rs        # 会话管理
├── signature_cache.rs        # 签名缓存
├── monitor.rs                # 监控模块
│
├── handlers/                 # 请求处理器
│   ├── openai.rs             # OpenAI API 处理
│   ├── claude.rs             # Claude/Anthropic API 处理
│   ├── gemini.rs             # Google Gemini API 处理
│   ├── audio.rs              # 音频处理
│   ├── mcp.rs                # MCP 协议处理
│   ├── warmup.rs             # 预热处理
│   └── common.rs             # 通用处理
│
├── mappers/                  # 协议转换器
│   ├── openai/               # OpenAI 格式转换
│   │   ├── request.rs
│   │   └── streaming.rs
│   ├── claude/               # Claude 格式转换
│   │   ├── request.rs
│   │   ├── response.rs
│   │   ├── streaming.rs
│   │   └── thinking_utils.rs
│   ├── gemini/               # Gemini 格式转换
│   │   ├── collector.rs
│   │   └── wrapper.rs
│   └── error_classifier.rs   # 错误分类
│
├── middleware/               # 中间件
│   ├── auth.rs               # 认证中间件
│   ├── ip_filter.rs          # IP 过滤
│   ├── monitor.rs            # 监控中间件
│   └── service_status.rs     # 服务状态检查
│
├── upstream/                 # 上游客户端
│   └── client.rs             # 上游 API 客户端
│
└── tests/                    # 测试
    ├── comprehensive.rs      # 综合测试
    ├── quota_protection.rs   # 配额保护测试
    └── ultra_priority_tests.rs # Ultra 优先级测试
```

**命令层:** `src-tauri/src/commands/proxy.rs` - 提供前端调用的 Tauri 命令接口

**数据库层:** `src-tauri/src/modules/proxy_db.rs` - 反代理相关的数据库操作

**核心工作流程:**

1. 前端 (`ApiProxy.tsx`) → 用户配置代理参数
2. 命令层 (`commands/proxy.rs`) → 接收前端请求
3. 服务器 (`proxy/server.rs`) → 启动 HTTP 服务
4. 处理器 (`proxy/handlers/*`) → 根据不同 API 类型分发请求
5. 转换器 (`proxy/mappers/*`) → 协议格式转换
6. 上游客户端 (`proxy/upstream/client.rs`) → 调用真实的 AI API

支持 OpenAI、Anthropic、Gemini 三种协议格式的互相转换。

---

## wxdump

### macOS 流程

**步骤 1: 手动获取密钥**

```bash
# 1. 打开微信（不登录）
# 2. 附加调试器
lldb -p $(pgrep WeChat)

# 3. 设置断点
br set -n sqlite3_key

# 4. 继续执行
c

# 5. 扫码登录微信（会卡住）

# 6. 回到终端，读取密钥
memory read --size 1 --format x --count 32 $rsi

# 7. 会输出类似：
# 0x60000241e920: 0xc2 0xf9 0x13 0xbe 0xda 0xe8 0x45 0x82
# 0x60000241e928: 0x93 0x94 0x5b 0xbf 0x61 0x86 0xd9 0x7f
# ...
```

**步骤 2: 转换密钥**

```python
ori_key = """
0x60000241e920: 0xc2 0xf9 0x13 0xbe 0xda 0xe8 0x45 0x82
0x60000241e928: 0x93 0x94 0x5b 0xbf 0x61 0x86 0xd9 0x7f
0x60000241e930: 0xab 0xd3 0x0e 0xf0 0x39 0xcf 0x4c 0xba
0x60000241e938: 0x99 0x3a 0x01 0x05 0x2f 0x75 0x2d 0xcd
"""

key = '0x' + ''.join(i.partition(':')[2].replace('0x', '').replace(' ', '') for i in ori_key.split('\n')[1:5])
print(key)
# 输出: 0xc2f913bedae845829394...（完整64位）
```

**步骤 3: 找到数据库文件**

```bash
find ~/Library/Containers/com.tencent.xinWeChat -name "*.db" -type f
```

**步骤 4: 解密数据库**

```bash
# 安装 PyWxDump
pip install pywxdump

# 解密单个数据库
wxdump decrypt \
  -k "你的64位密钥" \
  -i "~/Library/Containers/com.tencent.xinWeChat/.../MSG0.db" \
  -o "./decrypted/"

# 或批量解密整个目录
wxdump decrypt \
  -k "你的64位密钥" \
  -i "~/Library/Containers/com.tencent.xinWeChat/.../Message/" \
  -o "./decrypted/"
```

**步骤 5: 获得可用的 SQLite 数据库**

```bash
ls ./decrypted/
# 输出：
# de_MSG0.db
# de_MSG1.db
# de_MicroMsg.db
# ...

# 现在可以用任何 SQLite 工具打开了
sqlite3 ./decrypted/de_MSG0.db
```

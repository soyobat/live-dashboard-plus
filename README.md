# Live Dashboard

实时设备活动监控面板，公开展示我正在用什么 app、看什么内容。

支持 Windows 和 Android（Magisk）双端上报，前端二次元风格设计。

## 架构

```
┌─────────────┐     HTTPS POST      ┌──────────────┐     静态托管      ┌────────────┐
│ Windows Agent│ ──────────────────→ │   Bun 后端    │ ←─────────────── │  Next.js   │
│  (Python)    │                     │  + SQLite     │                  │  (静态导出) │
└─────────────┘                     └──────┬───────┘                  └────────────┘
                                           ↑
┌─────────────┐     HTTPS POST             │
│ Android Agent│ ──────────────────────────┘
│ (Magisk 模块) │
└─────────────┘
```

- **通信方式**：HTTPS POST 轮询（短连接，GFW 友好）
- **数据存储**：SQLite（bun:sqlite，零依赖）
- **前端刷新**：10 分钟轮询
- **数据保留**：7 天自动清理

## 技术栈

| 组件 | 技术 |
|------|------|
| 后端 | Bun + TypeScript + SQLite |
| 前端 | Next.js 15 + React 19 + Tailwind CSS 4（静态导出） |
| Windows Agent | Python + ctypes Win32 API + psutil |
| Android Agent | Magisk 模块 + shell 脚本（dumpsys） |
| 部署 | Docker + Nginx 反代 |

## 项目结构

```
live-dashboard/
├── packages/
│   ├── backend/                # Bun 后端
│   │   └── src/
│   │       ├── index.ts        # HTTP 服务入口 + 静态文件托管
│   │       ├── db.ts           # SQLite 建表 + 查询
│   │       ├── types.ts        # 类型定义
│   │       ├── routes/         # API 路由
│   │       │   ├── report.ts   # POST /api/report（Agent 上报）
│   │       │   ├── current.ts  # GET /api/current（当前状态）
│   │       │   ├── timeline.ts # GET /api/timeline（时间线）
│   │       │   └── health.ts   # GET /api/health
│   │       ├── middleware/
│   │       │   └── auth.ts     # Bearer token 鉴权
│   │       ├── services/
│   │       │   ├── nsfw-filter.ts  # NSFW 内容过滤（命中则丢弃）
│   │       │   ├── app-mapper.ts   # 包名/进程名 → 可读名映射
│   │       │   └── cleanup.ts     # 7天数据清理 + 离线检测
│   │       └── data/
│   │           ├── app-names.json      # App 名称字典
│   │           └── nsfw-blocklist.json # NSFW 黑名单
│   │
│   └── frontend/               # Next.js 前端
│       ├── app/                # 页面
│       └── src/
│           ├── components/     # UI 组件
│           ├── hooks/          # useDashboard 轮询 hook
│           └── lib/            # API 客户端
│
├── agents/
│   ├── windows/                # Windows Agent
│   │   ├── agent.py            # 主程序
│   │   ├── config.json         # 配置（gitignored）
│   │   ├── build.bat           # PyInstaller 打包
│   │   └── install-task.bat    # 注册开机自启
│   │
│   └── android/                # Magisk 模块
│       ├── service.sh          # 主循环脚本
│       ├── config.sh           # 配置（gitignored）
│       ├── module.prop         # 模块信息
│       └── META-INF/           # Magisk 安装脚本
│
├── deploy/nginx/               # Nginx 配置参考
├── Dockerfile                  # 多阶段构建
├── docker-compose.yml          # 容器编排
└── .env.example                # 环境变量模板
```

## 本地开发

### 前置要求

- [Bun](https://bun.sh/) v1+
- Node.js 18+（仅前端构建需要）

### 启动后端

```bash
cd packages/backend

# 配置环境变量
cp ../../.env.example .env
# 编辑 .env，设置设备 token

# 安装依赖
bun install

# 启动（默认 3000 端口）
bun run src/index.ts
```

### 构建前端

```bash
cd packages/frontend
bun install
bun run build

# 把构建产物复制到后端的 public 目录
cp -r out/ ../backend/public/
```

然后访问 `http://localhost:3000`。

## 环境变量

| 变量 | 说明 | 示例 |
|------|------|------|
| `DEVICE_TOKEN_N` | 设备令牌，格式 `token:device_id:name:platform` | `abc123:my-pc:My PC:windows` |
| `PORT` | 监听端口 | `3000` |
| `STATIC_DIR` | 前端静态文件目录 | `./public` |

支持多个设备，递增编号即可：`DEVICE_TOKEN_1`、`DEVICE_TOKEN_2`、`DEVICE_TOKEN_3`...

## API

| 方法 | 路径 | 说明 | 鉴权 |
|------|------|------|------|
| POST | `/api/report` | Agent 上报当前 app | Bearer token |
| GET | `/api/current` | 获取所有设备当前状态 | 无 |
| GET | `/api/timeline?date=YYYY-MM-DD` | 获取指定日期时间线 | 无 |
| GET | `/api/health` | 健康检查 | 无 |

### 上报请求体

```json
{
  "app_id": "chrome.exe",
  "window_title": "GitHub - live-dashboard",
  "timestamp": 1741866000000
}
```

## Agent 部署

### Windows

1. 安装 Python 3.8+ 和依赖：`pip install -r agents/windows/requirements.txt`
2. 复制 `config.json`，填入服务器地址和 token
3. 直接运行：`python agent.py`
4. 或打包成 exe：运行 `build.bat`，然后用 `install-task.bat` 注册开机自启

### Android（Magisk）

1. 编辑 `agents/android/config.sh`，填入服务器地址和 token
2. 将整个 `android/` 目录打包成 zip：`cd agents && zip -r live-dashboard-magisk.zip android/`
3. 在 Magisk 中刷入该 zip
4. 重启后自动运行，日志在模块目录下的 `agent.log`

## Docker 部署

```bash
# 配置环境变量
cp .env.example .env
# 编辑 .env，填入真实 token

# 构建并启动
docker compose up -d --build

# 查看日志
docker logs live_dashboard --tail 50
```

## 安全设计

- **设备鉴权**：每设备独立 token，通过环境变量注入，不落盘
- **身份绑定**：token 反查 device_id，忽略请求体中的 device_id，防伪造
- **NSFW 过滤**：服务端黑名单匹配，命中的记录直接丢弃不存库
- **去重**：SQLite 唯一约束 `(device_id, app_id, title_hash, time_bucket)` + `ON CONFLICT DO NOTHING`
- **路径穿越防护**：normalize + relative 检查 + realpath 符号链接校验
- **HTTPS 强制**：Agent 端校验 HTTPS，拒绝明文传输 token
- **限流**：Nginx 层 `limit_req` 限制上报频率
- **XSS 防护**：React JSX 默认转义，`Content-Type: application/json`，`X-Content-Type-Options: nosniff`

## 许可证

MIT

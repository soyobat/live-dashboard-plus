# Turso 数据库部署指南

## ✅ 已完成的适配

项目已完成 Turso (libSQL) 数据库适配，支持：
- ✅ 云端持久化存储（不再担心数据丢失）
- ✅ 自动部署时数据保留
- ✅ Render/任何云平台部署
- ✅ 本地开发降级使用 SQLite 文件

## 🚀 部署步骤

### 1. 安装依赖

```bash
cd packages/backend
bun install
```

### 2. 配置环境变量

在 Render Dashboard 中添加以下环境变量：

```bash
# 必需：设备 Token（根据实际需求添加）
DEVICE_TOKEN_1=your_token1:device-id-1:Device Name:windows

# 必需：HMAC 密钥（用于隐私保护）
HASH_SECRET=生成一个随机字符串 # openssl rand -hex 32

# 必需：Turso 数据库配置
TURSO_DATABASE_URL=libsql://live-dashboard-db-soyo.aws-us-east-1.turso.io
TURSO_AUTH_TOKEN=你的_turso_auth_token

# 可选：站点配置
DISPLAY_NAME=Your Name
SITE_TITLE=Your Dashboard
```

### 3. 初始化数据库

第一次部署时，数据库表会自动创建。

如果需要手动初始化或迁移，可以运行：

```bash
cd packages/backend
bun run src/db.ts
```

### 4. 验证部署

访问 Render 提供的 URL，检查：
- [ ] 前端页面正常加载
- [ ] API 接口返回数据
- [ ] Agent 能正常上报数据
- [ ] 历史数据不会丢失

## 📊 数据库架构

### 核心表

1. **activities** - 活动记录（7 天自动清理）
   - 用户应用使用历史
   - 包含设备信息、应用 ID、显示标题等

2. **device_states** - 设备状态
   - 当前在线设备列表
   - 最后活动时间、电池状态等

3. **health_records** - 健康数据
   - 心率、步数、睡眠等健康指标

### 数据保留策略

- activities 表：自动删除 7 天前的记录
- device_states 表：永久保留（除非手动清理）
- health_records 表：永久保留（除非手动清理）

## 🔧 本地开发

本地开发时，如果不设置 `TURSO_DATABASE_URL`，会自动降级使用本地 SQLite 文件：

```bash
# 本地 .env 文件（可选）
HASH_SECRET=local_dev_secret
# 不设置 TURSO_* 变量，使用本地 SQLite
```

## ⚠️ 注意事项

1. **HASH_SECRET 必须设置**
   - 用于窗口标题的 HMAC 哈希
   - 防止通过彩虹表攻击还原隐私数据
   - 生产环境务必使用强随机值

2. **Turso 免费额度**
   - 每月 9GB 存储
   - 每月 500 万次读取操作
   - 对于个人使用完全足够

3. **数据备份**
   - Turso 自带备份和恢复功能
   - 可以通过 Turso CLI 导出数据
   - 建议定期导出重要数据

## 🛠️ 故障排查

### 数据库连接失败

检查日志中的错误信息：
```
[db] Connecting to Turso database: libsql://***.turso.io
```

如果看到连接错误：
1. 确认 `TURSO_DATABASE_URL` 和 `TURSO_AUTH_TOKEN` 正确
2. 检查 Turso 数据库是否正常运行
3. 验证网络连通性

### 表不存在

如果是首次部署，数据库表会自动创建。

如果需要手动创建：
```bash
cd packages/backend
bun run -e "import './src/db.ts'"
```

### 数据查询慢

Turso 是 HTTP-based 数据库，延迟比本地 SQLite 高。

优化建议：
- 确保查询都有索引
- 减少不必要的查询
- 考虑添加缓存层

## 📝 更新日志

**2026-03-29**
- ✅ 完成 Turso 数据库适配
- ✅ 所有数据库操作改为异步
- ✅ 更新所有路由处理器
- ✅ 添加环境变量配置

---

**技术支持**：如有问题，请查看 Render 日志或提交 Issue。

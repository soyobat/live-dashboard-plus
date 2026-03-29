# 🚀 Render 部署清单

## ✅ 已完成的工作

### 代码适配
- [x] Turso (libSQL) 数据库客户端集成
- [x] 所有数据库操作改为异步
- [x] 路由处理器更新（report, current, timeline, health-data, health-webhook）
- [x] 定时任务更新（数据清理、设备离线检测）
- [x] 本地开发降级支持（无 Turso 配置时使用 SQLite 文件）

### 依赖安装
- [x] @libsql/client@0.14.0 已安装

### 配置文件
- [x] `.env.example` 已更新 Turso 配置说明
- [x] `render.yaml` 部署配置已创建
- [x] `DEPLOYMENT.md` 部署指南已创建
- [x] `test-db.ts` 数据库测试脚本已创建

---

## 📋 部署步骤（按顺序执行）

### 第 1 步：获取 Turso 认证信息

你已经注册了 Turso，现在需要获取认证 token：

```bash
# 如果还没安装 Turso CLI，先安装
curl -sSfL https://get.tur.so/install.sh | bash

# 登录 Turso
turso auth login

# 查看你的数据库
turso db show live-dashboard-db-soyo

# 如果没有 token，创建一个新的
turso db tokens create live-dashboard-db-soyo
```

**记录以下信息：**
- ✅ **TURSO_DATABASE_URL**: `libsql://live-dashboard-db-soyo.aws-us-east-1.turso.io`
- 🔑 **TURSO_AUTH_TOKEN**: （运行上面命令获取）

---

### 第 2 步：生成安全密钥

```bash
# 生成 HMAC 密钥（用于隐私保护）
openssl rand -hex 32
```

**记录生成的密钥**，例如：`a1b2c3d4e5f6...`

---

### 第 3 步：在 Render 设置环境变量

登录 [Render Dashboard](https://dashboard.render.com)，进入你的服务页面：

1. 点击 **"Environment"** 标签
2. 添加以下环境变量：

| Key | Value | 说明 |
|-----|-------|------|
| `HASH_SECRET` | （第 2 步生成的密钥） | HMAC 哈希密钥 |
| `TURSO_DATABASE_URL` | `libsql://live-dashboard-db-soyo.aws-us-east-1.turso.io` | Turso 数据库 URL |
| `TURSO_AUTH_TOKEN` | （第 1 步获取的 token） | Turso 认证 token |
| `DEVICE_TOKEN_1` | `your_token:device-id:Device Name:windows` | 设备认证 token |
| `DISPLAY_NAME` | `Your Name` | 站点显示名称 |
| `SITE_TITLE` | `Live Dashboard` | 站点标题 |

3. 点击 **"Save Changes"**

---

### 第 4 步：测试数据库连接（可选但推荐）

在本地创建 `.env` 文件：

```bash
cd packages/backend
```

创建文件 `.env`：
```env
HASH_SECRET=第 2 步生成的密钥
TURSO_DATABASE_URL=libsql://live-dashboard-db-soyo.aws-us-east-1.turso.io
TURSO_AUTH_TOKEN=第 1 步获取的 token
```

运行测试脚本：
```bash
bun run test-db.ts
```

**预期输出：**
```
🔍 尝试连接到 Turso 数据库...
URL: libsql://***.turso.io
✅ 连接成功！
测试结果: [ [Object: null prototype] { test: 1 } ]

⚠️  数据库为空，首次部署时会自动创建表

✨ 数据库配置验证完成！
```

---

### 第 5 步：触发重新部署

#### 方式 A：通过 Git 推送（推荐）

```bash
# 提交所有更改
git add .
git commit -m "feat: 适配 Turso 数据库，实现持久化存储"
git push origin main
```

Render 会自动检测到新的 commit 并触发部署。

#### 方式 B：手动触发

1. 登录 Render Dashboard
2. 进入你的服务页面
3. 点击 **"Manual Deploy"**
4. 选择 `main` 分支
5. 点击 **"Deploy"**

---

### 第 6 步：验证部署

部署完成后（约 2-5 分钟），检查以下内容：

#### 6.1 访问前端页面
```
https://live-dashboard-plus.onrender.com
```

#### 6.2 检查 API 响应
```bash
curl https://live-dashboard-plus.onrender.com/api/health
# 应返回：{"status":"ok"}
```

#### 6.3 检查数据库表创建

查看 Render 日志：
1. Render Dashboard → 你的服务 → **"Logs"**
2. 查找以下日志：
   ```
   [db] Connecting to Turso database: libsql://***.turso.io
   ```
3. 确认没有数据库错误

#### 6.4 测试 Agent 上报

在你的 Windows 设备上运行 Agent，然后检查：
- Render 日志中是否有 `[report]` 相关日志
- 访问 `https://live-dashboard-plus.onrender.com/api/current`
- 应该能看到设备信息

---

## 🎯 成功标志

部署成功后，你应该看到：

✅ 前端页面正常加载  
✅ `/api/health` 返回健康状态  
✅ `/api/current` 返回设备列表（可能为空）  
✅ Agent 能正常上报数据  
✅ **最重要的是：重启后数据不会丢失！**

---

## ⚠️ 常见问题排查

### 问题 1：数据库连接失败

**症状：**
```
[db] Error: connection closed
```

**解决：**
1. 检查 `TURSO_AUTH_TOKEN` 是否正确
2. 确认 Turso 数据库正常运行
3. 检查 Render 日志中的完整错误信息

### 问题 2：环境变量未生效

**症状：**
启动日志显示仍在使用本地 SQLite

**解决：**
1. Render Dashboard → Environment → 确认变量已保存
2. 手动触发一次重新部署
3. 检查变量名是否完全匹配（区分大小写）

### 问题 3：CORS 错误

**症状：**
浏览器控制台显示 CORS 错误

**解决：**
这是正常的，因为 API 已经添加了 CORS 头。如果是生产环境，确保前端域名正确。

### 问题 4：部署后仍然是空数据

**原因：**
数据库表是新建的，自然没有历史数据

**解决：**
这是正常的！让 Agent 运行一段时间，数据会逐步积累。

---

## 📊 监控与维护

### 日常监控

1. **Render 日志**
   - 访问频率：每天
   - 检查：数据库错误、Agent 上报状态

2. **Turso 使用量**
   - 访问频率：每周
   - 网址：https://app.turso.tech/
   - 检查：存储用量、读取次数

3. **数据备份**（推荐每月一次）
   ```bash
   turso db shell live-dashboard-db-soyo ".backup backup-$(date +%Y%m%d).db"
   ```

### 性能优化建议

如果后续用户增多，考虑：
1. 添加 Redis 缓存（减少 Turso 读取）
2. 升级 Turso 计划（$9/月，更多配额）
3. 启用 Render 付费计划（$7/月，不休眠）

---

## 🎉 部署完成！

现在你可以：
- ✅ 随时推送代码，自动部署且数据不丢失
- ✅ Agent 持续上报数据，永久保存
- ✅ 在前端查看历史活动记录
- ✅ 高枕无忧，不再担心数据丢失！

---

**下一步建议：**
1. 配置域名（可选）
2. 启用 HTTPS（Render 自动提供）
3. 设置监控告警（可选）
4. 邀请朋友使用 😄

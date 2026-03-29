// 测试 Turso 数据库连接
import { createClient } from "@libsql/client";

const tursoUrl = process.env.TURSO_DATABASE_URL;
const tursoToken = process.env.TURSO_AUTH_TOKEN;

if (!tursoUrl || !tursoToken) {
  console.error("❌ 错误：缺少环境变量");
  console.error("请设置:");
  console.error("  TURSO_DATABASE_URL=libsql://...");
  console.error("  TURSO_AUTH_TOKEN=eyJ...");
  process.exit(1);
}

console.log("🔍 尝试连接到 Turso 数据库...");
console.log("URL:", tursoUrl.replace(/\/\/([^\/]+)\./, '//***.$1'));

try {
  const client = createClient({
    url: tursoUrl,
    authToken: tursoToken,
  });

  // 测试查询
  const result = await client.execute("SELECT 1 as test");
  console.log("✅ 连接成功！");
  console.log("测试结果:", result.rows);

  // 检查表是否存在
  const tables = await client.execute(`
    SELECT name FROM sqlite_master 
    WHERE type='table' 
    ORDER BY name
  `);
  
  if (tables.rows.length > 0) {
    console.log("\n📊 已存在的表:");
    tables.rows.forEach((row: any) => {
      console.log("  -", row.name);
    });
  } else {
    console.log("\n⚠️  数据库为空，首次部署时会自动创建表");
  }

  client.close();
  console.log("\n✨ 数据库配置验证完成！");
} catch (error: any) {
  console.error("\n❌ 连接失败:");
  console.error("错误信息:", error.message);
  console.error("\n请检查:");
  console.error("1. TURSO_DATABASE_URL 是否正确");
  console.error("2. TURSO_AUTH_TOKEN 是否有效");
  console.error("3. 网络连接是否正常");
  process.exit(1);
}

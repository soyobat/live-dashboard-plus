/**
 * Cute app description mapping for privacy-friendly display.
 * Maps app_name (from backend app-names.json) to kawaii descriptions.
 */

const descriptions: Record<string, string> = {
  // Messaging
  Telegram: "正在聊天喵~",
  QQ: "正在聊天喵~",
  WeChat: "正在聊天喵~",
  微信: "正在聊天喵~",
  Discord: "正在聊天喵~",
  Line: "正在聊天喵~",

  // Browsers
  "Microsoft Edge": "正在冲浪喵~",
  "Google Chrome": "正在冲浪喵~",
  Chrome: "正在冲浪喵~",
  Firefox: "正在冲浪喵~",
  Safari: "正在冲浪喵~",
  Opera: "正在冲浪喵~",
  Arc: "正在冲浪喵~",

  // Code editors
  "VS Code": "正在写代码喵~",
  "Visual Studio Code": "正在写代码喵~",
  "Visual Studio": "正在写代码喵~",
  "IntelliJ IDEA": "正在写代码喵~",
  PyCharm: "正在写代码喵~",
  WebStorm: "正在写代码喵~",
  "Android Studio": "正在写代码喵~",
  Cursor: "正在写代码喵~",
  "Sublime Text": "正在写代码喵~",

  // File managers
  文件资源管理器: "正在翻文件喵~",
  "File Explorer": "正在翻文件喵~",
  Finder: "正在翻文件喵~",
  "Total Commander": "正在翻文件喵~",

  // Terminals
  "Windows Terminal": "正在敲命令喵~",
  Terminal: "正在敲命令喵~",
  PowerShell: "正在敲命令喵~",
  "Command Prompt": "正在敲命令喵~",
  iTerm2: "正在敲命令喵~",

  // Video
  哔哩哔哩: "正在看番喵~",
  bilibili: "正在看番喵~",
  YouTube: "正在看视频喵~",
  Netflix: "正在追剧喵~",
  "爱奇艺": "正在追剧喵~",
  优酷: "正在追剧喵~",
  腾讯视频: "正在追剧喵~",
  VLC: "正在看视频喵~",
  PotPlayer: "正在看视频喵~",
  mpv: "正在看视频喵~",

  // Music
  Spotify: "正在听歌喵~",
  网易云音乐: "正在听歌喵~",
  "QQ音乐": "正在听歌喵~",
  "Apple Music": "正在听歌喵~",
  foobar2000: "正在听歌喵~",

  // Gaming
  Steam: "正在玩游戏喵~",
  "Epic Games": "正在玩游戏喵~",
  "Genshin Impact": "正在玩原神喵~",
  原神: "正在玩原神喵~",
  "League of Legends": "正在打LOL喵~",
  "Honkai: Star Rail": "正在开拓喵~",
  "崩坏：星穹铁道": "正在开拓喵~",
  Minecraft: "正在挖矿喵~",

  // Galgame / Visual Novels
  "いろとりどりのセカイ": "正在攻略gal喵~",
  "五彩斑斓的世界": "正在攻略gal喵~",
  "FAVORITE": "正在攻略gal喵~",
  "ものべの": "正在攻略gal喵~",
  "CLANNAD": "正在攻略gal喵~",
  "Fate/stay night": "正在攻略gal喵~",
  "Summer Pockets": "正在攻略gal喵~",
  "サマーポケッツ": "正在攻略gal喵~",
  "Doki Doki Literature Club": "正在攻略gal喵~",
  "WHITE ALBUM 2": "正在攻略gal喵~",
  "千恋＊万花": "正在攻略gal喵~",
  "Making*Lovers": "正在攻略gal喵~",
  "Sabbat of the Witch": "正在攻略gal喵~",
  "サノバウィッチ": "正在攻略gal喵~",
  "Riddle Joker": "正在攻略gal喵~",
  "喫茶ステラと死神の蝶": "正在攻略gal喵~",
  Kirikiri: "正在攻略gal喵~",
  KiriKiri: "正在攻略gal喵~",
  BGI: "正在攻略gal喵~",
  SiglusEngine: "正在攻略gal喵~",
  "Ethornell": "正在攻略gal喵~",
  "CatSystem2": "正在攻略gal喵~",

  // Productivity
  "Microsoft Word": "正在写文档喵~",
  "Microsoft Excel": "正在算表格喵~",
  "Microsoft PowerPoint": "正在做PPT喵~",
  OneNote: "正在记笔记喵~",
  Notion: "正在记笔记喵~",
  Obsidian: "正在记笔记喵~",

  // Social / Reading
  Twitter: "正在刷推喵~",
  X: "正在刷推喵~",
  微博: "正在刷微博喵~",
  小红书: "正在逛小红书喵~",
  抖音: "正在刷短视频喵~",
  TikTok: "正在刷短视频喵~",
  知乎: "正在涨知识喵~",

  // System
  "Task Manager": "正在看任务管理器喵~",
  Settings: "正在调设置喵~",
  设置: "正在调设置喵~",
};

const DEFAULT_DESCRIPTION = "正在忙呢喵~";

// Pre-build lowercase index for O(1) lookups
const lowerIndex = new Map<string, string>();
for (const [key, value] of Object.entries(descriptions)) {
  lowerIndex.set(key.toLowerCase(), value);
}

export function getAppDescription(appName: string): string {
  if (!appName) return DEFAULT_DESCRIPTION;
  return lowerIndex.get(appName.toLowerCase()) ?? DEFAULT_DESCRIPTION;
}

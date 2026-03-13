import appNamesData from "../data/app-names.json";

// Build case-insensitive lookup maps
const windowsMap = new Map<string, string>();
for (const [key, value] of Object.entries(appNamesData.windows)) {
  windowsMap.set(key.toLowerCase(), value);
}

const androidMap = new Map<string, string>();
for (const [key, value] of Object.entries(appNamesData.android)) {
  androidMap.set(key.toLowerCase(), value);
}

export function resolveAppName(
  appId: string,
  platform: "windows" | "android"
): string {
  if (!appId || typeof appId !== "string") return "Unknown";
  const lower = appId.toLowerCase();
  const map = platform === "windows" ? windowsMap : androidMap;

  // Exact match (case-insensitive)
  const found = map.get(lower);
  if (found) return found;

  // Android: extract last segment of package name
  if (platform === "android" && appId.includes(".")) {
    const parts = appId.split(".");
    const last = parts[parts.length - 1];
    return last.charAt(0).toUpperCase() + last.slice(1);
  }

  // Windows: strip .exe
  if (platform === "windows" && lower.endsWith(".exe")) {
    return appId.replace(/\.exe$/i, "");
  }

  return appId;
}

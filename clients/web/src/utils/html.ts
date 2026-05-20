export function escapeHtml(value: string) {
  return value.replace(/[&<>'"]/g, (char) => {
    const entities: Record<string, string> = {
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      "'": "&#39;",
      '"': "&quot;",
    };
    return entities[char] ?? char;
  });
}

export function formatJson(value: unknown) {
  return escapeHtml(JSON.stringify(value, null, 2));
}

export interface JsonPreviewOptions {
  maxDepth?: number;
  maxArrayItems?: number;
  maxObjectKeys?: number;
  maxStringLength?: number;
}

const DEFAULT_JSON_PREVIEW: Required<JsonPreviewOptions> = {
  maxDepth: 4,
  maxArrayItems: 12,
  maxObjectKeys: 24,
  maxStringLength: 400,
};

function compactJsonValue(value: unknown, depth: number, options: Required<JsonPreviewOptions>): unknown {
  if (typeof value === "string") {
    return value.length > options.maxStringLength
      ? `${value.slice(0, options.maxStringLength)}… (${value.length - options.maxStringLength} more chars)`
      : value;
  }
  if (value === null || typeof value !== "object") return value;
  if (depth >= options.maxDepth) return "… depth limit";
  if (Array.isArray(value)) {
    const items = value.slice(0, options.maxArrayItems).map((item) => compactJsonValue(item, depth + 1, options));
    if (value.length > options.maxArrayItems) items.push(`… ${value.length - options.maxArrayItems} more item(s)`);
    return items;
  }
  const output: Record<string, unknown> = {};
  const entries = Object.entries(value as Record<string, unknown>);
  for (const [key, item] of entries.slice(0, options.maxObjectKeys)) {
    output[key] = compactJsonValue(item, depth + 1, options);
  }
  if (entries.length > options.maxObjectKeys) output.__truncated_keys__ = `${entries.length - options.maxObjectKeys} more key(s)`;
  return output;
}

export function formatJsonPreview(value: unknown, options: JsonPreviewOptions = {}) {
  const resolved = { ...DEFAULT_JSON_PREVIEW, ...options };
  return escapeHtml(JSON.stringify(compactJsonValue(value, 0, resolved), null, 2));
}

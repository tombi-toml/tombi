export function normalizePath(path: string): string {
  const base = import.meta.env.BASE_URL || "/";
  const baseNormalized =
    base && base !== "/" ? (base.endsWith("/") ? base.slice(0, -1) : base) : "";
  let normalized = path.trim();

  if (!normalized) return "/";

  normalized = normalized.replace(/\/{2,}/g, "/");

  if (baseNormalized) {
    if (normalized === baseNormalized) {
      normalized = "/";
    } else if (normalized.startsWith(`${baseNormalized}/`)) {
      normalized = normalized.slice(baseNormalized.length);
    }
  }

  if (!normalized.startsWith("/")) {
    normalized = `/${normalized}`;
  }

  if (normalized.length > 1 && normalized.endsWith("/")) {
    normalized = normalized.slice(0, -1);
  }

  return normalized;
}

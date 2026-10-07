// FinUI: file-drop, the rules that are not drawing: which files a drop takes (an `accept` as <input type=file> reads
// one: extensions, types, families of types) and a size said as people read it. Pure; tested in FinUI's repository.

/** Does `file` ({ name, type }) match `accept` (".mkv,.mp4", "image/*", "" for anything)? */
export function accepts(file, accept) {
  const rules = String(accept || '').split(',').map((r) => r.trim().toLowerCase()).filter(Boolean);
  if (!rules.length) return true;
  const name = String(file.name || '').toLowerCase(), type = String(file.type || '').toLowerCase();
  return rules.some((r) => (r.startsWith('.') ? name.endsWith(r) : r.endsWith('/*') ? type.startsWith(r.slice(0, -1)) : type === r));
}
const UNITS = ['B', 'KB', 'MB', 'GB', 'TB'];
/** 1536 → "1.5 KB", 4.1 GiB → "4.1 GB": one decimal under ten, none above. */
export function fileSize(bytes) {
  let n = Math.max(0, Number(bytes) || 0), u = 0;
  while (n >= 1024 && u < UNITS.length - 1) { n /= 1024; u++; }
  return `${u && n < 10 ? Number(n.toFixed(1)) : Math.round(n)} ${UNITS[u]}`;
}

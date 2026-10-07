// FinUI: tag-input, the rules that are not drawing: what typed or pasted words become: split at commas and new lines,
// trimmed, each tag once whatever its case, and no more than allowed. Pure; tested in FinUI's repository.

/** The tags after `text` is added to `list`. */
export function addTags(list, text, { max = Infinity } = {}) {
  const out = [...list];
  for (const raw of String(text).split(/[,\n]/)) {
    const t = raw.trim();
    if (!t || out.length >= max || out.some((x) => x.toLowerCase() === t.toLowerCase())) continue;
    out.push(t);
  }
  return out;
}

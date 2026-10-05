// FinUI: breadcrumb, the rules that are not drawing — a long trail keeps its first place and its last ones, and the rest
// wait behind one ellipsis (null). Pure; tested in FinUI's repository.

/** `items` kept to `max` places: the first, null for the ones left out, the last max − 2. */
export const collapse = (items, max = 4) => (items.length <= max ? [...items] : [items[0], null, ...items.slice(items.length - (max - 2))]);

// finui format: the two ways a component writes a value itself. Numbers are printed as en-US, the way tables read them
// back when they sort; a name with no picture becomes its initials.

const nf = new Intl.NumberFormat('en-US');
/** 1234.5 → "1,235". */
export const num = (n) => nf.format(Math.round(Number(n) || 0));

/** "Big Buck Bunny" → "BB". */
export function initials(name) {
  const parts = String(name || '?').trim().split(/\s+/).filter(Boolean);
  const a = parts[0]?.[0] || '?';
  const b = parts.length > 1 ? parts[parts.length - 1][0] : '';
  return (a + b).toUpperCase();
}

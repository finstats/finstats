// FinUI: number-stepper, the rules that are not drawing: a value kept inside its ends, a step up or down, and what was
// typed read as a number (a comma as a point, as half the world writes one). Pure; tested in FinUI's repository.

const tidy = (n, step) => { const d = (String(step).split('.')[1] || '').length; return Number(n.toFixed(Math.min(10, d))); };
export const clamp = (v, { min = -Infinity, max = Infinity } = {}) => Math.min(max, Math.max(min, v));
/** `v` moved `dir` steps (1 up, −1 down), inside its ends. */
export const stepBy = (v, dir, o) => tidy(clamp(v + dir * (o.step || 1), o), o.step || 1);
/** What was typed, as a number; null when it is none. */
export function parseNumber(text) {
  const t = String(text ?? '').trim().replace(/\s/g, '').replace(',', '.');
  return t !== '' && /^-?\d*\.?\d+$/.test(t) ? Number(t) : null;
}

// FinUI: toast, the rules that are not drawing — how long a message stays (long enough to read; longer when it offers
// something to do; until closed when asked) and which are shown when several arrive. Pure; tested in FinUI's repository.

/** How many milliseconds `toast` ({ text, action, sticky }) stays. */
export function lifetime({ text = '', action = null, sticky = false }) {
  if (sticky) return Infinity;
  const reading = Math.min(10000, Math.max(4000, 2000 + String(text).length * 55));
  return action ? Math.max(7000, reading) : reading;
}
/** The toasts on screen out of `list` (oldest first): the newest `max`, newest first. */
export const shown = (list, max = 3) => list.slice(-max).reverse();

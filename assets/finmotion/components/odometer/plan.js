// FinMotion: odometer, the rules that are not drawing. Pure; tested in its QA.

const digit = (c) => c >= '0' && c <= '9';

/** How a number shown as `from` becomes `to`, character by character from the right: a digit rolls from its old value to
 *  its new one: forward when the number grew, past 9 if it must (to is then more than 9), back when it shrank (past 0:
 *  less than 0); a digit that is new rolls up from 0; anything else (a comma, a unit) just stands. */
export function reels(from, to) {
  const grew = Number(String(to).replace(/\D/g, '')) >= Number(String(from).replace(/\D/g, '') || 0);
  const a = [...String(from)].reverse(), b = [...String(to)].reverse();
  return b.map((ch, i) => {
    if (!digit(ch)) return { kind: 'char', char: ch };
    const was = digit(a[i] || '') ? +a[i] : 0, now = +ch;
    const to = grew ? (now < was ? now + 10 : now) : (now > was ? now - 10 : now);
    return { kind: 'digit', from: was, to };
  }).reverse();
}

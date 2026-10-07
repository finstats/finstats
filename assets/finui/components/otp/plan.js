// FinUI: otp, the rules that are not drawing: a code typed or pasted into a row of boxes: one character a box, from the
// box it went into, keeping only what a box takes (digits, unless told otherwise). Pure; tested in FinUI's repository.

/** The boxes after `text` arrives at box `at`. What does not fit is dropped. */
export function spread(boxes, at, text, length = boxes.length, allowed = /[0-9]/) {
  const out = boxes.slice(0, length);
  while (out.length < length) out.push('');
  let i = at;
  for (const ch of String(text)) {
    if (i >= length) break;
    if (!allowed.test(ch)) continue;
    out[i++] = ch;
  }
  return out;
}

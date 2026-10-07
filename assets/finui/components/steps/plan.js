// FinUI: steps, the rules that are not drawing: which steps of a wizard can be gone to (back to any done, forward only
// to the next once this one is done) and what each step is now. Pure; tested in FinUI's repository.

/** Can step `i` be gone to from `current`, with `done` the steps finished? */
export const reachable = (i, current, done) => i <= current ? (i === current || done.includes(i)) : i === current + 1 && done.includes(current);
/** 'done', 'current' or 'later'. */
export const stateOf = (i, current, done) => (i === current ? 'current' : done.includes(i) ? 'done' : 'later');

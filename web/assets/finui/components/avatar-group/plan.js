// FinUI: avatar-group, the rules that are not drawing — how many faces a group shows in `max` places: all of them when
// they fit, else one place fewer and a "+N" in the last. Pure; tested in FinUI's repository.

export const fit = (people, max) => (people.length <= max ? { shown: [...people], more: 0 } : { shown: people.slice(0, max - 1), more: people.length - (max - 1) });

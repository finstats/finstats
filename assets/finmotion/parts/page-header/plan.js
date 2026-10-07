// FinMotion: page-header, the rules that are not drawing. Pure; tested in its QA.

/** How gathered a header is, 0 to 1, once the page has scrolled `scrolled` px of the `over` px it gathers across. */
export const gathered = (scrolled, over) => Math.max(0, Math.min(1, scrolled / over));

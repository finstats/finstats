// The watchlist, wherever a film or a show is offered: whether this server keeps one at all.
import { state } from './state.js';

/** An older server says nothing about watchlists, and then no page offers one. */
export const hasWatchlist = () => !!(state.user && state.user.features && state.user.features.watchlist);

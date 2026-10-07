// FinMotion: the parts. Each moves one FinUI component and lives in a folder of its own (parts/<name>/): its module exports
// `part` ({ name, selector, enhance(el) → stop? }), and enhance() is called once for every element matching selector, the
// ones on the page when motion() starts and the ones drawn later.

import { part as toggle } from './toggle/toggle.js';
import { part as tabs } from './tabs/tabs.js';
import { part as field } from './field/field.js';
import { part as copy } from './copy/copy.js';
import { part as statTile } from './stat-tile/stat-tile.js';
import { part as barChart } from './bar-chart/bar-chart.js';
import { part as lineChart } from './line-chart/line-chart.js';
import { part as sparkline } from './sparkline/sparkline.js';
import { part as donutChart } from './donut-chart/donut-chart.js';
import { part as heatmap } from './heatmap/heatmap.js';
import { part as progress } from './progress/progress.js';
import { part as toast } from './toast/toast.js';
import { part as drawer } from './drawer/drawer.js';
import { part as dataTable } from './data-table/data-table.js';
import { part as poster } from './poster/poster.js';
import { part as pageHeader } from './page-header/page-header.js';

// The modal is styles only (its arrival): it has no module.
export const PARTS = [toggle, tabs, field, copy, statTile, barChart, lineChart, sparkline, donutChart, heatmap, progress, toast, drawer, dataTable, poster, pageHeader];

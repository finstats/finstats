// Settings → Unlinked media: plays whose title the library does not have under that name, almost always from an import
// (Tautulli, Jellystat, Streamystats) whose server named things differently. Each is linked to the right title by hand,
// with the likeliest one already found. The list itself is `missing.js`; this is its page.
import { h } from '../dom.js';
import { can } from '../state.js';
import { card } from '../components.js';
import { missingCard } from './missing.js';
import { t } from '../i18n.js';

export default {
  key: 'unlinked', label: t('settings.sections.unlinked.label'), sub: t('settings.sections.unlinked.sub'), group: t('settings.groups.data'), icon: 'link',
  visible: () => can('manage'),
  entries: [{ id: 'unlinked', label: t('settings.unlinked.entry'), hint: t('settings.unlinked.entry_hint') }],
  async render(slot) {
    const body = h('div');
    const box = card({ title: t('settings.sections.unlinked.label'), sub: t('settings.unlinked.card_sub'), body, id: 'unlinked' });
    slot.append(box);
    missingCard(body, box, { empty: t('settings.unlinked.empty') });
  },
};

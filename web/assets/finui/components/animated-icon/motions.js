// FinUI: what each icon does when it moves. An icon is drawn in stroke by stroke, does what it is about — refresh turns,
// download drops into its tray, a heart beats — and is drawn out again; the drawing is the same for all, the act is the
// icon's own. A motion names the act, the shapes that do it (by their place in the icon, all of them when left out), the
// way they go (dx, dy: -1, 0 or 1; dir: 1 or -1 for a turn; sx: how far a stretch goes) and the point they turn about (in the icon's 24×24 box;
// left out, each shape turns about its own centre). The shapes of a motion move as one; `stagger` has them take turns, a
// bar after a bar. Pure, so a test reads it without a page.

/** Every act there is; animated-icon.css has the keyframes of each. */
export const ACTS = ['spin', 'exit', 'nudge', 'beat', 'blaze', 'twinkle', 'swing', 'shake', 'bob', 'pop', 'redraw', 'grow', 'blink', 'slide', 'squeeze', 'turn', 'flip', 'orbit', 'sweep', 'lift', 'stretch', 'door', 'unroll', 'join', 'scribble', 'fold', 'wink', 'ring', 'flick', 'upend'];

const C = [12, 12];
export const MOTIONS = {
  // The door swings open on its hinge (its left edge) and closes again; the house stands.
  home: [{ act: 'door', parts: [2], origin: [10, 21] }],
  activity: [{ act: 'redraw' }],
  users: [{ act: 'bob', parts: [0, 2], dy: -1, stagger: true }],
  user: [{ act: 'bob', parts: [0], dy: -1 }],
  library: [{ act: 'swing', parts: [2], origin: [18, 20] }],
  play: [{ act: 'nudge', dx: 1 }],
  pause: [{ act: 'bob', dy: 1, stagger: true }],
  // Each knob slides 3 units, and the line either side of it stretches from its far end so the gap goes with the knob.
  sliders: [
    { act: 'stretch', parts: [0], sx: 0.7, origin: [4, 6] }, { act: 'stretch', parts: [1], sx: 2.5, origin: [20, 6] }, { act: 'slide', parts: [2], dx: -1 },
    { act: 'stretch', parts: [3], sx: 2.5, origin: [4, 12] }, { act: 'stretch', parts: [4], sx: 0.7, origin: [20, 12] }, { act: 'slide', parts: [5], dx: 1 },
    { act: 'stretch', parts: [6], sx: 0.7, origin: [4, 18] }, { act: 'stretch', parts: [7], sx: 2.5, origin: [20, 18] }, { act: 'slide', parts: [8], dx: -1 },
  ],
  log: [{ act: 'redraw', parts: [2, 3], stagger: true }],
  settings: [{ act: 'spin', parts: [0], origin: C }],
  search: [{ act: 'orbit' }],
  plus: [{ act: 'turn', parts: [0, 1], origin: C }],
  x: [{ act: 'turn', origin: C }],
  check: [{ act: 'redraw' }],
  copy: [{ act: 'nudge', parts: [0], dx: -1, dy: -1 }],
  menu: [{ act: 'squeeze', stagger: true }],
  arrowUp: [{ act: 'exit', dy: -1 }],
  trendUp: [{ act: 'redraw', parts: [0] }, { act: 'nudge', parts: [1], dx: 1, dy: -1 }],
  trendDown: [{ act: 'redraw', parts: [0] }, { act: 'nudge', parts: [1], dx: 1, dy: 1 }],
  minus: [{ act: 'squeeze' }],
  chevronDown: [{ act: 'nudge', dy: 1 }],
  chevronRight: [{ act: 'nudge', dx: 1 }],
  chevronLeft: [{ act: 'nudge', dx: -1 }],
  table: [{ act: 'redraw', parts: [1, 2, 3], stagger: true }],
  chart: [{ act: 'grow', parts: [0, 1, 2], stagger: true }],
  upload: [{ act: 'exit', parts: [0, 1], dy: -1 }],
  together: [{ act: 'bob', parts: [0, 4], dy: -1 }],
  share: [{ act: 'pop', parts: [0, 1, 2], stagger: true }],
  download: [{ act: 'exit', parts: [0, 1], dy: 1 }],
  inbox: [{ act: 'bob', parts: [0], dy: 1 }],
  plug: [{ act: 'nudge', parts: [0, 1, 2], dy: -1 }],
  alert: [{ act: 'shake', parts: [0] }, { act: 'blink', parts: [1, 2] }],
  info: [{ act: 'redraw', parts: [1] }, { act: 'pop', parts: [2] }],
  logout: [{ act: 'exit', parts: [1, 2], dx: 1 }],
  trash: [{ act: 'lift', parts: [0, 1], origin: [4, 7] }],
  refresh: [{ act: 'spin', origin: C }],
  film: [{ act: 'blink', parts: [3, 5, 4, 6], stagger: true }],
  database: [{ act: 'bob', dy: -1, stagger: true }],
  // The two halves part, twisting a little, and click back into each other.
  link: [{ act: 'join', parts: [0], dx: 1, dy: -1, dir: 1 }, { act: 'join', parts: [1], dx: -1, dy: 1, dir: -1 }],
  shield: [{ act: 'beat' }],
  server: [{ act: 'blink', parts: [2, 3], stagger: true }],
  // The ribbon is drawn up to where it hangs and unrolls down the page again, settling with a sway of its length.
  bookmark: [{ act: 'unroll', origin: [12, 3.5] }],
  heart: [{ act: 'beat' }],
  globe: [{ act: 'flip', parts: [2, 3], origin: C }],
  lan: [{ act: 'redraw', parts: [3, 4], stagger: true }],
  external: [{ act: 'exit', parts: [0, 1], dx: 1, dy: -1 }],
  skip: [{ act: 'nudge', dx: 1 }],
  stop: [{ act: 'beat' }],
  volume: [{ act: 'blink', parts: [1, 2], stagger: true }],
  captions: [{ act: 'redraw', parts: [1, 2, 3, 4], stagger: true }],
  tv: [{ act: 'swing', parts: [1], origin: [12, 7] }],
  tag: [{ act: 'swing', origin: [7.5, 7.5] }],
  recap: [{ act: 'spin', dir: -1, origin: C }],
  cpu: [{ act: 'pop', parts: [1] }, { act: 'blink', parts: [2, 3, 4, 5, 6, 7, 8, 9], stagger: true }],
  clock: [{ act: 'spin', parts: [1], origin: C }],
  github: [{ act: 'swing', parts: [1], origin: [9, 18] }],
  calendar: [{ act: 'bob', parts: [2, 3], dy: -1, stagger: true }],
  // It burns: its tongue bends and licks either way, the lick at its side jumps, and its base stays put.
  flame: [{ act: 'blaze', origin: [12, 21] }],
  repeat: [{ act: 'nudge', parts: [0], dx: 1 }, { act: 'nudge', parts: [2], dx: -1 }],
  sparkle: [{ act: 'twinkle' }],
  trophy: [{ act: 'swing', origin: [12, 21] }],
  compass: [{ act: 'spin', parts: [1], origin: C }],
  monitor: [{ act: 'redraw', parts: [0] }],
  layers: [{ act: 'bob', parts: [0], dy: -1 }],
  sun: [{ act: 'spin', parts: [1, 2, 3, 4, 5, 6, 7, 8], origin: C }, { act: 'beat', parts: [0] }],
  moon: [{ act: 'swing', origin: C }],
  gauge: [{ act: 'sweep', parts: [1], origin: [12, 14] }],
  lock: [{ act: 'shake', parts: [1] }],
  unlock: [{ act: 'nudge', parts: [1], dy: -1 }],
  shuffle: [{ act: 'nudge', parts: [1, 4], dx: 1 }],
  anchor: [{ act: 'swing', origin: [12, 5] }],
  // The speaker pumps and the plus pops beside it: louder.
  volumeUp: [{ act: 'beat', parts: [0] }, { act: 'pop', parts: [1, 2] }],
  // The minus squeezes in towards the speaker: quieter.
  volumeDown: [{ act: 'squeeze', parts: [1] }],
  volumeLow: [{ act: 'nudge', parts: [1], dx: 1 }],
  volumeMute: [{ act: 'turn', parts: [1, 2], origin: [18.5, 12] }],
  volumeOff: [{ act: 'blink', parts: [1, 2], stagger: true }, { act: 'redraw', parts: [3] }],
  // Both cups pulse with the beat, and the tip of the boom blinks: it is live.
  headset: [{ act: 'beat', parts: [1, 2] }, { act: 'blink', parts: [4] }],
  headsetOff: [{ act: 'redraw', parts: [5] }],
  headphones: [{ act: 'beat', parts: [1, 2] }],
  // The capsule bobs in its cradle, as if speaking.
  mic: [{ act: 'bob', parts: [0], dy: -1 }],
  micOff: [{ act: 'redraw', parts: [4] }],
  speaker: [{ act: 'pop', parts: [1] }, { act: 'beat', parts: [2] }],
  // The notes sway to their own tune, from where they sit.
  music: [{ act: 'swing', origin: [12, 20] }],
  listMusic: [{ act: 'bob', parts: [3, 4], dy: -1 }],
  // The antenna wobbles on its foot and the speaker plays.
  radio: [{ act: 'swing', parts: [1], origin: [6.5, 8] }, { act: 'beat', parts: [2] }],
  podcast: [{ act: 'pop', parts: [0] }, { act: 'blink', parts: [2, 3], stagger: true }],
  rewind: [{ act: 'nudge', dx: -1, stagger: true }],
  fastForward: [{ act: 'nudge', dx: 1, stagger: true }],
  skipBack: [{ act: 'nudge', parts: [1], dx: -1 }],
  playCircle: [{ act: 'nudge', parts: [1], dx: 1 }],
  pauseCircle: [{ act: 'bob', parts: [1, 2], dy: 1, stagger: true }],
  record: [{ act: 'beat', parts: [1] }],
  disc: [{ act: 'spin', origin: C }],
  // The bars jump out of turn, as a level meter does, not left to right.
  equalizer: [{ act: 'grow', parts: [1, 3, 0, 4, 2], stagger: true }],
  // The bars grow from the middle line, up and down at once, one after another.
  waveform: [{ act: 'grow', origin: C, stagger: true }],
  cast: [{ act: 'blink', parts: [1, 2, 3], stagger: true }],
  airplay: [{ act: 'nudge', parts: [1], dy: -1 }],
  // The light in its corner blinks: it is recording.
  video: [{ act: 'blink', parts: [2] }],
  videoOff: [{ act: 'redraw', parts: [2] }],
  camera: [{ act: 'pop', parts: [1] }, { act: 'blink', parts: [2] }],
  cameraOff: [{ act: 'redraw', parts: [2] }],
  // The sun shines over the hills.
  image: [{ act: 'beat', parts: [1] }],
  repeatOne: [{ act: 'nudge', parts: [0], dx: 1 }, { act: 'nudge', parts: [2], dx: -1 }, { act: 'pop', parts: [4] }],
  // It writes: the pencil runs along its tip in a quick zigzag, a word's worth, and lifts back to where it began.
  pencil: [{ act: 'scribble' }],
  penSquare: [{ act: 'scribble', parts: [1] }],
  // The eraser rubs back and forth along the line it stands on; the line stays.
  eraser: [{ act: 'shake', parts: [1, 2] }],
  // The pen runs along and its mark is drawn under it again.
  highlighter: [{ act: 'redraw', parts: [0] }, { act: 'nudge', parts: [1, 2], dx: 1 }],
  // The brush paints: it sweeps its tip to and fro from the end of its handle.
  brush: [{ act: 'swing', origin: [20, 5] }],
  palette: [{ act: 'pop', parts: [1, 2, 3, 4], stagger: true }],
  // The blades snip: each half turns about the pivot, the other way from its twin.
  scissors: [{ act: 'swing', parts: [0, 1], origin: [11.9, 12] }, { act: 'swing', parts: [2, 3], dir: -1, origin: [11.9, 12] }],
  clipboard: [{ act: 'bob', parts: [1], dy: -1 }],
  clipboardCheck: [{ act: 'redraw', parts: [2] }],
  file: [{ act: 'bob', dy: -1 }],
  fileText: [{ act: 'redraw', parts: [4, 2, 3], stagger: true }],
  filePlus: [{ act: 'turn', parts: [2, 3], origin: [12, 14.5] }],
  folder: [{ act: 'bob', dy: -1 }],
  // The front flap lifts on its bottom edge and drops back; the back of the folder stays.
  folderOpen: [{ act: 'lift', parts: [1], origin: [5, 20] }],
  folderPlus: [{ act: 'turn', parts: [1, 2], origin: [12, 13.5] }],
  // The disk's shutter slides open and shut, as it does when the disk goes in.
  save: [{ act: 'slide', parts: [2], dx: -1 }],
  // It hangs from its top bend and swings.
  paperclip: [{ act: 'swing', origin: [10.5, 3] }],
  // The letter is written again: its bar, its stem, its foot.
  type: [{ act: 'redraw', stagger: true }],
  // The lines shrink towards the edge they are aligned to and spring back, one after another.
  alignLeft: [{ act: 'squeeze', stagger: true }],
  alignCenter: [{ act: 'squeeze', origin: C, stagger: true }],
  alignRight: [{ act: 'squeeze', origin: [20, 12], stagger: true }],
  // It counts: one, two.
  listOrdered: [{ act: 'pop', parts: [3, 4], stagger: true }],
  listChecks: [{ act: 'redraw', parts: [3, 4], stagger: true }],
  quote: [{ act: 'bob', dy: -1, stagger: true }],
  // The brackets open apart and close again round the slash.
  code: [{ act: 'nudge', parts: [0], dx: -1 }, { act: 'nudge', parts: [1], dx: 1 }],
  // The prompt nudges on and the cursor blinks after it.
  terminal: [{ act: 'nudge', parts: [1], dx: 1 }, { act: 'blink', parts: [2] }],
  undo: [{ act: 'nudge', dx: -1 }],
  redo: [{ act: 'nudge', dx: 1 }],
  // The two corners are pulled apart, the crop taken wider, and let back: closed in, they would run into each other.
  crop: [{ act: 'nudge', parts: [0], dx: -1, dy: 1 }, { act: 'nudge', parts: [1], dx: 1, dy: -1 }],
  // The wand waves from its handle and its sparkles twinkle one after another.
  wand: [{ act: 'swing', parts: [0], origin: [4, 20] }, { act: 'twinkle', parts: [1, 2, 3], stagger: true }],
  // It dips its tip into the colour, twice.
  pipette: [{ act: 'nudge', dx: -1, dy: 1 }],
  // A page turns over the spine to the left and comes back.
  book: [{ act: 'flip', parts: [3], origin: C }],
  // The flap folds up off the envelope, stays open a moment, and folds shut again.
  mail: [{ act: 'fold', parts: [1], origin: [12, 7.5] }],
  // The letter rises out of the envelope and sinks back in.
  mailOpen: [{ act: 'bob', parts: [3], dy: -1 }],
  send: [{ act: 'exit', dx: 1, dy: -1 }],
  // The bubble shrinks into its tail and pops out of it again.
  message: [{ act: 'pop', origin: [3, 21] }],
  // One bubble speaks, then the other.
  messages: [{ act: 'bob', dy: -1, stagger: true }],
  // Someone is typing: the dots bob one after another.
  messageDots: [{ act: 'bob', parts: [1, 2, 3], dy: -1, stagger: true }],
  phone: [{ act: 'ring', origin: [12, 12] }],
  phoneCall: [{ act: 'ring', parts: [0], origin: [12, 12] }, { act: 'blink', parts: [1, 2], stagger: true }],
  phoneOff: [{ act: 'redraw', parts: [1] }],
  // The bell swings from its top; the clapper, hanging lowest, swings widest.
  bell: [{ act: 'swing', origin: [12, 3] }],
  bellOff: [{ act: 'redraw', parts: [3] }],
  bellRing: [{ act: 'swing', parts: [0, 1, 2], origin: [12, 3] }, { act: 'blink', parts: [3, 4] }],
  atSign: [{ act: 'redraw', parts: [1] }],
  hash: [{ act: 'redraw', stagger: true }],
  // It is raised to the mouth (turning up on its handle) and lowered, and its call goes out in waves.
  megaphone: [{ act: 'lift', parts: [0, 1], origin: [9.5, 19.5] }, { act: 'blink', parts: [2, 3], stagger: true }],
  thumbsUp: [{ act: 'bob', dy: -1 }],
  thumbsDown: [{ act: 'bob', dy: 1 }],
  // One eye winks and the smile widens.
  smile: [{ act: 'wink', parts: [3] }, { act: 'stretch', parts: [1], sx: 1.2, origin: [12, 15] }],
  // The star turns over like a coin; its other face is the same.
  star: [{ act: 'flip', origin: [12, 12] }],
  // The cloth flaps on the pole from where it is tied at the top; the pole stands.
  flag: [{ act: 'swing', parts: [1], dir: -1, origin: [5, 4.5] }],
  // The lid hops on the box, twice, as if something inside wants out.
  gift: [{ act: 'bob', parts: [2, 3, 4, 5], dy: -1 }],
  userPlus: [{ act: 'turn', parts: [2, 3], origin: [19, 11] }],
  userMinus: [{ act: 'squeeze', parts: [2] }],
  userCheck: [{ act: 'redraw', parts: [2] }],
  userX: [{ act: 'turn', parts: [2, 3], origin: [19, 11] }],
  crown: [{ act: 'bob', parts: [0], dy: -1 }],
  eye: [{ act: 'wink', origin: [12, 12] }],
  eyeOff: [{ act: 'redraw', parts: [2] }],
  reply: [{ act: 'nudge', dx: -1 }],
  forward: [{ act: 'nudge', dx: 1 }],
  // The ribbon swings from its knot under the medal.
  award: [{ act: 'swing', parts: [1], origin: [12, 14] }],
  contact: [{ act: 'redraw', parts: [3, 4], stagger: true }],
  arrowDown: [{ act: 'exit', dy: 1 }],
  arrowLeft: [{ act: 'exit', dx: -1 }],
  arrowRight: [{ act: 'exit', dx: 1 }],
  arrowUpRight: [{ act: 'exit', dx: 1, dy: -1 }],
  chevronUp: [{ act: 'nudge', dy: -1 }],
  // The two chevrons part, each the way it points, and close again.
  chevronsUpDown: [{ act: 'nudge', parts: [0], dy: -1 }, { act: 'nudge', parts: [1], dy: 1 }],
  moreHorizontal: [{ act: 'bob', dy: -1, stagger: true }],
  // Stacked dots would bump into each other going up and down, so they wave sideways, one after another.
  moreVertical: [{ act: 'bob', dx: 1, stagger: true }],
  // The cells pop one after another, round the grid clockwise.
  grid: [{ act: 'pop', parts: [0, 1, 3, 2], stagger: true }],
  // The funnel is squeezed and a drop falls from its spout, twice.
  filter: [{ act: 'squeeze', parts: [0], origin: [12, 10] }, { act: 'exit', parts: [1], dy: 1 }],
  sortAsc: [{ act: 'nudge', parts: [0, 1], dy: -1 }, { act: 'redraw', parts: [2, 3, 4], stagger: true }],
  sortDesc: [{ act: 'nudge', parts: [0, 1], dy: 1 }, { act: 'redraw', parts: [2, 3, 4], stagger: true }],
  // Each corner goes out the way it faces and comes back from the middle: the frame grows past itself.
  maximize: [
    { act: 'exit', parts: [0], dx: -1, dy: -1 }, { act: 'exit', parts: [1], dx: 1, dy: -1 },
    { act: 'exit', parts: [2], dx: 1, dy: 1 }, { act: 'exit', parts: [3], dx: -1, dy: 1 },
  ],
  minimize: [
    { act: 'nudge', parts: [0], dx: 1, dy: 1 }, { act: 'nudge', parts: [1], dx: -1, dy: 1 },
    { act: 'nudge', parts: [2], dx: -1, dy: -1 }, { act: 'nudge', parts: [3], dx: 1, dy: -1 },
  ],
  // The panel's edge slides out and back: the sidebar opens wider.
  sidebar: [{ act: 'slide', parts: [1], dx: 1 }],
  login: [{ act: 'exit', parts: [1, 2], dx: 1 }],
  // The button is pressed in, and the ring is drawn round again from the gap.
  power: [{ act: 'redraw', parts: [0] }, { act: 'bob', parts: [1], dy: 1 }],
  // The switch is thrown off and back on: the knob runs its track and returns.
  toggleOn: [{ act: 'flick', parts: [1], dx: -1 }],
  toggleOff: [{ act: 'flick', parts: [1], dx: 1 }],
  loader: [{ act: 'spin', origin: C }],
  circleCheck: [{ act: 'redraw', parts: [1] }],
  circleX: [{ act: 'turn', parts: [1, 2], origin: C }],
  // The question mark swings from the top of its hook, dot and all.
  circleHelp: [{ act: 'swing', parts: [1, 2], origin: [12, 7] }],
  ban: [{ act: 'redraw', parts: [1] }],
  zoomIn: [{ act: 'orbit', parts: [0, 1] }, { act: 'pop', parts: [2, 3], origin: [11, 11] }],
  zoomOut: [{ act: 'orbit', parts: [0, 1] }, { act: 'squeeze', parts: [2], origin: [11, 11] }],
  mapPin: [{ act: 'bob', dy: -1 }],
  // The map folds up towards its left edge and opens out again with a snap.
  map: [{ act: 'door', origin: [3, 12] }],
  navigation: [{ act: 'nudge', dx: 1, dy: -1 }],
  // The arrowheads go out each its own way and come back from the cross.
  move: [
    { act: 'exit', parts: [2], dy: -1 }, { act: 'exit', parts: [3], dx: 1 },
    { act: 'exit', parts: [4], dy: 1 }, { act: 'exit', parts: [5], dx: -1 },
  ],
  // The handle is taken hold of and dragged a little, twice.
  grip: [{ act: 'nudge', dy: -1 }],
  // The pin is pressed in, twice.
  pin: [{ act: 'bob', dy: 1 }],
  // The waves light up one after another, from the dot outward.
  wifi: [{ act: 'blink', parts: [1, 2, 3], stagger: true }],
  wifiOff: [{ act: 'redraw', parts: [4] }],
  bluetooth: [{ act: 'redraw' }],
  // Its cells light up one after another, left to right, as it charges.
  battery: [{ act: 'blink', parts: [2, 3, 4], stagger: true }],
  batteryCharging: [{ act: 'beat', parts: [2] }],
  // It buzzes.
  smartphone: [{ act: 'shake' }],
  // The lid shuts on its hinge and springs open again; the base stays.
  laptop: [{ act: 'grow', parts: [0], origin: [12, 15] }],
  // A page comes out of the bottom, and another.
  printer: [{ act: 'exit', parts: [3], dy: 1 }],
  // Keys are struck one after another, in no order, as a hand types; the space bar last.
  keyboard: [{ act: 'pop', parts: [2, 6, 1, 4, 7, 3, 5, 8], stagger: true }],
  // The wheel rolls down.
  mouse: [{ act: 'bob', parts: [1], dy: 1 }],
  // Its light flickers while it reads; the drive stays.
  hardDrive: [{ act: 'blink', parts: [2] }],
  // The cloud drifts along with the wind and back.
  cloud: [{ act: 'slide', dx: 1 }],
  cloudUpload: [{ act: 'exit', parts: [1, 2], dy: -1 }],
  cloudDownload: [{ act: 'exit', parts: [1, 2], dy: 1 }],
  // The drops fall out of the cloud, one after another.
  cloudRain: [{ act: 'exit', parts: [2, 1, 3], dy: 1, stagger: true }],
  snowflake: [{ act: 'spin', origin: C }],
  // It flashes.
  zap: [{ act: 'blink' }],
  // The umbrella sways from the hand that holds its handle.
  umbrella: [{ act: 'swing', origin: [11, 20] }],
  wind: [{ act: 'slide', dx: 1, stagger: true }],
  // The mercury drops into its bulb and rises again.
  thermometer: [{ act: 'grow', parts: [1] }],
  droplet: [{ act: 'exit', dy: 1 }],
  // The key turns in its lock about its shaft: the teeth go round to the other side and back.
  key: [{ act: 'flip', origin: [12, 12] }],
  fingerprint: [{ act: 'redraw', stagger: true }],
  cart: [{ act: 'nudge', dx: 1 }],
  creditCard: [{ act: 'slide', dx: 1 }],
  // The clasp opens on its hinge (the right edge) and shuts, and the note inside peeks up.
  wallet: [{ act: 'door', parts: [2, 3], origin: [21, 13] }, { act: 'bob', parts: [1], dy: -1 }],
  // The coin in front is tossed and turns over; the one behind waits.
  coins: [{ act: 'flip', parts: [0, 1], origin: [9, 14.5] }],
  // The receipt is printed out: drawn up short under its top edge, it rolls down to its length.
  receipt: [{ act: 'unroll', origin: [12, 3] }],
  package: [{ act: 'bob', dy: -1 }],
  truck: [{ act: 'nudge', dx: 1 }],
  // The rocket lifts off up and away, and the exhaust it leaves flickers.
  rocket: [{ act: 'exit', parts: [0, 1, 2, 3], dx: 1, dy: -1 }, { act: 'blink', parts: [4] }],
  lightbulb: [{ act: 'beat', parts: [0] }, { act: 'blink', parts: [2, 3, 4, 5, 6], stagger: true }],
  coffee: [{ act: 'exit', parts: [2, 3, 4], dy: -1, stagger: true }],
  // Each leg swings where it meets the body, those on the left one way and those on the right the other.
  bug: [
    { act: 'swing', parts: [5], origin: [7.5, 12] }, { act: 'swing', parts: [6], origin: [7.5, 15] }, { act: 'swing', parts: [7], origin: [7.7, 18] },
    { act: 'swing', parts: [8], dir: -1, origin: [16.5, 12] }, { act: 'swing', parts: [9], dir: -1, origin: [16.5, 15] }, { act: 'swing', parts: [10], dir: -1, origin: [16.3, 18] },
  ],
  gamepad: [{ act: 'pop', parts: [3, 4], stagger: true }],
  // The piece comes out of its place, a little twisted, and clicks back in.
  puzzle: [{ act: 'join', dx: 1, dy: -1, dir: 1 }],
  // The glass is turned over end for end, and the sand in it runs down and heaps up again.
  hourglass: [{ act: 'upend', parts: [0, 1, 2, 3], origin: C }, { act: 'grow', parts: [4] }],
  // The hand goes round, and the button on top is pressed to start it and to stop it.
  timer: [{ act: 'spin', parts: [1], origin: [12, 13.5] }, { act: 'bob', parts: [2], dy: 1 }],
  alarm: [{ act: 'shake' }],
  // The arrow strikes the centre, again and again.
  target: [{ act: 'exit', parts: [2, 3], dx: -1, dy: 1 }],
};

/** Each of the `count` shapes of icon `name`, in order: { act, k, dx, dy, dir, sx, origin } — act null for a shape that is
 *  only drawn; k is its place among the shapes of its act, for a group that takes turns; origin a CSS transform-origin,
 *  or null for the shape's own centre. */
export function plan(name, count) {
  const motions = MOTIONS[name];
  if (!motions) throw new Error(`no motion for the icon ${name}`);
  const parts = Array.from({ length: count }, () => ({ act: null, k: 0, dx: 0, dy: 0, dir: 1, sx: 1, origin: null }));
  for (const m of motions) {
    const which = m.parts || parts.map((_, i) => i);
    // Parts move as one unless the motion says they take turns: a staggered refresh comes apart as it spins.
    which.forEach((i, k) => {
      parts[i] = { act: m.act, k: m.stagger ? k : 0, dx: m.dx || 0, dy: m.dy || 0, dir: m.dir || 1, sx: m.sx || 1, origin: m.origin ? `${m.origin[0]}px ${m.origin[1]}px` : null };
    });
  }
  return parts;
}

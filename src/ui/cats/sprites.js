// Özgün piksel kedi sprite'ları — Wakai Desktop'un görsel kimliği.
//
// Her sprite 16×16'lık katmanlardan oluşur. `base` her zaman çizilir; diğer katmanlar
// CSS ile canlandırılır (göz kırpma, kuyruk sallama, Zz, parıltı). Karakterler:
//   .  saydam        o  kontur         p  kürk           s  kürk gölgesi
//   w  karın/krem    k  göz            n  burun/ağız     c  yanak
//   y  lavanta vurgu g  açık lavanta   h  beyaz parıltı
//
// Önizleme: node scripts/preview-sprites.mjs sprites.png base,tailA

export const PALETTE = {
  o: 'var(--cat-outline)',
  p: 'var(--cat-fur)',
  s: 'var(--cat-fur-shade)',
  w: 'var(--cat-belly)',
  k: 'var(--cat-eye)',
  n: 'var(--cat-nose)',
  c: 'var(--cat-cheek)',
  y: 'var(--cat-accent)',
  g: 'var(--cat-accent-soft)',
  h: '#ffffff',
};

const E = '................';

// Ortak baş (satır 0–10) — oturan, mutlu, meraklı, çalışan ve rahat kediler paylaşır.
const head = (eyesTop, eyesBottom, mouth = 'oppppoppoppppo..') => [
  E,
  '.oo........oo...',
  '.oso......oso...',
  '.opsoooooospo...',
  '.oppppppppppo...',
  'oppppppppppppo..',
  eyesTop,
  eyesBottom,
  'opcpppnnpppcpo..',
  mouth,
  '.oppppppppppo...',
];

const EYES_OPEN = ['oppkkppppkkppo..', 'oppkkppppkkppo..'];
const EYES_HAPPY = ['opppkppppkpppo..', 'oppkpkppkpkppo..'];
const EYES_CLOSED = ['oppppppppppppo..', 'oppkkppppkkppo..'];
const EYES_SPARKLE = ['oppkhppppkhppo..', 'oppkkppppkkppo..'];

const SITTING_BODY = [
  '.opwwwwwwwwpo...',
  'opwwwwwwwwwwpo..',
  'opwwwwwwwwwwpo..',
  'oppowwwwwwoppo..',
  '.oooooooooooo...',
];

// Kuyruk: oturan kedinin sağında yukarı kıvrılır. İki kare dönüşümlü gösterilir.
const TAIL_A = [E, E, E, E, E, E, E, E, E,
  '..............o.',
  '.............opo',
  '.............opo',
  '..............po',
  '..............po',
  '..............oo',
  E];
const TAIL_B = [E, E, E, E, E, E, E, E, E, E,
  '..............oo',
  '..............po',
  '..............po',
  '..............po',
  '..............oo',
  E];

// Göz kırpma: gözlerin üst sırasını kürk rengiyle örter.
const BLINK = [E, E, E, E, E, E, '...pp....pp.....', E, E, E, E, E, E, E, E, E];

export const SPRITES = {
  face: {
    base: [
      E,
      '.oo..........oo.',
      '.opo........opo.',
      '.oppo......oppo.',
      '.opppoooooopppo.',
      '.oppppppppppppo.',
      'oppppppppppppppo',
      'oppkkppppppkkppo',
      'oppkkppppppkkppo',
      'opcppppnnppppcpo',
      'opppppoppopppppo',
      '.oppppppppppppo.',
      '..oppppppppppo..',
      '...oooooooooo...',
      E,
      E,
    ],
    blink: [E, E, E, E, E, E, E, '...pp......pp...', E, E, E, E, E, E, E, E],
  },

  sitting: {
    base: [...head(...EYES_OPEN), ...SITTING_BODY],
    blink: BLINK,
    tailA: TAIL_A,
    tailB: TAIL_B,
  },

  happy: {
    base: [...head(...EYES_HAPPY, 'opppponnoppppo..'), ...SITTING_BODY],
    tailA: TAIL_A,
    tailB: TAIL_B,
    sparkle: [
      E,
      '..............y.',
      'y............yyy',
      '..............y.',
      E, E, E, E, E, E, E, E, E, E, E, E,
    ],
  },

  curious: {
    base: [...head(...EYES_SPARKLE), ...SITTING_BODY],
    blink: BLINK,
    tailA: TAIL_A,
    tailB: TAIL_B,
    question: [
      '.............yyy',
      '...............y',
      '..............y.',
      E,
      '..............y.',
      E, E, E, E, E, E, E, E, E, E, E,
    ],
  },

  relaxed: {
    base: [...head(...EYES_CLOSED), ...SITTING_BODY],
    tailA: TAIL_A,
    tailB: TAIL_B,
  },

  working: {
    base: [
      ...head(...EYES_OPEN).slice(0, 10),
      '..oooooooooo....',
      '..oggggggggo....',
      '..ogggyygggo....',
      '..oggggggggo....',
      'opoooooooooopo..',
      'oooooooooooooo..',
    ],
    blink: BLINK,
  },

  sleeping: {
    base: [
      E,
      E,
      E,
      E,
      E,
      E,
      '..o...o.........',
      '.opo.opo........',
      '.opppppoooooooo.',
      'oppppppppppppppo',
      'opkkpkkppppppppo',
      'opcpnpcppppppppo',
      'oppppppppppppppo',
      '.oppwwwwppppppo.',
      '.oooooooooooooo.',
      E,
    ],
    zz: [
      E,
      '.........yyyy...',
      '...........y....',
      '..........y.....',
      '.........yyyy...',
      '.............yy.',
      E, E, E, E, E, E, E, E, E, E,
    ],
  },
};

export const POSES = Object.keys(SPRITES);

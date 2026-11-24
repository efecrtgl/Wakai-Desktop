// Özgün piksel kedi sprite'ları. Her karakter bir pikseli temsil eder:
// . saydam  o kontur  p kürk  s kürk gölgesi  w karın  k göz  n burun  c yanak
export const PALETTE = {
  o: 'var(--cat-outline)',
  p: 'var(--cat-fur)',
  s: 'var(--cat-fur-shade)',
  w: 'var(--cat-belly)',
  k: 'var(--cat-eye)',
  n: 'var(--cat-nose)',
  c: 'var(--cat-cheek)',
};

export const SPRITES = {
  face: [
    '................',
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
  ],
};

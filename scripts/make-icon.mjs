// Uygulama simgesini piksel kedi sprite'ından üretir: node scripts/make-icon.mjs
import { writeFileSync } from 'node:fs';

const grid = [
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
  '................',
  '................',
];
const colors = { o: '#5b4a4f', p: '#f7b8c8', k: '#3d3034', n: '#e88aa3', c: '#f49ab2' };
const pad = 2;
const size = 16 + pad * 2;
let rects = '';
grid.forEach((row, y) => {
  let x = 0;
  while (x < row.length) {
    const ch = row[x];
    let run = 1;
    while (x + run < row.length && row[x + run] === ch) run++;
    if (colors[ch]) rects += `<rect x="${x + pad}" y="${y + pad + 1}" width="${run}" height="1" fill="${colors[ch]}"/>`;
    x += run;
  }
});
const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size} ${size}" width="1024" height="1024" shape-rendering="crispEdges"><rect width="${size}" height="${size}" rx="4" fill="#fff4ec"/>${rects}</svg>`;
writeFileSync('src-tauri/icons/app-icon.svg', svg);
console.log('src-tauri/icons/app-icon.svg yazıldı');

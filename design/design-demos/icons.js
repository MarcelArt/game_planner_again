/* Pixel item icons for the demos (12x12 maps drawn with squares). Stand-ins for the icons a player
   would upload; Resin and Copper Block deliberately have none so the no-icon fallback is visible.
   GP.icon(itemId) -> data URL or null. Draw with image-rendering: pixelated. Load after shared.js. */
(function () {
  const T = {
    lump: ['............','....aaaa....','..aabbbbaa..','.abbcbbbbba.','.abbbbbcbba.','abbcbbbbbbba','abbbbbbbcbba','.abbbcbbbba.','.aabbbbbbaa.','...aabbaa...','....aaaa....','............'],
    ingot: ['............','............','............','...aaaaaaaa.','..abbbbbbca.','.abbbbbbcca.','abbbbbbccca.','acccccccca..','aaaaaaaaa...','............','............','............'],
    block: ['............','.aaaaaaaaaa.','.abbbbbbbba.','.abccccccda.','.abccccccda.','.abccccccda.','.abccccccda.','.abccccccda.','.abccccccda.','.adddddddda.','.aaaaaaaaaa.','............'],
    log: ['............','..aaaaaaaa..','.abbbbbbbba.','.abccccccba.','.abcddddcba.','.abcdccdcba.','.abcdccdcba.','.abcddddcba.','.abccccccba.','.abbbbbbbba.','..aaaaaaaa..','............'],
    stick: ['............','.........aa.','........aba.','.......aba..','......aba...','.....aba....','....aba.....','...aba......','..aba.......','.aba........','.aa.........','............'],
    torch: ['............','.....ff.....','....fyyf....','....fyyf....','.....ff.....','.....ab.....','.....ab.....','.....ab.....','.....ab.....','.....ab.....','.....ab.....','............'],
    pickaxe: ['............','..hhhhhh....','.hiiiiiih...','.hh....sih..','......sbsih.','.....sbs.hh.','....sbs..h..','...sbs......','..sbs.......','.sbs........','.ss.........','............'],
    lantern: ['............','....aaaa....','...a....a...','...aaaaaa...','..abbbbbba..','..abyyyyba..','..abyffyba..','..abyffyba..','..abyyyyba..','..abbbbbba..','..aaaaaaaa..','............'],
  };
  const wood = { a: '#4a2e16', b: '#8a5a2b', c: '#c79a55', d: '#a9793d' };
  const defs = {
    oak_log: [T.log, wood],
    oak_plank: [T.block, { a: '#4a2e16', b: '#dab676', c: '#c79a55', d: '#a9793d' }],
    stick: [T.stick, wood],
    coal: [T.lump, { a: '#0e0e13', b: '#2a2a35', c: '#565668' }],
    iron_ore: [T.lump, { a: '#44474f', b: '#868a93', c: '#e0b08a' }],
    torch: [T.torch, { a: '#4a2e16', b: '#8a5a2b', f: '#e8743b', y: '#ffd95a' }],
    iron_ingot: [T.ingot, { a: '#4b5563', b: '#eef1f5', c: '#aab3c0' }],
    iron_block: [T.block, { a: '#4b5563', b: '#f4f6f9', c: '#cfd6e0', d: '#9aa5b4' }],
    iron_pickaxe: [T.pickaxe, { h: '#4b5563', i: '#e3e8ef', s: '#4a2e16', b: '#8a5a2b' }],
    copper_ingot: [T.ingot, { a: '#6e3214', b: '#f2a877', c: '#d9763a' }],
    lantern: [T.lantern, { a: '#6e3214', b: '#d9763a', y: '#ffd95a', f: '#fff3b8' }],
  };
  const cache = {};
  window.GP.icon = function (id) {
    if (!(id in defs)) return null;
    if (cache[id]) return cache[id];
    const [map, pal] = defs[id];
    const c = document.createElement('canvas'); c.width = c.height = 12;
    const x = c.getContext('2d');
    map.forEach((row, r) => [...row].forEach((ch, col) => { if (pal[ch]) { x.fillStyle = pal[ch]; x.fillRect(col, r, 1, 1); } }));
    return (cache[id] = c.toDataURL());
  };
})();

"""Recolour dir-3.html into palette variants by remapping every colour by hue family, keeping lightness steps."""
import re, colorsys, sys
src = open('dir-3.html').read()
THEMES = {
  # family: (target hue deg, saturation multiplier, lightness multiplier)
  'ember':    {'blue': (22, .34, .92), 'violet': (24, 1.0, .88), 'gold': (46, 1, 1), 'green': (150, 1, 1)},
  'graphite': {'blue': (225, .10, .90), 'violet': (47, .95, .86), 'gold': (210, .25, 1), 'green': (152, 1, 1)},
  'pine':     {'blue': (160, .58, .88), 'violet': (36, 1.0, .90), 'gold': (18, .9, 1), 'green': (78, 1, 1)},
  'light':    {'blue': (40, .30, None), 'violet': (188, .9, None), 'gold': (30, 1, None), 'green': (158, 1, None), 'invert': True},
}
def fam(h, s):
    if s < .08: return 'grey'
    d = h * 360
    if 195 <= d < 238: return 'blue'
    if 238 <= d < 292: return 'violet'
    if 26 <= d < 58: return 'gold'
    if 135 <= d < 195: return 'green'
    return 'other'
def remap(r, g, b, t):
    h, l, s = colorsys.rgb_to_hls(r/255, g/255, b/255)
    f = fam(h, s)
    inv = t.get('invert')
    if inv and (f in ('blue', 'grey', 'violet', 'gold') or l > .8 or l < .2):
        # mid-lightness picture tints stay; everything structural flips
        if not (f in ('blue','green','violet') and .34 < l < .62 and s < .45):
            l = 1 - l * .94
    if f in t and f != 'grey':
        th, sm, lm = t[f]
        h = th / 360; s = min(1, s * sm)
        if lm: l = l * lm if l < .5 else l
    r, g, b = colorsys.hls_to_rgb(h, l, s)
    return round(r*255), round(g*255), round(b*255)
def run(name):
    t = THEMES[name]
    def hexsub(m):
        x = m.group(1)
        if len(x) == 3: x = ''.join(c*2 for c in x)
        a = x[6:] if len(x) == 8 else ''
        r, g, b = remap(int(x[0:2],16), int(x[2:4],16), int(x[4:6],16), t)
        return '#%02x%02x%02x%s' % (r, g, b, a)
    def rgbasub(m):
        r, g, b = remap(int(m.group(1)), int(m.group(2)), int(m.group(3)), t)
        return 'rgba(%d, %d, %d,%s)' % (r, g, b, m.group(4))
    out = re.sub(r'#([0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b', hexsub, src)
    out = re.sub(r'rgba\((\d+),\s*(\d+),\s*(\d+),([^)]*)\)', rgbasub, out)
    open(f'dir-3-{name}.html', 'w').write(out)
for n in THEMES: run(n)

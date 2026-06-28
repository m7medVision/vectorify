// Port of the Rust `category_color` (FNV-1a hash -> HSV) so feature colors match
// the old UI exactly. ponytail: uses charCodeAt, identical to Rust's byte hash for
// ASCII categories (the only kind the UI produces); non-ASCII would differ.
export function categoryColor(cat: string): [number, number, number] {
  let h = 2166136261 >>> 0;
  for (let i = 0; i < cat.length; i++) {
    h = (h ^ cat.charCodeAt(i)) >>> 0;
    h = Math.imul(h, 16777619) >>> 0;
  }
  return hsv(h % 360, 0.65, 0.9);
}

function hsv(h: number, s: number, v: number): [number, number, number] {
  const c = v * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = v - c;
  let r = 0,
    g = 0,
    b = 0;
  switch (Math.floor(h / 60)) {
    case 0: r = c; g = x; break;
    case 1: r = x; g = c; break;
    case 2: g = c; b = x; break;
    case 3: g = x; b = c; break;
    case 4: r = x; b = c; break;
    default: r = c; b = x;
  }
  return [
    Math.round((r + m) * 255),
    Math.round((g + m) * 255),
    Math.round((b + m) * 255),
  ];
}

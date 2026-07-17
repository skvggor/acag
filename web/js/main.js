// The omakase counter. The real acag engine (Rust → WebAssembly) plates every
// cover on the page: the same render_cover_svg the desktop app rasterizes,
// injected as inline SVG. Each round the house re-rolls the style, the plate
// morphs to the new aspect ratio, and the engine typesets the title live,
// character by character — auto-wrap, auto-size, WCAG AAA, all of it running in
// the browser. Reduced motion / no WASM / no GSAP fall back to the pre-rendered
// poster (the same config, rendered by the same code at build time).

const BASE_INK = { r: 0x16, g: 0x13, b: 0x10 };
const BG_MIX = 0.12; // how much of the theme's background tints the page ink
const TINT_MIN_LIGHTNESS = 0.58; // dark theme hues are lifted to stay legible
const DRIFT_SECONDS = 11; // idle time before the house plates the next cover
const FIRST_DRIFT_SECONDS = 6.5;
const TYPE_MIN_MS = 30;
const TYPE_JITTER_MS = 40;
const PARALLAX_PX = 7;
const PARALLAX_DEG = 1.3;
const GROUND_PX = 10; // the wagara ground slides against the plate for depth
const TILT_RANGE = 22; // degrees of device tilt mapped to full parallax
const WAVE_BASE_PX = 40; // matches .ground__wave in the stylesheet
const WAVE_SECONDS = 1.4;
const NOREN_MIN_MS = 700; // the curtain holds at least this long, so opening reads as staged
const RESCUE_MS = 7000; // mirrors the CSS reveal-rescue valve

// Mirrors `poster_config()` in examples/site.rs, so swapping the poster for the
// first live render is invisible.
const FIRST_PLATE = { title: 'Design systems that scale', category: 'engineering', number: '014' };
const FIRST_STYLE = { theme: 0, pattern: 0, layout: 0, format: 0, strength: 1, grain: 0.25 };
const BRAND = 'skvggor.dev';

// The house menu: titles the engine typesets while the visitor watches.
const PLATES = [
  FIRST_PLATE,
  { title: 'Performance without the magic', category: 'engineering', number: '021' },
  { title: 'The quiet art of refactoring legacy code', category: 'essays', number: '007' },
  { title: 'Ship less, design more', category: 'product', number: '032' },
  { title: 'Naming things is design work', category: 'design', number: '009' },
  { title: 'A field guide to code review', category: 'process', number: '017' },
  { title: 'Typography for terminal people', category: 'design', number: '026' },
  { title: 'What the compiler taught me about patience', category: 'essays', number: '041' },
];

// Page-chrome kanji only — the engine never draws Japanese glyphs on artwork.
const THEME_KANJI = {
  terracotta: '赤土',
  sumi: '墨',
  matcha: '抹茶',
  washi: '和紙',
  ai: '藍',
  sakura: '桜',
};

const root = document.documentElement;
const stage = document.getElementById('stage');
const card = document.getElementById('card');
const cover = document.getElementById('cover');
const poster = document.getElementById('poster');
const slug = document.getElementById('slug');
const slugTheme = document.getElementById('slug-theme');
const slugPattern = document.getElementById('slug-pattern');
const slugLayout = document.getElementById('slug-layout');
const slugFormat = document.getElementById('slug-format');
const themesGroup = document.getElementById('themes');
const omakaseButton = document.getElementById('omakase');
const titleInput = document.getElementById('title-input');
const hint = document.getElementById('hint');
const downloadToggle = document.getElementById('download-toggle');
const downloadsPanel = document.getElementById('downloads');
const downloadsClose = document.getElementById('downloads-close');
const ground = document.getElementById('ground');
const cursorElement = document.getElementById('cursor');
const noren = document.getElementById('noren');

const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
const hasGsap = typeof window.gsap !== 'undefined';
const animated = hasGsap && !reducedMotion;
const bootStarted = performance.now();

let renderCover = null;
let catalog = null;
let live = false;
let style = { ...FIRST_STYLE };
let plateIndex = 0;
let plate = PLATES[plateIndex];
let customTitle = null;
let typeCall = null;
let driftCall = null;
let renderQueued = false;

function clamp(value, low, high) {
  return value < low ? low : value > high ? high : value;
}

/* ---- Color math: tint the chrome from the engine's own palette ---- */

function hexToRgb(hex) {
  return {
    r: parseInt(hex.slice(1, 3), 16),
    g: parseInt(hex.slice(3, 5), 16),
    b: parseInt(hex.slice(5, 7), 16),
  };
}

function rgbToHex({ r, g, b }) {
  const part = (v) => Math.round(clamp(v, 0, 255)).toString(16).padStart(2, '0');
  return `#${part(r)}${part(g)}${part(b)}`;
}

function mix(a, b, amount) {
  return {
    r: a.r + (b.r - a.r) * amount,
    g: a.g + (b.g - a.g) * amount,
    b: a.b + (b.b - a.b) * amount,
  };
}

function rgbToHsl({ r, g, b }) {
  const rn = r / 255;
  const gn = g / 255;
  const bn = b / 255;
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const l = (max + min) / 2;
  if (max === min) return { h: 0, s: 0, l };
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h;
  if (max === rn) h = ((gn - bn) / d + (gn < bn ? 6 : 0)) / 6;
  else if (max === gn) h = ((bn - rn) / d + 2) / 6;
  else h = ((rn - gn) / d + 4) / 6;
  return { h, s, l };
}

function hslToRgb({ h, s, l }) {
  if (s === 0) return { r: l * 255, g: l * 255, b: l * 255 };
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;
  const channel = (t) => {
    let tc = t;
    if (tc < 0) tc += 1;
    if (tc > 1) tc -= 1;
    if (tc < 1 / 6) return p + (q - p) * 6 * tc;
    if (tc < 1 / 2) return q;
    if (tc < 2 / 3) return p + (q - p) * (2 / 3 - tc) * 6;
    return p;
  };
  return { r: channel(h + 1 / 3) * 255, g: channel(h) * 255, b: channel(h - 1 / 3) * 255 };
}

/** The theme's background hue, lifted enough to read on the dark chrome. */
function chromeTint(themeBackground) {
  const hsl = rgbToHsl(hexToRgb(themeBackground));
  if (hsl.l < TINT_MIN_LIGHTNESS) hsl.l = TINT_MIN_LIGHTNESS;
  return rgbToHex(hslToRgb(hsl));
}

function pageInk(themeBackground) {
  return rgbToHex(mix(BASE_INK, hexToRgb(themeBackground), BG_MIX));
}

/* ---- The engine ---- */

function currentTitle() {
  return customTitle !== null ? customTitle : plate.title;
}

function renderSvg(title, grain) {
  return renderCover(
    title,
    plate.category,
    '',
    plate.number,
    BRAND,
    style.theme,
    style.pattern,
    style.layout,
    style.format,
    style.strength,
    grain,
  );
}

function inject(title, grain = style.grain) {
  cover.innerHTML = renderSvg(title, grain);
}

/* ---- Plate geometry ---- */

function cardSize() {
  const box = stage.getBoundingClientRect();
  const slugSpace = slug.getBoundingClientRect().height + 18;
  const availableWidth = box.width * 0.97;
  const availableHeight = Math.max(120, box.height - slugSpace);
  const { width, height } = catalog.formats[style.format];
  const scale = Math.min(availableWidth / width, availableHeight / height);
  return { width: width * scale, height: height * scale };
}

function applySize(immediate) {
  const size = cardSize();
  if (animated && !immediate) {
    return window.gsap.to(card, {
      width: size.width,
      height: size.height,
      duration: 0.75,
      ease: 'power3.inOut',
    });
  }
  card.style.width = `${size.width}px`;
  card.style.height = `${size.height}px`;
  card.style.aspectRatio = 'auto';
  return null;
}

/* ---- Chrome tinting ---- */

function applyTint(immediate) {
  const theme = catalog.themes[style.theme];
  const values = { '--tint': chromeTint(theme.bg), '--bg-tint': pageInk(theme.bg) };
  if (animated && !immediate) {
    window.gsap.to(root, { ...values, duration: 0.75, ease: 'power2.inOut' });
    window.gsap.to(card, { backgroundColor: theme.bg, duration: 0.75, ease: 'power2.inOut' });
    return;
  }
  root.style.setProperty('--tint', values['--tint']);
  root.style.setProperty('--bg-tint', values['--bg-tint']);
  card.style.backgroundColor = theme.bg;
}

function updateSlug() {
  const theme = catalog.themes[style.theme];
  const kanji = THEME_KANJI[theme.label];
  slugTheme.innerHTML = kanji ? `${theme.label} <span lang="ja">${kanji}</span>` : theme.label;
  slugPattern.textContent = catalog.patterns[style.pattern];
  slugLayout.textContent = catalog.layouts[style.layout];
  slugFormat.textContent = catalog.formats[style.format].label.replace(' · ', ' ');
  for (const button of themesGroup.children) {
    button.setAttribute('aria-pressed', String(Number(button.dataset.theme) === style.theme));
  }
}

/* ---- The typewriter: the engine typesets while you watch ---- */

function cancelType() {
  if (typeCall) {
    typeCall.kill();
    typeCall = null;
  }
}

function typeTitle(title, onDone) {
  cancelType();
  let position = 0;
  const step = () => {
    position += 1;
    const done = position >= title.length;
    // Grain only lands with the final stroke; feTurbulence per keystroke is waste.
    inject(title.slice(0, position), done ? style.grain : 0);
    if (done) {
      typeCall = null;
      onDone();
      return;
    }
    typeCall = window.gsap.delayedCall((TYPE_MIN_MS + Math.random() * TYPE_JITTER_MS) / 1000, step);
  };
  step();
}

/* ---- Drift: left idle, the house keeps plating ---- */

function killDrift() {
  if (driftCall) {
    driftCall.kill();
    driftCall = null;
  }
}

function scheduleDrift(seconds = DRIFT_SECONDS) {
  killDrift();
  if (!animated || customTitle !== null) return;
  driftCall = window.gsap.delayedCall(seconds, () => omakase());
}

/* ---- Omakase: re-roll the style, morph the plate, retype the title ---- */

function nextStyle() {
  const random = (length) => Math.floor(Math.random() * length);
  let theme = random(catalog.themes.length);
  while (theme === style.theme) theme = random(catalog.themes.length);
  return {
    theme,
    pattern: random(catalog.patterns.length),
    layout: random(catalog.layouts.length),
    format: random(catalog.formats.length),
    strength: 0.7 + Math.random() * 0.3,
    grain: Math.random() < 0.5 ? 0.18 + Math.random() * 0.22 : 0,
  };
}

function omakase() {
  if (!live) return;
  cancelType();
  killDrift();
  style = nextStyle();
  if (customTitle === null) {
    plateIndex = (plateIndex + 1) % PLATES.length;
    plate = PLATES[plateIndex];
  }

  if (!animated) {
    applyTint(true);
    applySize(true);
    inject(currentTitle());
    updateSlug();
    return;
  }

  const retype = customTitle === null;
  const timeline = window.gsap.timeline();
  timeline.to(cover, { opacity: 0, y: 12, duration: 0.38, ease: 'power2.in' }, 0);
  timeline.to(slug, { opacity: 0, duration: 0.3, ease: 'power2.in' }, 0);
  timeline.add(() => {
    applyTint();
    applySize();
  }, 0.18);
  timeline.add(() => {
    window.gsap.set(cover, { y: 0 });
    updateSlug();
    window.gsap.to(slug, { opacity: 1, duration: 0.5, ease: 'power2.out' });
    window.gsap.to(cover, { opacity: 1, duration: 0.4, ease: 'power2.out' });
    if (retype) {
      typeTitle(currentTitle(), () => scheduleDrift());
    } else {
      inject(currentTitle());
      scheduleDrift();
    }
  }, 1.0);
}

/** A lighter round for direct theme choices: same plate, new ink. */
function restyleTheme(themeIndex) {
  if (!live || themeIndex === style.theme) return;
  cancelType();
  style.theme = themeIndex;
  if (!animated) {
    applyTint(true);
    inject(currentTitle());
    updateSlug();
    return;
  }
  applyTint();
  window.gsap.to(cover, {
    opacity: 0,
    duration: 0.2,
    ease: 'power2.in',
    onComplete: () => {
      inject(currentTitle());
      updateSlug();
      window.gsap.to(cover, { opacity: 1, duration: 0.3, ease: 'power2.out' });
    },
  });
  scheduleDrift();
}

/* ---- Controls ---- */

function buildThemeStamps() {
  catalog.themes.forEach((theme, index) => {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'theme';
    button.dataset.theme = String(index);
    button.style.backgroundColor = theme.bg;
    button.style.color = theme.text;
    button.setAttribute('aria-pressed', String(index === style.theme));
    button.setAttribute('aria-label', `${theme.label} theme`);
    button.title = theme.label;
    const kanji = document.createElement('span');
    kanji.lang = 'ja';
    kanji.textContent = THEME_KANJI[theme.label] ?? theme.label.slice(0, 1);
    button.append(kanji);
    button.addEventListener('click', () => restyleTheme(index));
    themesGroup.append(button);
  });
}

function queueCustomRender() {
  if (renderQueued) return;
  renderQueued = true;
  requestAnimationFrame(() => {
    renderQueued = false;
    inject(currentTitle());
  });
}

function wireControls() {
  card.addEventListener('click', () => omakase());
  omakaseButton.addEventListener('click', () => omakase());

  titleInput.addEventListener('input', () => {
    const value = titleInput.value.trim();
    if (value) {
      customTitle = value;
      killDrift();
      cancelType();
      queueCustomRender();
    } else {
      customTitle = null;
      inject(currentTitle());
      scheduleDrift();
    }
  });
}

/* ---- Downloads drawer ---- */

function wireDownloads() {
  const close = () => {
    downloadsPanel.hidden = true;
    downloadToggle.setAttribute('aria-expanded', 'false');
    document.body.classList.remove('downloads-open');
  };
  downloadToggle.addEventListener('click', () => {
    const open = downloadsPanel.hidden;
    downloadsPanel.hidden = !open;
    downloadToggle.setAttribute('aria-expanded', String(open));
    document.body.classList.toggle('downloads-open', open);
    if (open) downloadsClose.focus();
  });
  downloadsClose.addEventListener('click', () => {
    close();
    downloadToggle.focus();
  });
  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && !downloadsPanel.hidden) {
      close();
      downloadToggle.focus();
    }
  });
}

/* ---- Motion: the sheet answers the hand (pointer) or the wrist (gyro) ---- */

let applyDrift = null;

function wireMotion() {
  if (!animated) return;

  const cardX = window.gsap.quickTo(card, 'x', { duration: 0.7, ease: 'power2.out' });
  const cardY = window.gsap.quickTo(card, 'y', { duration: 0.7, ease: 'power2.out' });
  const cardTiltX = window.gsap.quickTo(card, 'rotationX', { duration: 0.7, ease: 'power2.out' });
  const cardTiltY = window.gsap.quickTo(card, 'rotationY', { duration: 0.7, ease: 'power2.out' });
  const groundX = window.gsap.quickTo(ground, 'x', { duration: 1.1, ease: 'power2.out' });
  const groundY = window.gsap.quickTo(ground, 'y', { duration: 1.1, ease: 'power2.out' });
  window.gsap.set(card, { transformPerspective: 900 });

  applyDrift = (nx, ny) => {
    cardX(nx * PARALLAX_PX);
    cardY(ny * PARALLAX_PX * 0.7);
    cardTiltY(nx * PARALLAX_DEG);
    cardTiltX(-ny * PARALLAX_DEG);
    // The ground slides the other way, so the sheet reads as layered depth.
    groundX(-nx * GROUND_PX);
    groundY(-ny * GROUND_PX);
  };

  window.addEventListener('pointermove', (event) => {
    applyDrift(
      (event.clientX / window.innerWidth) * 2 - 1,
      (event.clientY / window.innerHeight) * 2 - 1,
    );
  });

  // Every press sends a wave through the cloth — ink meeting water.
  window.addEventListener('pointerdown', (event) => pressWave(event.clientX, event.clientY));

  bindTilt();
}

/** A ripple through the seigaiha ground, radiating from the press point. */
function pressWave(x, y) {
  const rect = ground.getBoundingClientRect();
  const wave = document.createElement('span');
  wave.className = 'ground__wave';
  ground.append(wave);
  const reach = Math.hypot(
    Math.max(x, window.innerWidth - x),
    Math.max(y, window.innerHeight - y),
  );
  // The gradient ring sits at 80% of the radius; scale it past the far corner.
  const scale = (reach * 1.15) / (WAVE_BASE_PX * 0.4);
  window.gsap.set(wave, { x: x - rect.left, y: y - rect.top });
  window.gsap.fromTo(
    wave,
    { scale: 0, opacity: 0.55 },
    {
      scale,
      opacity: 0,
      duration: WAVE_SECONDS,
      ease: 'power2.out',
      onComplete: () => wave.remove(),
    },
  );
}

/* Device tilt: gamma rolls left/right; beta rests near 45° in a held hand. */
function handleOrientation(event) {
  if ((event.gamma == null && event.beta == null) || !applyDrift) return;
  applyDrift(
    clamp((event.gamma || 0) / TILT_RANGE, -1, 1),
    clamp(((event.beta || 45) - 45) / TILT_RANGE, -1, 1),
  );
}

function enableTilt() {
  window.addEventListener('deviceorientation', handleOrientation, { passive: true });
}

function bindTilt() {
  const Sensor = window.DeviceOrientationEvent;
  if (!Sensor) return;
  if (typeof Sensor.requestPermission === 'function') {
    // iOS gates the motion sensor behind a user gesture.
    window.addEventListener(
      'pointerdown',
      () => {
        Sensor.requestPermission()
          .then((permission) => {
            if (permission === 'granted') enableTilt();
          })
          .catch(() => {});
      },
      { once: true },
    );
  } else {
    enableTilt();
  }
}

/* ---- The brush cursor: registration ring and motion stroke ---- */

function wireCursor() {
  if (!animated) return;
  if (!window.matchMedia('(pointer: fine)').matches) return;
  root.classList.add('cursor-live');

  const ring = cursorElement.querySelector('.cursor__ring');
  const bar = cursorElement.querySelector('.cursor__bar');
  const moveX = window.gsap.quickTo(cursorElement, 'x', { duration: 0.16, ease: 'power3.out' });
  const moveY = window.gsap.quickTo(cursorElement, 'y', { duration: 0.16, ease: 'power3.out' });

  // The stroke: its angle follows the hand's motion, its presence the speed.
  const stroke = { angle: 0, alpha: 0, reach: 12 };
  window.gsap.ticker.add(() => {
    stroke.alpha *= 0.9;
    const radians = (stroke.angle * Math.PI) / 180;
    window.gsap.set(bar, {
      rotation: stroke.angle,
      x: -Math.cos(radians) * stroke.reach,
      y: -Math.sin(radians) * stroke.reach,
      opacity: stroke.alpha,
    });
  });

  let shown = false;
  let last = null;
  window.addEventListener('pointermove', (event) => {
    moveX(event.clientX);
    moveY(event.clientY);
    if (last) {
      const dx = event.clientX - last.x;
      const dy = event.clientY - last.y;
      const speed = Math.hypot(dx, dy);
      if (speed > 2) {
        stroke.angle = (Math.atan2(dy, dx) * 180) / Math.PI;
        stroke.alpha = Math.min(0.95, speed / 26);
        stroke.reach = 12 + Math.min(10, speed * 0.35);
      }
    }
    last = { x: event.clientX, y: event.clientY };
    if (!shown) {
      shown = true;
      window.gsap.set(cursorElement, { x: event.clientX, y: event.clientY });
      window.gsap.to(cursorElement, { opacity: 1, duration: 0.35 });
    }
  });

  // The ring answers what it is over: a turned seal over anything pressable,
  // a thin brush tip over text entry.
  const setRing = (state) => {
    const shapes = {
      idle: { scaleX: 1, scaleY: 1, rotation: 0, borderRadius: '50%' },
      hover: { scaleX: 1.4, scaleY: 1.4, rotation: 45, borderRadius: '20%' },
      text: { scaleX: 0.14, scaleY: 1.15, rotation: 0, borderRadius: '2px' },
    };
    window.gsap.to(ring, { ...shapes[state], duration: 0.28, ease: 'power3.out', overwrite: 'auto' });
  };
  window.addEventListener('pointerover', (event) => {
    const hit = event.target.closest('a, button, input, [role="button"]');
    setRing(!hit ? 'idle' : hit.matches('input, textarea') ? 'text' : 'hover');
  });

  // Pressing squeezes the whole cursor, like a seal meeting paper.
  window.addEventListener('pointerdown', () => {
    window.gsap.to(cursorElement, { scale: 0.72, duration: 0.12, ease: 'power2.out' });
  });
  window.addEventListener('pointerup', () => {
    window.gsap.to(cursorElement, { scale: 1, duration: 0.35, ease: 'back.out(2.5)' });
  });

  document.documentElement.addEventListener('mouseleave', () => {
    shown = false;
    window.gsap.to(cursorElement, { opacity: 0, duration: 0.3 });
  });
}

/* ---- Boot ---- */

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Wait for `promise`, but never longer than `ms`; failures count as settled. */
function settled(promise, ms) {
  return Promise.race([Promise.resolve(promise).catch(() => {}), delay(ms)]);
}

function reveal() {
  const items = document.querySelectorAll('[data-reveal]');
  // Past the CSS rescue valve the items are already visible; don't re-hide.
  const rescued = performance.now() - bootStarted > RESCUE_MS;
  if (!animated || rescued) {
    for (const item of items) item.style.opacity = 1;
    return;
  }
  window.gsap.set(items, { y: 10, opacity: 0 });
  // clearProps: a leftover transform would turn the downloads wrap into the
  // containing block of its position: fixed bottom sheet on small viewports.
  window.gsap.to(items, {
    y: 0,
    opacity: 1,
    duration: 0.8,
    ease: 'power2.out',
    stagger: 0.12,
    delay: 0.3,
    clearProps: 'transform',
  });
}

/** The curtain parts from the center out; the crest bows out first. */
function liftNoren() {
  if (!animated) {
    noren.remove();
    return;
  }
  const bands = noren.querySelectorAll('.noren__band');
  const crest = noren.querySelector('.noren__crest');
  const timeline = window.gsap.timeline({ onComplete: () => noren.remove() });
  timeline.to(crest, { opacity: 0, scale: 0.9, duration: 0.3, ease: 'power2.in' }, 0);
  timeline.to(
    bands,
    {
      yPercent: -103,
      duration: 0.9,
      ease: 'power3.inOut',
      stagger: { each: 0.07, from: 'center' },
    },
    0.12,
  );
}

/** The house opens: fonts settle, the curtain holds a beat, then parts. */
async function openHouse() {
  await settled(document.fonts.ready, 1500);
  const elapsed = performance.now() - bootStarted;
  if (elapsed < NOREN_MIN_MS) await delay(NOREN_MIN_MS - elapsed);
  liftNoren();
  reveal();
}

async function withoutLiveEngine() {
  root.classList.add('no-live');
  hint.textContent = 'The live engine needs WebAssembly — this plate was pre-rendered by the same code.';
  await settled(poster.decode(), 1200);
  await openHouse();
}

async function boot() {
  if (hasGsap) window.gsap.ticker.lagSmoothing(0);

  let module;
  try {
    module = await import('../wasm/acag.js');
    await module.default();
  } catch (error) {
    console.warn('acag wasm unavailable:', error);
    await withoutLiveEngine();
    return;
  }

  renderCover = module.render_cover;
  catalog = JSON.parse(module.catalog());
  live = true;

  buildThemeStamps();
  applySize(true);
  applyTint(true);
  inject(currentTitle());
  updateSlug();
  poster.hidden = true;

  wireControls();
  window.addEventListener('resize', () => {
    if (live) applySize(true);
  });

  await openHouse();
  scheduleDrift(FIRST_DRIFT_SECONDS);
}

wireDownloads();
// Motion and the cursor are page furniture, not engine features — they live
// even when WebAssembly is unavailable and the poster stands in.
wireMotion();
wireCursor();
boot();

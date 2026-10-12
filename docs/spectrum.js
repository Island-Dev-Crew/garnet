'use strict';
// The spectrum: one ray per kind of authority. It draws what the live checker
// reports for the workbench above it (declared, reached without a declaration,
// newly gained) and never infers a verdict of its own.
(() => {
  const figure = document.getElementById('spectrum');
  const canvas = document.getElementById('spectrum-canvas');
  if (!figure || !canvas || !canvas.getContext) return;
  const ctx = canvas.getContext('2d');
  const CAPS = ['fs', 'net', 'net_internal', 'proc', 'env', 'time', 'mem', 'ffi', '*'];
  const PULSE_MS = 320;
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  const state = { declared: new Set(), flagged: new Set() };
  const gem = new Image();
  gem.decoding = 'async';
  gem.src = 'assets/garnet-stone-cutout.webp';
  let visible = false, queued = false;

  const token = name => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  const still = () => reducedMotion.matches || document.body.classList.contains('motion-off');

  function layout(w, h) {
    if (w < 640) {
      // Phones: rays fan along an arc below the stone, so no ray crosses a label.
      const gx = w / 2, gy = h * 0.3, size = Math.min(w * 0.4, 160);
      const rx = Math.min(w * 0.34, 200), ry = Math.min(h * 0.52, rx * 1.3);
      const ends = CAPS.map((_, i) => {
        const angle = (155 - i * (130 / (CAPS.length - 1))) * Math.PI / 180;
        return { x: gx + rx * Math.cos(angle), y: gy + ry * Math.sin(angle), cos: Math.cos(angle) };
      });
      return { narrow: true, gx, gy, size, from: { x: gx, y: 0 }, ends };
    }
    const gx = w * 0.36, gy = h * 0.5, size = Math.min(h * 0.86, w * 0.26);
    const top = h * 0.08, bottom = h * 0.92;
    const ends = CAPS.map((_, i) => ({ x: w - 150, y: top + (bottom - top) * (i / (CAPS.length - 1)), cos: 1 }));
    return { narrow: false, gx, gy, size, from: { x: 0, y: gy }, ends };
  }

  function draw(now) {
    queued = false;
    const dpr = Math.min(devicePixelRatio || 1, 2);
    const w = canvas.clientWidth, h = canvas.clientHeight;
    if (!w || !h) return;
    if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    const L = layout(w, h);
    const gold = token('--gold'), bright = token('--bright'), muted = token('--muted'), mono = token('--mono');
    const beam = ctx.createLinearGradient(L.from.x, L.from.y, L.gx, L.gy);
    beam.addColorStop(0, 'rgba(215,181,114,0)');
    beam.addColorStop(1, gold);
    ctx.strokeStyle = beam;
    ctx.lineWidth = 1.5;
    ctx.beginPath(); ctx.moveTo(L.from.x, L.from.y); ctx.lineTo(L.gx, L.gy); ctx.stroke();
    const pulse = still() ? 1 : 0.55 + 0.45 * Math.sin(now / PULSE_MS);
    CAPS.forEach((cap, i) => {
      const end = L.ends[i], flagged = state.flagged.has(cap), lit = state.declared.has(cap);
      ctx.lineWidth = flagged || lit ? 1.8 : 1;
      ctx.strokeStyle = flagged ? bright : lit ? gold : 'rgba(242,232,216,0.2)';
      ctx.globalAlpha = flagged ? pulse : 1;
      ctx.shadowColor = flagged ? bright : gold;
      ctx.shadowBlur = flagged || lit ? 14 : 0;
      ctx.beginPath(); ctx.moveTo(L.gx, L.gy); ctx.lineTo(end.x, end.y); ctx.stroke();
      ctx.shadowBlur = 0;
      ctx.globalAlpha = 1;
      ctx.fillStyle = flagged ? bright : lit ? gold : muted;
      ctx.font = (flagged || lit ? '600 ' : '400 ') + '13px ' + mono;
      const side = L.narrow ? (end.cos > 0.25 ? 1 : end.cos < -0.25 ? -1 : 0) : 1;
      ctx.textAlign = side > 0 ? 'left' : side < 0 ? 'right' : 'center';
      ctx.textBaseline = side ? 'middle' : 'top';
      // Keep every label inside the canvas, even on the narrowest phones.
      const width = ctx.measureText(cap).width;
      let x = end.x + side * (L.narrow ? 9 : 12);
      if (side < 0) x = Math.max(x, width + 2);
      else if (side > 0) x = Math.min(x, w - width - 2);
      else x = Math.min(Math.max(x, width / 2 + 2), w - width / 2 - 2);
      ctx.fillText(cap, x, side ? end.y : end.y + 9);
      ctx.beginPath(); ctx.arc(end.x, end.y, flagged || lit ? 3.5 : 2, 0, Math.PI * 2); ctx.fill();
    });
    if (gem.complete && gem.naturalWidth) {
      const gw = L.size, gh = gw * gem.naturalHeight / gem.naturalWidth;
      ctx.drawImage(gem, L.gx - gw / 2, L.gy - gh / 2, gw, gh);
    }
    if (visible && state.flagged.size && !still()) request();
  }
  function request() { if (!queued) { queued = true; requestAnimationFrame(draw); } }

  function show({ program = '', declared = [], flagged = [], message = '' } = {}) {
    state.declared = new Set(declared.filter(cap => CAPS.includes(cap)));
    state.flagged = new Set(flagged.filter(cap => CAPS.includes(cap)));
    document.getElementById('spectrum-program').textContent = program;
    document.getElementById('spectrum-state').textContent = message;
    canvas.setAttribute('aria-label', 'Authority spectrum. ' + message);
    request();
  }

  addEventListener('garnet:spectrum', event => show(event.detail));
  addEventListener('resize', request);
  gem.addEventListener('load', request);
  reducedMotion.addEventListener('change', request);
  new MutationObserver(request).observe(document.body, { attributes: true, attributeFilter: ['class'] });
  if ('IntersectionObserver' in window) {
    new IntersectionObserver(entries => { visible = entries.some(entry => entry.isIntersecting); request(); }).observe(canvas);
  } else {
    visible = true;
  }
  request();
})();

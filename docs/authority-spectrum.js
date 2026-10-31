/* The picture follows returned checker data. It never grants or executes authority. */
(() => {
  'use strict';
  const kinds = Object.freeze(['fs', 'net', 'net_internal', 'proc', 'env', 'time', 'mem', 'ffi', '*']);
  const labels = values => [...new Set((Array.isArray(values) ? values : []).filter(v => kinds.includes(v)))];
  const empty = (phase = 'idle', message = 'Run a check. Watch its authority come into view.') => ({phase, kind: 'none', declared: [], missing: [], gained: [], program: 'Awaiting checker output', message});
  let latest = empty();

  function fromCheck(check, surface) {
    if (check?.schema !== 'garnet.wasm.check/1' || typeof check.ok !== 'boolean' || !Array.isArray(check.diagnostics) || surface?.schema !== 'garnet.wasm.diff-caps/1' || surface.ok !== true || !Array.isArray(surface.new_surface?.aggregate)) return empty('error', 'No valid checker result. No authority verdict is shown.');
    const declared = labels(surface.new_surface?.aggregate);
    const missing = labels(check.diagnostics.filter(d => d.code === 'check.caps_coverage').map(d => /does not declare `([^`]+)`/.exec(d.message)?.[1]));
    return {phase: check.ok ? 'passed' : 'rejected', kind: 'check', declared, missing, gained: [], program: '@caps(' + declared.join(', ') + ')', message: check.ok ? 'Declared authority. This source check passes.' : missing.length ? missing.join(', ') + ' is reached without a declaration. Check rejected.' : 'Check rejected. Inspect its diagnostics.'};
  }

  function fromDiff(diff) {
    if (diff?.schema !== 'garnet.wasm.diff-caps/1' || diff.ok !== true || typeof diff.authority_expanded !== 'boolean' || !Array.isArray(diff.new_surface?.aggregate) || !Array.isArray(diff.aggregate_added) || !Array.isArray(diff.functions_caps_expanded)) return empty('error', 'No valid authority diff. No expansion verdict is shown.');
    const declared = labels(diff.new_surface?.aggregate), gained = labels([...(diff.aggregate_added || []), ...(diff.functions_caps_expanded || []).flatMap(f => f.gained || [])]);
    return {phase: diff.authority_expanded ? 'expanded' : 'unchanged', kind: 'diff', declared, gained, missing: [], program: '@caps(' + declared.join(', ') + ')', message: diff.authority_expanded ? (gained.length ? 'New declaration: ' + gained.join(', ') + '. ' : '') + 'diff-caps reports authority expanded.' : 'No declared authority was added.'};
  }

  // `announce: false` leaves speaking to the page's own announcer (the front door has one).
  function markup(id, {announce = true} = {}) {
    const wide = kinds.map((cap, i) => {
      const y = 24 + i * 35;
      return `<g data-spectrum-band="${cap}" style="--i:${i}"><path class="spectrum-ray" d="M260 164L626 ${y}"/><circle class="spectrum-end" cx="626" cy="${y}" r="3"/><text x="645" y="${y + 6}" class="spectrum-name">${cap}</text><text x="798" y="${y + 5}" text-anchor="end" class="spectrum-mark"></text></g>`;
    }).join('');
    const positions = [[50,180,38,176,'end'],[75,219,65,236,'end'],[110,260,88,282,'middle'],[147,287,137,312,'end'],[180,300,180,336,'middle'],[212,287,223,312,'start'],[249,260,265,282,'start'],[285,219,295,236,'start'],[310,180,322,176,'start']];
    const narrow = kinds.map((cap, i) => {
      const [x,y,tx,ty,anchor] = positions[i];
      return `<g data-spectrum-band="${cap}" style="--i:${i}"><path class="spectrum-ray" d="M180 95L${x} ${y}"/><circle class="spectrum-end" cx="${x}" cy="${y}" r="3"/><text class="spectrum-name" x="${tx}" y="${ty}" text-anchor="${anchor}">${cap}</text></g>`;
    }).join('');
    return `<figure class="authority-spectrum" id="${id}" data-phase="idle" aria-labelledby="${id}-caption"><div class="spectrum-visual"><svg class="spectrum-wide" viewBox="0 0 820 330" role="img" aria-label="Authority spectrum. Awaiting checker output."><path class="spectrum-input" d="M0 164H260"/>${wide}<image data-href="assets/garnet-spectrum.webp" x="130" y="33" width="260" height="260" aria-hidden="true"/></svg><svg class="spectrum-narrow" viewBox="0 0 360 360" role="img" aria-label="Authority spectrum. Awaiting checker output."><path class="spectrum-input" d="M180 0V95"/>${narrow}<image data-href="assets/garnet-spectrum.webp" x="105" y="20" width="150" height="150" aria-hidden="true"/></svg></div><figcaption id="${id}-caption"><code class="spectrum-program">Awaiting checker output</code><p class="spectrum-status"${announce ? ' role="status" aria-live="polite"' : ''}>Run a check. Watch its authority come into view.</p></figcaption><div class="spectrum-key" aria-label="Spectrum legend"><span><i class="key-declared" aria-hidden="true"></i>Declared</span><span><i class="key-attention" aria-hidden="true"></i>Missing or newly gained</span><span>Live checker</span></div></figure>`;
  }

  const verdicts = new Set(['passed', 'rejected', 'expanded', 'unchanged']);
  function paint(figure, result) {
    // Restart the refraction on each fresh verdict; any other phase rests.
    delete figure.dataset.fresh;
    if (verdicts.has(result.phase)) { void figure.getBoundingClientRect(); figure.dataset.fresh = ''; }
    figure.dataset.phase = result.phase;
    figure.dataset.resultKind = result.kind;
    figure.querySelector('.spectrum-program').textContent = result.program;
    figure.querySelector('.spectrum-status').textContent = result.message;
    const names = [];
    figure.querySelectorAll('[data-spectrum-band]').forEach(band => {
      const cap = band.dataset.spectrumBand;
      const status = result.missing.includes(cap) ? 'missing' : result.gained.includes(cap) ? 'gained' : result.declared.includes(cap) ? 'declared' : 'inactive';
      band.dataset.state = status;
      const mark = band.querySelector('.spectrum-mark');
      if (mark) mark.textContent = status === 'missing' ? '!' : status === 'gained' ? '+' : status === 'declared' ? '✓' : '';
      if (status !== 'inactive') names.push(cap + ': ' + status);
    });
    figure.querySelectorAll('svg').forEach(svg => svg.setAttribute('aria-label', 'Authority spectrum. ' + result.message + ' ' + [...new Set(names)].join('. ')));
  }

  function set(result) {
    latest = {...result, declared: [...result.declared], missing: [...result.missing], gained: [...result.gained]};
    document.querySelectorAll('[data-spectrum-live] .authority-spectrum').forEach(figure => paint(figure, latest));
  }
  function clear(message) { set(empty('idle', message)); }
  // The gemstone sits below the fold; fetch it as the figure approaches.
  function lazyArt(root) {
    const images = [...root.querySelectorAll('image[data-href]')];
    const show = () => images.forEach(image => { image.setAttribute('href', image.dataset.href); image.removeAttribute('data-href'); });
    if (!('IntersectionObserver' in window)) return show();
    const watch = new IntersectionObserver(entries => { if (entries.some(entry => entry.isIntersecting)) { watch.disconnect(); show(); } }, {rootMargin: '600px 0px'});
    watch.observe(root);
  }
  async function mountReference() {
    const host = document.querySelector('[data-spectrum-reference]');
    if (!host) return;
    let serial = 0, module = null, attempt = 0, loading = null;
    // One load at a time, bounded like the front door's; a timeout or failure moves the next
    // attempt to a fresh module instance.
    const load = () => loading ??= (async () => {
      let timer;
      try {
        const api = await Promise.race([(async () => { const m = await import('./playground/pkg/garnet_wasm.js' + (attempt ? '?retry=' + attempt : '')); await m.default({module_or_path: 'playground/pkg/garnet_wasm_bg.wasm'}); return m; })(), new Promise((_, reject) => { timer = setTimeout(() => reject(Error('Checker load timed out')), 20000); })]);
        module = api; return api;
      } catch (error) { attempt++; loading = null; throw error; }
      finally { clearTimeout(timer); }
    })();
    const before = '@caps()\ndef main() {\n  println("report ready")\n  0\n}';
    const after = '@caps(net)\ndef main() {\n  println("report ready")\n  0\n}';
    const file = declared => '@caps(' + (declared ? 'fs' : '') + ')\ndef main() {\n  fs::write_file("report.txt", "Ready for review")\n  0\n}';
    const raw = host.querySelector('.spectrum-result-json'), code = host.querySelector('.spectrum-source'), buttons = [...host.querySelectorAll('[data-spectrum-example]')];
    host.querySelector('[data-spectrum-mount]').innerHTML = markup('reference-spectrum');
    const figure = host.querySelector('.authority-spectrum');
    lazyArt(figure);
    code.textContent = file(false);
    paint(figure, empty('idle', 'Use an example to inspect its authority.'));
    for (const button of buttons) button.addEventListener('click', async () => {
      const current = ++serial, choice = button.dataset.spectrumExample;
      host.querySelector('.spectrum-delta').hidden = choice !== 'diff';
      buttons.forEach(b => b.setAttribute('aria-pressed', String(b === button)));
      const pending = empty('checking', 'Checking the source…');
      paint(figure, pending); raw.textContent = 'Awaiting actual checker output.';
      code.textContent = choice === 'diff' ? '# before.garnet\n' + before + '\n\n# after.garnet\n' + after : file(choice === 'declared');
      try {
        if (!module) await load();
        if (current !== serial) return;
        if (choice === 'diff') {
          const diff = JSON.parse(module.diff_caps_source(before, after));
          paint(figure, fromDiff(diff)); raw.textContent = JSON.stringify(diff, null, 2);
        } else {
          const source = file(choice === 'declared'), check = JSON.parse(module.check_source(source)), surface = JSON.parse(module.diff_caps_source('', source));
          paint(figure, fromCheck(check, surface)); raw.textContent = JSON.stringify({check, surface}, null, 2);
        }
      } catch {
        if (current !== serial) return;
        paint(figure, empty('error', 'Checker unavailable. No verdict was produced. Retry an example.'));
        raw.textContent = 'No result. The visual does not substitute recorded output.';
      }
    });
  }

  window.GarnetSpectrum = Object.freeze({kinds, fromCheck, fromDiff, markup, paint, set, clear, snapshot: () => ({...latest, declared: [...latest.declared], missing: [...latest.missing], gained: [...latest.gained]})});
  const mount = () => {
    document.querySelectorAll('[data-spectrum-live] [data-spectrum-mount]').forEach(slot => { slot.innerHTML = markup('live-spectrum', {announce: false}); lazyArt(slot.firstElementChild); });
    document.querySelectorAll('[data-spectrum-live] .authority-spectrum').forEach(figure => paint(figure, latest));
    void mountReference();
  };
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', mount, {once:true}); else mount();
})();

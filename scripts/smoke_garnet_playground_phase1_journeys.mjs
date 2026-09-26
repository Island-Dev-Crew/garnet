// Shared by the committed proof and the development smoke. Assertions inspect
// actual controls and adapter results; preset output is never treated as proof.
export async function phase1Journeys(page, baseUrl, equal) {
  page.setDefaultTimeout(4000);
  const pure = '@caps()\ndef main() { println("hello") 0 }\n';
  await page.locator('#source-editor').fill(pure);
  await page.locator('#run-source').click();
  await page.locator('#run-json-toggle').click();
  equal(JSON.parse(await page.locator('#run-json').textContent()).exit_class, 'ok', 'readable/raw run');
  await page.locator('#check-source').click();
  equal(await page.locator('#check-result').textContent(), 'No checker diagnostics.', 'readable check');
  await page.locator('#baseline-editor').fill(pure);
  await page.locator('#diff-caps').click();
  equal(await page.locator('#lane-card').getAttribute('data-state'), 'unknown', 'unchanged coverage unknown');
  if (!(await page.locator('#lane-card').textContent()).includes('this is not merge approval')) throw Error('lane approval caveat absent');
  equal(await page.locator('#authority-rows tr').count(), 1, 'annotated function row');
  await page.locator('#source-editor').fill('@caps(fs)\ndef main() { 0 }');
  await page.locator('#diff-caps').click();
  equal(await page.locator('#lane-card').getAttribute('data-state'), 'review', 'widening human review');
  await page.locator('#source-editor').fill('@caps()\ndef main() { time::now_ms() }');
  await page.locator('#diff-caps').click();
  equal(await page.locator('#lane-card').getAttribute('data-state'), 'blocked', 'checker invalid blocked');
  await page.locator('#source-editor').fill('def main(');
  await page.locator('#diff-caps').click();
  equal(await page.locator('#lane-card').getAttribute('data-state'), 'blocked', 'parse invalid blocked');
  await page.locator('#source-editor').fill('def helper() { 0 }\n@caps()\ndef main() { helper() }');
  await page.locator('#diff-caps').click();
  equal(await page.locator('#lane-card').getAttribute('data-state'), 'unknown', 'unannotated coverage unknown');
  await page.locator('#baseline-editor').fill('@caps(net)\ndef f() { 0 }\n@caps()\ndef main() { 0 }');
  await page.locator('#source-editor').fill('@caps(net)\ndef f() { 0 }\n@caps(net)\ndef main() { 0 }');
  await page.locator('#diff-caps').click();
  equal(await page.locator('#lane-card').getAttribute('data-state'), 'review', 'per-function gain requires review');
  equal(await page.evaluate(() => window.__garnetPlayground.state.lastDiff.authority_expanded), false, 'S37 aggregate rule unchanged');
  // T5a (C5-08): a new capability-bearing function under an existing aggregate
  // goes to review, and a check failure never leaves the diff label green.
  await page.locator('#baseline-editor').fill('@caps(fs)\ndef main() { 0 }');
  await page.locator('#source-editor').fill('@caps(fs)\ndef exfil() { 0 }\n@caps(fs)\ndef main() { 0 }');
  await page.locator('#diff-caps').click();
  equal(await page.locator('#lane-card').getAttribute('data-state'), 'review', 'new fs function under existing aggregate requires review');
  if (!(await page.locator('#lane-card').textContent()).includes('exfil added with fs')) throw Error('new function not named on the review card');
  // T5a (Codex review of #598): the checker accepts a repeated name, so a new
  // name declared twice routes to review if ANY entry declares a capability,
  // in either order, while the program-wide aggregate stays unchanged.
  for (const order of ['@caps(fs)\ndef extra() { 0 }\n@caps()\ndef extra() { 0 }', '@caps()\ndef extra() { 0 }\n@caps(fs)\ndef extra() { 0 }']) {
    await page.locator('#baseline-editor').fill('@caps(fs)\ndef main() { 0 }');
    await page.locator('#source-editor').fill(`${order}\n@caps(fs)\ndef main() { 0 }`);
    await page.locator('#diff-caps').click();
    equal(await page.evaluate(() => window.__garnetPlayground.state.lastDiff.authority_expanded), false, 'repeated new name leaves the aggregate unchanged');
    equal(await page.locator('#lane-card').getAttribute('data-state'), 'review', `repeated new name with a declared capability requires review (${order.startsWith('@caps(fs)') ? 'fs first' : 'fs last'})`);
    if (!(await page.locator('#lane-card').textContent()).includes('extra added with fs')) throw Error('repeated new name not named with its capabilities');
  }
  await page.locator('#baseline-editor').fill('@caps()\ndef main() { 0 }');
  await page.locator('#source-editor').fill('@caps()\ndef main() { time::now_ms() }');
  await page.locator('#diff-caps').click();
  equal(await page.locator('#diff-verdict').textContent(), 'No authority expansion', 'check-failure diff label text kept');
  equal(await page.locator('#diff-verdict').getAttribute('data-state'), 'neutral', 'check failure leaves the diff label neutral, not green');
  const presets = {};
  for (const name of ['capability_cycle', 'illegal_enum', 'undeclared_clock', 'wording_vs_write']) {
    await page.locator('#example-picker').selectOption(name);
    await page.locator('#check-source').click();
    presets[name] = await page.evaluate(() => window.__garnetPlayground.state.lastCheck);
    equal(presets[name].ok, name === 'wording_vs_write', `${name} real checker verdict`);
  }
  await page.locator('#diff-caps').click();
  equal(await page.locator('#lane-card').getAttribute('data-state'), 'unknown', 'wording unchanged');
  await page.locator('#preset-alternate').click();
  await page.locator('#diff-caps').click();
  equal(await page.locator('#lane-card').getAttribute('data-state'), 'review', 'write variant expands');
  // Keyboard: Tab must retain text and insert two spaces, check/diff shortcuts
  // must execute the same adapters as their buttons.
  await page.locator('#source-editor').fill(pure);
  await page.locator('#source-editor').press('ControlOrMeta+A');
  await page.locator('#source-editor').press('ArrowLeft');
  await page.locator('#source-editor').press('Tab');
  equal((await page.locator('#source-editor').inputValue()).startsWith('  '), true, 'Tab spaces');
  await page.locator('#source-editor').press('ControlOrMeta+Enter');
  equal(await page.locator('#check-state').textContent(), 'Check passed', 'keyboard check');
  await page.locator('#source-editor').press('ControlOrMeta+Shift+Enter');
  equal(await page.locator('#diff-verdict').textContent(), 'No authority expansion', 'keyboard diff');
  equal(await page.locator('#source-lines').textContent(), '1\n2\n3', 'line numbers');
  await page.locator('#source-editor').press('ControlOrMeta+A');
  const selectedText = await page.locator('#source-editor').inputValue();
  await page.locator('#source-editor').press('Tab');
  equal(await page.locator('#source-editor').inputValue(), selectedText, 'Tab preserves selected text');
  // Share round-trip includes Unicode and HTML-shaped data. Restoring must not
  // evaluate markup or execute Garnet, and malformed links must preserve defaults.
  const unicode = '@caps()\ndef main() { println("🌴 café <img src=x onerror=alert(1)>") }';
  await page.locator('#source-editor').fill(unicode);
  await page.locator('#baseline-editor').fill(pure);
  await page.locator('#share-source').click();
  const link = await page.locator('#share-link').inputValue();
  if (!link.includes('#garnet=')) throw Error('versioned fragment link missing');
  await page.locator('#source-editor').fill('changed');
  await page.evaluate(link => { location.hash = new URL(link).hash; }, link);
  await page.waitForFunction(expected => document.querySelector('#source-editor').value === expected, unicode);
  equal(await page.locator('#run-state').textContent(), 'Not run', 'hashchange invalidates old results');
  await page.goto(link, { waitUntil: 'networkidle' });
  await page.reload({ waitUntil: 'networkidle' });
  equal(await page.locator('#source-editor').inputValue(), unicode, 'Unicode share source');
  equal(await page.locator('#baseline-editor').inputValue(), pure, 'share baseline');
  equal(await page.locator('#run-state').textContent(), 'Not run', 'share never autoruns');
  equal(await page.locator('img[src="x"]').count(), 0, 'share markup inert');
  for (const fragment of ['garnet=%bad', 'garnet=' + 'x'.repeat(17000), 'garnet=' + btoa(JSON.stringify({v:2,source:'oops',baseline:''}))]) {
    await page.goto(`${baseUrl}/playground.html#${fragment}`, { waitUntil: 'networkidle' });
    await page.reload({ waitUntil: 'networkidle' });
    equal(await page.locator('#share-status').getAttribute('data-state'), 'denied', 'malformed share rejected');
    equal(await page.locator('#run-state').textContent(), 'Not run', 'malformed never autoruns');
  }
  await page.locator('#source-editor').fill('x'.repeat(17000));
  await page.locator('#share-source').click();
  equal(await page.locator('#share-status').getAttribute('data-state'), 'denied', 'oversize share capped');
  await page.goto(`${baseUrl}/playground.html`, { waitUntil: 'networkidle' });
  await page.waitForFunction(() => document.querySelector('#evidence-badge')?.dataset.state === 'ready');
  const evidence = await page.locator('#evidence-badge').textContent();
  if (!/[a-f0-9]{64}/.test(evidence) || !evidence.includes('scripts/build_playground_wasm.py')) throw Error('evidence digest or rebuild command absent');
  await page.setViewportSize({width:375,height:844});
  equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, '375px overflow');
  await page.goto(`${baseUrl}/index.html`, { waitUntil: 'domcontentloaded', timeout:15000 });
  await page.locator('#pg-load').click();
  const frame = page.frameLocator('#pg-mount iframe');
  await frame.locator('#runtime-status[data-state=ready]').waitFor();
  equal(await frame.locator('#baseline-panel').isVisible(), false, 'embedded baseline initially closed');
  await frame.locator('#toggle-baseline').click();
  equal(await frame.locator('#baseline-panel').isVisible(), true, 'embedded baseline toggle');
  await frame.locator('#toggle-baseline').click();
  equal(await frame.locator('#baseline-panel').isVisible(), false, 'embedded baseline closes');
  await page.setViewportSize({width:1440,height:1000});
  const iframeHeight = await page.locator('#pg-mount iframe').evaluate(el => el.getBoundingClientRect().height);
  equal(iframeHeight < 1180, true, 'compact embed height');
  await frame.locator('a.brand-home').click();
  await page.waitForURL(`${baseUrl}/index.html`);
  equal(await page.locator('#pg-mount iframe').count(), 0, 'home exits iframe');
  await page.goto(`${baseUrl}/playground.html`, {waitUntil:'networkidle'});
  await page.locator('#source-editor').fill('@caps(time)\ndef main() { time::now_ms() }');
  await page.locator('#run-source').click();
  equal(await page.locator('#run-state').textContent(), 'Adapter error', 'unsupported clock trap shown');
  if (!(await page.locator('#run-result').textContent()).includes('Browser adapter failed')) throw Error('adapter failure hidden');
  await page.reload({waitUntil:'networkidle'});
  return {review_fixes:true, results_readable:true, lanes_card:true, authority_panel:true, share_link:true, presets_v2:presets, editor_keys:true, evidence_badge:true, embed_mode:true, mobile_375:true};
}

export async function offlineJourney(browser, baseUrl, equal) {
  const context = await browser.newContext({serviceWorkers:'allow'});
  const external = [];
  context.on('request', request => { if (new URL(request.url()).origin !== baseUrl) external.push(request.url()); });
  await context.route('**/*', route => new URL(route.request().url()).origin === baseUrl ? route.continue() : route.abort());
  try {
    const page = await context.newPage();
    // First visit only the landing page. The playground and its module/WASM
    // dependencies must arrive through worker install, not a warmed page cache.
    await page.goto(`${baseUrl}/manifest.webmanifest`, {waitUntil:'domcontentloaded'});
    await page.evaluate(async () => { const old = await caches.open('garnet-web-v6'); await old.put('./stale-playground-probe', new Response('old')); });
    await page.goto(`${baseUrl}/index.html`, {waitUntil:'domcontentloaded'});
    await page.evaluate(async () => { await navigator.serviceWorker.ready; });
    await page.waitForFunction(async () => !!navigator.serviceWorker.controller && (await caches.match('./playground/pkg/garnet_wasm_bg.wasm')) !== undefined);
    const cached = await page.evaluate(async () => { const names = await caches.keys(); return Promise.all(names.map(async name => ({name, paths:(await (await caches.open(name)).keys()).map(x => new URL(x.url).pathname)}))); });
    equal(cached.some(cache => cache.name === 'garnet-web-v6'), false, 'old cache retired');
    await context.setOffline(true);
    await page.goto(`${baseUrl}/playground.html`, {waitUntil:'networkidle'});
    await page.locator('#runtime-status[data-state=ready]').waitFor();
    await page.locator('#run-source').click();
    equal(await page.locator('#run-state').textContent(), 'Completed', 'cold install offline run');
    await page.locator('#check-source').click();
    equal(await page.locator('#check-state').textContent(), 'Check passed', 'cold install offline check');
    await page.locator('#diff-caps').click();
    equal(await page.locator('#diff-verdict').textContent(), 'No authority expansion', 'cold install offline diff');
    await page.goto(`${baseUrl}/index.html`, {waitUntil:'domcontentloaded'});
    await page.locator('#pg-load').click();
    await page.frameLocator('#pg-mount iframe').locator('#runtime-status[data-state=ready]').waitFor();
    equal(external.length, 0, 'offline no external requests');
    return {offline_embed:true, cold_install:true, offline_run:true, offline_check:true, offline_diff:true, old_cache_retired:true, cached, external_requests:external};
  } finally { await context.close(); }
}

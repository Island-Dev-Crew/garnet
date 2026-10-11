'use strict';
// A live source check and declared-surface diff. Review choices belong to this
// browser session; the separately recorded CLI seal does not attest them.
window.GarnetAcceptance = (() => {
  let generation = 0;
  const source = declared => '@caps(' + (declared ? 'fs' : '') + ')\ndef main() {\n  fs::write_file("report.txt", "Ready for review")\n  0\n}';
  const icon = '<svg viewBox="0 0 72 84" width="72" height="84" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true"><path d="M15 4h28l15 15v60H15zM43 4v16h15M24 39h25M24 48h25M24 57h17"/></svg>';

  function mount({panel, engine, highlight, openRecord, sourceLink}) {
    const token = ++generation;
    let revision = 0, busy = false, checked = false;
    panel.innerHTML = `<div class="lab-top"><h3>One file. One decision.</h3><span class="tag"><span class="dot"></span>Live Garnet checker</span></div>
      <ol class="journey-steps" aria-label="Guided acceptance journey"><li aria-current="step">01 <span>Write</span></li><li>02 <span>Check</span></li><li>03 <span>Review</span></li><li>04 <span>Verify</span></li></ol>
      <div class="lab-body"><div class="editor"><div class="filebar"><span><span class="file-dot">◇</span> report.garnet</span><span>GARNET</span></div><pre class="source" id="demo-code"></pre><div class="editor-tools"><label class="toggle-label"><input type="checkbox" id="allow-fs">Declare filesystem access</label><button class="btn small primary" id="run-demo">Run check <span aria-hidden="true">→</span></button></div></div>
      <div class="live-output">${window.GarnetScenes.check()}<h4 class="result-title" id="result-title" aria-live="polite" aria-atomic="true">A file write. An explicit boundary.</h4><p class="result-copy" id="result-copy">Check the call. Declare its authority.</p><details class="raw-output"><summary>Inspect result JSON</summary><pre id="raw-result">Run a check to inspect its result.</pre></details></div></div>
      <div class="review-band"><div><span class="review-label">Declared authority</span><strong id="authority-delta">fs · not declared</strong></div><div><span class="review-label">Your decision</span><div class="review-actions"><button class="btn small" id="accept-change" disabled>Accept change</button><button class="btn small" id="reject-change" disabled>Reject change</button></div></div><p id="review-decision" role="status">Check the source to begin.</p></div>
      <div class="journey-links"><button class="text-action" id="inspect-cli">Inspect the CLI record <span aria-hidden="true">→</span></button><a id="edit-playground" href="playground.html">Edit in playground ↗</a><button class="text-action" id="reset-journey">Start again</button></div>
      <details class="lab-foot scope-details"><summary>Scope · Annotated filesystem calls</summary><p>This live checker inspects the annotated named call; it does not execute a file write. The diff compares declarations. Closure, function-value and dynamic-dispatch edges are outside its propagation; unannotated function bodies are not checked. Accept/Reject is a local review choice, not a signed approval. The CLI record separately demonstrates unsigned seal/v2 source binding on current source.</p><a href="${sourceLink}" target="_blank" rel="noopener">Read the scope ↗</a></details>`;
    const $ = selector => panel.querySelector(selector);
    const steps = [...panel.querySelectorAll('.journey-steps li')];
    const stage = index => steps.forEach((item, i) => {
      item.classList.toggle('complete', i < index);
      if (i === index) item.setAttribute('aria-current', 'step'); else item.removeAttribute('aria-current');
    });
    const decisions = enabled => { $('#accept-change').disabled = !enabled; $('#reject-change').disabled = !enabled; };
    const reset = () => {
      revision++; checked = false; decisions(false); stage(0);
      $('#demo-code').innerHTML = highlight(source($('#allow-fs').checked));
      $('.gate-budget').textContent = '@caps(' + ($('#allow-fs').checked ? 'fs' : '') + ')';
      window.GarnetScenes.phase('idle', 'fs::write_file → report.txt · source inspection');
      $('#authority-delta').textContent = $('#allow-fs').checked ? 'fs · declared, awaiting check' : 'fs · not declared';
      $('#result-title').textContent = 'A file write. An explicit boundary.';
      $('#result-copy').textContent = 'Check the call. Declare its authority.';
      $('#raw-result').textContent = 'Run a check to inspect its result.';
      $('#review-decision').textContent = 'Check the source to begin.';
      delete $('#review-decision').dataset.decision;
      const shared = btoa(JSON.stringify({v:1, source:source($('#allow-fs').checked), baseline:source(false)})).replace(/\+/g,'-').replace(/\//g,'_').replace(/=+$/,'');
      $('#edit-playground').href = 'playground.html#garnet=' + shared;
    };
    $('#allow-fs').onchange = reset;
    $('#reset-journey').onclick = () => { $('#allow-fs').checked = false; reset(); };
    $('#inspect-cli').onclick = openRecord;
    $('#run-demo').onclick = async () => {
      if (busy) return;
      busy = true; const version = revision, current = source($('#allow-fs').checked);
      const button = $('#run-demo'); button.disabled = true; button.textContent = 'Checking…';
      window.GarnetScenes.phase('checking', 'Inspecting the declared filesystem call…');
      try {
        const api = await engine();
        if (token !== generation || version !== revision || !panel.isConnected || !button.isConnected) return;
        const check = JSON.parse(api.check_source(current));
        const diff = JSON.parse(api.diff_caps_source(source(false), current));
        checked = check.ok === true;
        window.GarnetScenes.phase(checked ? 'allowed' : 'blocked', checked ? 'Matching fs declaration. This source check passes.' : 'The request names fs. The declaration does not.');
        $('#result-title').textContent = checked ? 'Declared access. This check passes.' : 'Undeclared access. Check rejected.';
        $('#result-copy').textContent = checked ? 'Zero diagnostics. Review the + fs declaration.' : check.diagnostics.map(item => item.message).join('\n');
        $('#authority-delta').textContent = diff.authority_expanded ? '+ fs · review required' : 'fs · not declared';
        $('#raw-result').textContent = JSON.stringify({check, diff}, null, 2);
        $('#review-decision').textContent = checked ? 'A passing check brings the choice to you.' : 'Add @caps(fs), then check again.';
        decisions(checked); stage(checked ? 2 : 1);
      } catch (error) {
        if (token !== generation || version !== revision || !button.isConnected) return;
        checked = false; decisions(false);
        window.GarnetScenes.phase('error', 'Checker unavailable. No verdict was produced.');
        $('#result-title').textContent = 'The checker could not initialize.';
        $('#result-copy').textContent = 'Try again or open the Garnet playground.';
        $('#raw-result').textContent = String(error);
      } finally {
        busy = false;
        if (button.isConnected) { button.disabled = false; button.innerHTML = 'Run check <span aria-hidden="true">→</span>'; }
      }
    };
    for (const [id, accepted] of [['accept-change', true], ['reject-change', false]]) {
      $('#' + id).onclick = () => {
        if (!checked) return;
        $('#review-decision').textContent = accepted ? 'Accepted for this review. Inspect the bound CLI record.' : 'Change declined. Authority stays with the reviewer.';
        stage(accepted ? 3 : 2);
        window.GarnetScenes.phase(accepted ? 'accepted' : 'declined', accepted ? 'Accepted in this local review. Inspect the bound CLI record.' : 'Declined in this local review.');
        $('#review-decision').dataset.decision = accepted ? 'accepted' : 'rejected';
      };
    }
    reset();
  }

  async function record({panel, escape}) {
    const token = ++generation;
    panel.innerHTML = '<div class="lab-top"><h3>Bind the source. Recompute the record.</h3><span class="tag illustration"><span class="dot"></span>Recorded CLI · current source</span></div><p class="record-loading" role="status">Opening the recorded CLI journey…</p>';
    try {
      const response = await fetch('demos/acceptance/record.json');
      if (!response.ok) throw Error('Record unavailable');
      const data = await response.json();
      if (token !== generation) return;
      if (data.schema !== 'garnet.website.cli-record/1' || data.records?.length !== 7) throw Error('Invalid record');
      const commands = data.records;
      panel.innerHTML = `<div class="lab-top"><h3>Bind the source. Recompute the record.</h3><span class="tag illustration"><span class="dot"></span>Recorded CLI · current source</span></div><div class="lab-body record-body"><div class="editor"><div class="filebar"><span>seal/v2 · recorded output</span><span>UNSIGNED</span></div><div class="record-terminal"><p class="terminal-label">Source unchanged</p><pre>${escape(commands[5].command + '\n' + commands[5].stdout)}</pre><p class="terminal-label mismatch-label">Source changed after sealing</p><pre>${escape(commands[6].command + '\n' + commands[6].stderr)}</pre></div><div class="editor-tools"><a href="demos/acceptance/replay.py" download>Download replay script ↓</a><a href="demos/acceptance/record.json" download>Full JSON ↓</a></div></div><div class="live-output">${window.GarnetScenes.record()}<h4 class="result-title" id="result-title">Same source. Same seal.</h4><p class="result-copy">Seven recorded commands. A changed source fails verification.</p><button class="btn small" id="illustrate">Replay the evidence path <span aria-hidden="true">→</span></button></div></div><details class="record-transcript"><summary>Inspect all seven commands</summary>${commands.map(item => '<pre>' + escape('$ ' + item.command + '\n' + item.stdout + item.stderr + '\nexit ' + item.exit_code) + '</pre>').join('')}</details><details class="lab-foot scope-details"><summary>Scope · Source and manifest</summary><p>Captured from current-source CLI at <code>${escape(data.source_revision.slice(0,12))}</code>. This is seal/v2 content binding, not signature verification, identity proof, program correctness or a cryptographic human acceptance record. Garnet emits this seal unsigned; external signing is separate. The published v0.8.2 download uses seal/v1.</p><p><a href="https://github.com/Island-Dev-Crew/garnet/blob/aba11b329e3284845ba7c9cd4c870998503a9030/garnet-cli/src/cmd/verify.rs" target="_blank" rel="noopener">Read the verification source ↗</a></p><p><a href="demos/acceptance/before.garnet" download>Before source ↓</a> · <a href="demos/acceptance/accepted.garnet" download>Declared source ↓</a> · <a href="demos/acceptance/changed.garnet" download>Changed source ↓</a> · <a href="demos/acceptance/README.md">Reproduce the record →</a></p></details>`;
      panel.querySelector('#illustrate').onclick = () => {
        window.GarnetScenes.replay('record');
      };
    } catch {
      if (token !== generation) return;
      panel.innerHTML = '<div class="lab-top"><h3>The CLI record could not load.</h3></div><p>Try the record again.</p><button class="btn small" id="retry-record">Retry record</button>';
      panel.querySelector('#retry-record').onclick = () => record({panel, escape});
    }
  }
  return {mount, record, invalidate: () => { generation++; }};
})();

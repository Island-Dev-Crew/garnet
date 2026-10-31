'use strict';
// Illustrative traces are driven by real checker results where available.
// No scene runs a host operation, signs a decision, or creates an acceptance proof.
window.GarnetScenes = (() => {
  let serial=0;
  const frame=(kind,content,label)=>`<figure class="evidence-scene scene-${kind}" data-phase="idle" aria-label="${label}"><svg viewBox="0 0 720 360" role="img" aria-labelledby="scene-title-${++serial}"><title id="scene-title-${serial}">${label}</title><defs><linearGradient id="scene-gold-${serial}"><stop stop-color="#75603c"/><stop offset=".5" stop-color="#edce89"/><stop offset="1" stop-color="#75603c"/></linearGradient></defs><path class="scene-grid" d="M0 60H720M0 120H720M0 180H720M0 240H720M0 300H720M80 0V360M160 0V360M240 0V360M320 0V360M400 0V360M480 0V360M560 0V360M640 0V360"/>${content}</svg><figcaption class="scene-caption"></figcaption></figure>`;
  const file=(x,y,cls='')=>`<g class="scene-file ${cls}" transform="translate(${x} ${y})"><path d="M0 0H56L80 24V110H0Z"/><path d="M56 0V24H80M17 47H63M17 63H63M17 79H47"/></g>`;
  function check(){return frame('check',`
    <text class="scene-kicker" x="32" y="34">A REQUEST IS NOT PERMISSION</text>
    <path class="scene-track" d="M92 187H630"/>
    <g class="request-source"><rect class="scene-panel" x="34" y="94" width="167" height="188" rx="10"/>${file(77,117)}<text class="scene-label" x="117" y="257" text-anchor="middle">AGENT CODE</text></g>
    <g class="authority-gate"><path class="gate-body" d="M360 62 440 111V263L360 310 280 263V111Z"/><path class="gate-line" d="M360 62V310M280 111 440 263M440 111 280 263"/><rect class="lock-body" x="337" y="163" width="46" height="42" rx="5"/><path class="lock-shackle" d="M346 163V149A14 14 0 0 1 374 149V163"/><circle class="lock-key" cx="360" cy="182" r="4"/><text class="scene-label gate-budget" x="360" y="241" text-anchor="middle">@caps()</text></g>
    <g class="protected-target"><rect class="scene-panel" x="520" y="94" width="167" height="188" rx="10"/>${file(564,117)}<text class="scene-label" x="604" y="257" text-anchor="middle">REPORT.TXT</text></g>
    <g class="request-packet"><rect x="109" y="169" width="52" height="36" rx="7"/><text x="135" y="192" text-anchor="middle">fs</text></g>
    <g class="check-impact"><circle cx="302" cy="187" r="28"/><circle cx="302" cy="187" r="44"/><path d="m291 176 22 22m0-22-22 22"/></g>
    <g class="pass-ticket"><circle cx="608" cy="187" r="25"/><path d="m596 187 8 8 16-18"/></g>
    <text class="scene-subtext" x="117" y="324" text-anchor="middle">REQUESTED</text><text class="scene-subtext" x="360" y="342" text-anchor="middle">DECLARED AUTHORITY</text><text class="scene-subtext" x="604" y="324" text-anchor="middle">TARGET</text>
  `,'A filesystem request meets the declared-authority boundary. The checker result determines rejection or declaration coverage.');}
  function diff(){return frame('diff',`
    <text class="scene-kicker" x="32" y="34">THE CHANGE BEHIND THE CHANGE</text>
    <rect class="scene-panel" x="38" y="88" width="238" height="210" rx="12"/><text class="scene-label" x="64" y="120">BEFORE</text><circle class="permission-dot" cx="157" cy="191" r="43"/><text class="permission-text" x="157" y="200" text-anchor="middle">fs</text><text class="scene-subtext" x="157" y="267" text-anchor="middle">FILE AUTHORITY</text>
    <path class="scene-track" d="M293 192H412"/><path class="scene-arrow" d="m395 184 17 8-17 8"/>
    <rect class="scene-panel" x="442" y="88" width="240" height="210" rx="12"/><text class="scene-label" x="468" y="120">AFTER</text><circle class="permission-dot" cx="520" cy="191" r="32"/><text class="permission-text" x="520" y="200" text-anchor="middle">fs</text><g class="permission-added"><circle cx="611" cy="191" r="39"/><text class="permission-text" x="611" y="200" text-anchor="middle">net</text></g><text class="scene-subtext" x="562" y="267" text-anchor="middle">DECLARED SURFACE</text>
    <g class="review-ring"><path d="M326 270H394V322H326Z"/><path d="m343 283 17 22 18-22"/></g><text class="scene-label review-tag" x="360" y="347" text-anchor="middle">HUMAN REVIEW</text>
  `,'The real capability diff reveals an added network declaration for human review.');}
  function record(){return frame('record',`
    <text class="scene-kicker" x="32" y="34">SAME SOURCE. SAME RECORD.</text>
    <path class="record-thread scene-track" d="M128 194H592"/>
    <g class="record-node record-source"><rect class="scene-panel" x="48" y="102" width="166" height="185" rx="10"/>${file(91,124)}<text class="scene-label" x="131" y="265" text-anchor="middle">SOURCE</text></g>
    <g class="record-node record-seal"><path class="seal-body" d="M360 88 441 135V249L360 296 279 249V135Z"/><path d="m330 187 23 23 38-41" class="record-tick"/><text class="scene-label" x="360" y="263" text-anchor="middle">SEAL / V2</text></g>
    <g class="record-node record-verify"><rect class="scene-panel" x="506" y="102" width="166" height="185" rx="10"/><path class="record-tick" d="m553 185 23 23 41-44"/><text class="scene-label" x="589" y="265" text-anchor="middle">VERIFY</text></g>
    <g class="record-break"><path d="m476 169-32 50m-5-50 32 50"/><circle cx="455" cy="194" r="35"/></g><text class="scene-subtext record-hash" x="360" y="336" text-anchor="middle">SOURCE HASH → CONTENT BINDING</text>
  `,'Illustrated replay of recorded CLI evidence: unchanged source verifies; altered source fails.');}
  function depth(){return frame('depth',`
    <text class="scene-kicker" x="32" y="34">A STOPPING POINT, WRITTEN DOWN</text>
    <path class="depth-rail" d="M90 82V304M560 82V304"/>
    <g class="stack-call call-one"><rect x="155" y="264" width="340" height="45" rx="6"/><text x="182" y="293">01</text><text x="305" y="293">CALL ENTERS</text></g>
    <g class="stack-call call-two"><rect x="155" y="208" width="340" height="45" rx="6"/><text x="182" y="237">02</text><text x="305" y="237">CALL ENTERS</text></g>
    <g class="stack-call call-three"><rect x="155" y="152" width="340" height="45" rx="6"/><text x="182" y="181">03</text><text x="305" y="181">CEILING REACHED</text></g>
    <path class="depth-ceiling" d="M110 135H544"/><text class="scene-label" x="572" y="141">MAX 3</text>
    <g class="stack-call call-four"><rect x="155" y="83" width="340" height="41" rx="6"/><text x="182" y="109">04</text><text x="305" y="109">REFUSED</text></g>
    <text class="scene-subtext" x="325" y="341" text-anchor="middle">ILLUSTRATED RECURSION LIMIT</text>
  `,'Illustration: three calls fit the stated ceiling; the fourth meets the limit.');}
  function memory(){return frame('memory',`
    <text class="scene-kicker" x="32" y="34">EVERY KIND HAS A PLACE</text>
    <g class="memory-card"><rect x="209" y="71" width="302" height="73" rx="7"/><text class="memory-example" x="360" y="101" text-anchor="middle">Fix a failed check</text><text class="scene-subtext" x="360" y="126" text-anchor="middle">CURRENT TASK</text></g>
    <path class="memory-route" d="M360 145V178H108V216"/><g class="memory-token"><circle cx="360" cy="160" r="7"/></g><path class="memory-branch scene-track" d="M360 145V178M104 178H614M104 178V216M275 178V216M445 178V216M614 178V216"/>
    ${['WORKING','EPISODIC','SEMANTIC','PROCEDURAL'].map((s,i)=>`<g class="memory-bin ${i===0?'selected':''}" data-kind="${i}"><rect x="${37+i*170}" y="218" width="143" height="86" rx="8"/><text class="scene-label" x="${108+i*170}" y="253" text-anchor="middle">${s}</text><text class="scene-subtext" x="${108+i*170}" y="279" text-anchor="middle">${['TASK','EVENT','FACT','PROCEDURE'][i]}</text></g>`).join('')}
    <text class="scene-subtext" x="360" y="342" text-anchor="middle">ILLUSTRATED MEMORY ROLES</text>
  `,'Illustration: task, event, fact and procedure belong to different memory roles.');}
  function actor(){return frame('actor',`
    <text class="scene-kicker" x="32" y="34">CHANGE BEHAVIOR. KEEP THE ADDRESS.</text>
    <path class="scene-track" d="M78 201H633"/>
    <rect class="scene-panel queue-shell" x="36" y="116" width="247" height="161" rx="10"/><text class="scene-label" x="59" y="146">MAILBOX</text>
    ${['01','02','03'].map((s,i)=>`<g class="mail-message message-${i}" style="--mail-distance:${554-i*71}px"><rect x="${58+i*71}" y="175" width="57" height="51" rx="6"/><text x="${86+i*71}" y="207" text-anchor="middle">${s}</text></g>`).join('')}
    <g class="actor-station"><path class="gate-body" d="M462 89 553 141V254L462 305 371 254V141Z"/><text class="scene-label actor-version" x="462" y="196" text-anchor="middle">BEHAVIOR V1</text><text class="scene-subtext" x="462" y="232" text-anchor="middle">SAME ADDRESS</text></g>
    <g class="delivered-box"><rect class="scene-panel" x="598" y="146" width="85" height="111" rx="8"/><text class="scene-label" x="640" y="184" text-anchor="middle">ORDER</text><text class="delivery-count" x="640" y="224" text-anchor="middle">0</text></g>
    <text class="scene-subtext" x="360" y="342" text-anchor="middle">ILLUSTRATED BUFFER → SWAP → REPLAY</text>
  `,'Illustration: mailbox order survives a behavior change at the same actor address.');}
  const find=()=>document.querySelector('#scenario-panel .evidence-scene');
  function phase(value,caption){const el=find();if(!el)return;el.dataset.phase=value;el.querySelector('.scene-caption').textContent=caption||'';}
  function replay(kind,option=0){const el=find();if(!el)return;el.classList.remove('replaying');void el.getBoundingClientRect();el.classList.add('replaying');
    if(kind==='memory'){el.dataset.kind=option;el.style.setProperty('--memory-destination',`${108+option*170-360}px`);el.querySelector('.memory-route').setAttribute('d',`M360 145V178H${108+option*170}V216`);el.querySelectorAll('.memory-bin').forEach((g,i)=>g.classList.toggle('selected',i===option));el.querySelector('.memory-example').textContent=['Fix a failed check','A check was rejected','Filesystem access requires fs','Check before accepting'][option];el.querySelector('.memory-card .scene-subtext').textContent=['CURRENT TASK','PAST EVENT','REUSABLE FACT','REUSABLE PROCEDURE'][option];}
    if(kind==='actor'){el.querySelector('.actor-version').textContent='BEHAVIOR V2';el.querySelector('.delivery-count').textContent='3';}
    phase('replay',{record:'Unchanged source verifies. Altered source is refused.',depth:'Three calls enter. The fourth meets the ceiling.',memory:['Task → working','Event → episodic','Fact → semantic','Procedure → procedural'][option],actor:'Buffer 01, 02, 03. Swap behavior. Replay in order.'}[kind]);
  }
  const compact=matchMedia('(max-width:900px)');
  function layout(){const panel=document.querySelector('#scenario-panel'),editor=panel?.querySelector('.editor'),output=panel?.querySelector('.live-output'),tools=panel?.querySelector('.editor-tools');if(!editor||!output||!tools?.querySelector('button'))return;if(compact.matches){if(tools.parentElement!==output)output.prepend(tools);}else if(tools.parentElement!==editor)editor.append(tools);}
  compact.addEventListener('change',layout);
  return {check,diff,record,depth,memory,actor,phase,replay,layout};
})();

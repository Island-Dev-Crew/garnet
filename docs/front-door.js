
'use strict';
const $=s=>document.querySelector(s), $$=s=>[...document.querySelectorAll(s)];
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
let motionPaused = false;
const state={scenario:'check',os:null,mode:'release',engine:null,busy:false,memory:0,motion:reducedMotion.matches};
const escapeHtml=s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const liveSource='https://github.com/Island-Dev-Crew/garnet/blob/aba11b329e3284845ba7c9cd4c870998503a9030/';
// One polite announcer speaks for the front door, so verdicts, decisions and cues never
// compete. It is always rendered; clearing it first lets a repeated message speak again.
let announceTimer,announceClear;function announce(message){const el=$('#site-announcer');if(!el||!message)return;clearTimeout(announceTimer);clearTimeout(announceClear);el.textContent='';announceTimer=setTimeout(()=>{el.textContent=message;announceClear=setTimeout(()=>{el.textContent=''},7000)},60)}
window.GarnetAnnounce=announce;
const newTab=label=>label+' <span aria-hidden="true">↗</span><span class="sr-only"> (opens in a new tab)</span>';
let toastTimer;function toast(s){$('#toast').textContent=s;$('#toast').hidden=false;announce(s);clearTimeout(toastTimer);toastTimer=setTimeout(()=>$('#toast').hidden=true,3300)}
async function copyText(text){const previous=document.activeElement;let area;try{if(navigator.clipboard&&window.isSecureContext){await navigator.clipboard.writeText(text)}else{area=document.createElement('textarea');area.value=text;area.style.position='fixed';area.style.opacity='0';document.body.append(area);area.select();if(!document.execCommand('copy'))throw Error('copy unavailable');area.remove()}toast('Command copied.')}catch{toast('Copy is unavailable here. Select and copy the command above.')}finally{area?.remove();previous?.focus({preventScroll:true})}}
let requestedHash='#home',activeHash=null,routing=false,lastExperienceHash='#home',routeEpoch=0;
let referenceLoaded=false,referencePromise=null,focusedLoadedPage=null,focusedLoadEpoch=0;
const routeScroll=new Map(),frame=$('#original-frame'),brand=$('#garnet-brand');
// A link to status.html#release-details opens the Status page at that section. The fragment
// belongs to that one navigation: go() sets or clears it, a route change drops it, and it is
// used once, when the focused frame holds the page or when routing settles on it.
let pendingAnchor=null;
function scrollFocused(hash,win){let top=routeScroll.get(hash)||0;if(pendingAnchor?.hash===hash){const id=pendingAnchor.id;pendingAnchor=null;try{const target=win.document.getElementById(id);if(target)top=target.getBoundingClientRect().top+(win.scrollY||0)-24}catch{}}win.scrollTo({top,behavior:'instant'})}
function applyPendingAnchor(){if(!pendingAnchor||pendingAnchor.hash!==activeHash)return;const page=focusedPages[activeHash]?.[1];try{if(focusedLoadedPage===page&&focusedFrame.contentDocument?.documentElement.dataset.garnetPage===page)scrollFocused(activeHash,focusedFrame.contentWindow)}catch{}}
let focusedFrame=$('#focused-frame'),cancelFocusedLoad=null,cachedPlaygroundFrame=null;
const focusedPages={'#page-why':['Why','why.html'],'#page-docs':['Documentation','minispec.html'],'#page-status':['Status','status.html'],'#page-models':['Models inside Garnet','models.html'],'#page-playground':['Playground','playground.html']};
const referenceAliases={'#caps':'#caps-chapter','#why':'#why-chapter','#readiness':'#readiness-chapter','#docs':'#docs-catalogue','#spec':'#docs-catalogue','#vision':'#thesis-agent','#download':'#install','#roadmap':'#readiness'};
const referenceSections=new Set(['#why','#code','#compare','#caps','#studio','#spec','#vision','#community','#download','#roadmap','#proof','#try','#demonstration','#docs','#readiness']);
const isFocused=h=>Object.hasOwn(focusedPages,h),isReference=h=>h==='#reference'||referenceSections.has(h);
const viewOf=h=>isReference(h)?'reference':isFocused(h)?'focused':h==='#install'?'install':'home';
const childPosition=f=>{try{return f.contentWindow?.scrollY||0}catch{return 0}};
let armedLink=null,armCreatedEntry=false,cueTimer=null,lastActivationHow='click';
if('scrollRestoration' in history)history.scrollRestoration='manual';
try{const saved=JSON.parse(sessionStorage.getItem('garnet-front-door-position')||'{}');for(const [k,v]of Object.entries(saved.scroll||{}))if(Number.isFinite(v))routeScroll.set(k,v);if(['#home','#install','#use-cases'].includes(saved.experience))lastExperienceHash=saved.experience}catch{}
function savePosition(){if(activeHash)routeScroll.set(activeHash,isReference(activeHash)?childPosition(frame):isFocused(activeHash)?childPosition(focusedFrame):scrollY);if(activeHash&&isReference(activeHash))routeScroll.set('#reference',childPosition(frame));try{sessionStorage.setItem('garnet-front-door-position',JSON.stringify({experience:lastExperienceHash,scroll:Object.fromEntries(routeScroll)}))}catch{}}
window.addEventListener('pagehide',savePosition);
function disarm(){armedLink?.classList.remove('is-armed');armedLink?.removeAttribute('aria-description');armedLink=null;armCreatedEntry=false;clearTimeout(cueTimer);$('#navigation-cue').hidden=true;const said=$('#site-announcer');if(said&&/again for the full/.test(said.textContent))said.textContent='';}
function arm(link,created,how='click'){disarm();armedLink=link;armCreatedEntry=created;link.classList.add('is-armed');const name=link===brand?'Garnet':link.textContent.trim(),destination=link===brand?'the full homepage':'the full page';link.setAttribute('aria-description','Activate again to open '+destination+'.');const cue=$('#navigation-cue'),words=$('#navigation-cue-text');words.replaceChildren();const inMenu=link!==brand&&!!link.closest('.navlinks')&&$('.menu-toggle').getClientRects().length>0;const label=document.createElement('b');label.textContent=name;if(inMenu)words.append('Open the menu, then '+(how==='key'?'press Enter on ':how==='touch'?'tap ':'click '),label,' again for ');else{words.append(how==='key'?'Press Enter again for ':how==='touch'?'Tap ':'Click ');if(how!=='key')words.append(label,' again for ');}words.append(destination+'.');cue.hidden=false;announce(words.textContent);const box=link.getBoundingClientRect(),header=document.querySelector('header').getBoundingClientRect();cue.style.left=Math.max(16,Math.min(box.left,innerWidth-cue.offsetWidth-16))-header.left+'px';cueTimer=setTimeout(()=>{cue.hidden=true},6500);}
function go(hash,replace=false,anchor=null){pendingAnchor=anchor&&isFocused(hash)?{hash,id:anchor}:null;if(location.hash!==hash){const url=new URL(location.href);url.hash=hash;history[replace?'replaceState':'pushState'](null,'',url);}else if(!routing)applyRoute(hash);void route();}
async function fadeMode(from,to,duration){const veil=$('#mode-veil');if(state.motion||reducedMotion.matches||!veil.animate){veil.style.opacity=String(to);return;}const animation=veil.animate([{opacity:from},{opacity:to}],{duration,easing:'ease-in-out',fill:'forwards'});try{await animation.finished}catch{}veil.style.opacity=String(to);animation.cancel();}
function referenceDocument(){try{return frame.contentDocument}catch{return null}}
function restoreReference(hash){void frame.getBoundingClientRect();let top=routeScroll.get(hash);if(top===undefined&&referenceSections.has(hash)){const target=referenceDocument()?.getElementById((referenceAliases[hash]||hash).slice(1));if(target){const reveal=target.closest('.reveal')||target.querySelector('.reveal');if(reveal){reveal.style.transition='none';reveal.classList.add('in');}top=target.getBoundingClientRect().top+childPosition(frame)-(hash==='#docs'||hash==='#spec'?16:24);}}if(top===undefined)top=routeScroll.get('#reference')||0;frame.contentWindow.scrollTo({top,behavior:'instant'});}
function ensureReference(){if(referenceLoaded)return Promise.resolve(true);if(referencePromise)return referencePromise;$('#reference-loader').hidden=false;$('#reference-fallback').hidden=true;const pending=new Promise(resolve=>{let timer,settled=false;const finish=okay=>{if(settled)return;settled=true;clearTimeout(timer);frame.removeEventListener('load',loaded);frame.removeEventListener('error',failed);referenceLoaded=okay;if(okay)referenceDocument()?.body.classList.toggle('motion-off',state.motion);$('#reference-loader').hidden=true;$('#reference-fallback').hidden=okay;if(okay&&isReference(activeHash))restoreReference(activeHash);resolve(okay)};const loaded=()=>finish(referenceDocument()?.documentElement.dataset.garnetReference==='true'),failed=()=>finish(false);frame.addEventListener('load',loaded);frame.addEventListener('error',failed);timer=setTimeout(failed,8000);frame.contentWindow.location.replace(new URL(frame.dataset.src,location.href).href)});referencePromise=pending;pending.then(okay=>{if(!okay)referencePromise=null});return pending;}
$('#retry-reference').onclick=async()=>{const button=$('#retry-reference');button.disabled=true;try{await ensureReference()}finally{button.disabled=false}};
function ensureFocused(hash){
 const page=focusedPages[hash]?.[1];if(!page)return;
 try{if(focusedLoadedPage===page&&focusedFrame.contentDocument?.documentElement.dataset.garnetPage===page){scrollFocused(hash,focusedFrame.contentWindow);return}}catch{}
 cancelFocusedLoad?.();
 const epoch=++focusedLoadEpoch,previous=focusedFrame;
 const restorePlayground=page==='playground.html'&&cachedPlaygroundFrame;
 const keepPlayground=focusedLoadedPage==='playground.html'&&!restorePlayground;
 const current=restorePlayground||previous.cloneNode(false);
 current.id='focused-frame';current.hidden=false;current.title='Garnet '+focusedPages[hash][0];
 focusedFrame=current;focusedLoadedPage=null;$('#focused-loader').hidden=false;$('#focused-fallback').hidden=true;
 // Keep an opened workbench mounted so navigation cannot discard source edits.
 if(keepPlayground){cachedPlaygroundFrame=previous;previous.id='cached-playground-frame';previous.hidden=true;previous.after(current);}
 else if(restorePlayground){previous.remove();focusedLoadedPage=page;cancelFocusedLoad=null;$('#focused-loader').hidden=true;scrollFocused(hash,current.contentWindow);return;}
 else previous.replaceWith(current);
 let timer,settled=false;
 const cleanup=()=>{clearTimeout(timer);current.removeEventListener('load',loaded);current.removeEventListener('error',failed);};
 const finish=okay=>{if(settled)return;settled=true;cleanup();if(epoch!==focusedLoadEpoch)return;cancelFocusedLoad=null;$('#focused-loader').hidden=true;$('#focused-fallback').hidden=okay;if(okay){focusedLoadedPage=page;if(activeHash===hash)scrollFocused(hash,current.contentWindow)}};
 const loaded=()=>{let okay=false;try{okay=current.contentDocument?.documentElement.dataset.garnetPage===page}catch{}finish(okay)},failed=()=>finish(false);
 cancelFocusedLoad=()=>{settled=true;cleanup();};
 current.addEventListener('load',loaded);current.addEventListener('error',failed);timer=setTimeout(failed,8000);
 // A fresh detail frame keeps document replacement out of the parent reading history.
 current.src=GarnetSite.pageUrl(page);
}
$('#focused-retry').onclick=()=>{focusedLoadedPage=null;ensureFocused(activeHash)};
function applyRoute(hash){if(pendingAnchor&&pendingAnchor.hash!==hash)pendingAnchor=null;const name=viewOf(hash);$$('.route').forEach(e=>e.hidden=e.id!==name+'-view');$('.navlinks').classList.remove('open');$('.menu-toggle').setAttribute('aria-expanded','false');document.body.classList.toggle('reference-mode',isReference(hash));document.body.classList.toggle('focused-mode',isFocused(hash));if(!isReference(hash)&&!isFocused(hash))lastExperienceHash=hash;$$('#navigation a').forEach(link=>link.setAttribute('aria-current',link.hash===hash||link.dataset.referenceChapter===hash||link.dataset.detailRoute===hash||link.dataset.sharedRoute===hash?'location':'false'));document.title=isFocused(hash)?'Garnet — '+focusedPages[hash][0]:hash==='#install'?'Get Garnet':'Garnet — inspect the authority';
 if(isFocused(hash)){const [title,page]=focusedPages[hash];$('#focused-title').textContent=title;focusedFrame.title='Garnet '+title;$('#focused-direct').href=GarnetSite.pageUrl(page);window.scrollTo({top:0,behavior:'instant'});void ensureFocused(hash)}
 else if(isReference(hash)){window.scrollTo({top:0,behavior:'instant'});if(referenceLoaded)restoreReference(hash)}
 else if(hash==='#use-cases'){if(routeScroll.has(hash))window.scrollTo({top:routeScroll.get(hash),behavior:'instant'});else $('#use-cases').scrollIntoView({behavior:'instant'})}
 else window.scrollTo({top:routeScroll.get(hash)||0,behavior:'instant'});
 if(activeHash!==null){const heading=isFocused(hash)?$('#focused-title'):isReference(hash)?$('#reference-title'):hash==='#install'?$('#install-title'):hash==='#use-cases'?$('#cases-title'):$('#hero-title');heading.setAttribute('tabindex','-1');if(armedLink){(armedLink.getClientRects().length?armedLink:$('.menu-toggle')).focus({preventScroll:true});}else heading.focus({preventScroll:true});}
}
async function route(){const hash=location.hash||'#home';if(hash==='#main'){$('#main').focus();return;}requestedHash=['#home','#install','#use-cases'].includes(hash)||isFocused(hash)||isReference(hash)?hash:'#home';if(routing)return;routing=true;const epoch=++routeEpoch;$('#main').setAttribute('aria-busy','true');try{while(activeHash!==requestedHash){let next=requestedHash;savePosition();const changedMode=activeHash!==null&&(isReference(next)!==isReference(activeHash)||isReference(next)&&next!==activeHash||isFocused(next)!==isFocused(activeHash)||isFocused(next)&&next!==activeHash);if(changedMode)await fadeMode(0,1,220);if(epoch!==routeEpoch)return;next=requestedHash;if(isReference(next))void ensureReference();applyRoute(next);activeHash=next;if(changedMode)await fadeMode(1,0,300);if(epoch!==routeEpoch)return;}}finally{if(epoch===routeEpoch){$('#mode-veil').style.opacity='0';$('#main').setAttribute('aria-busy','false');routing=false;applyPendingAnchor();}}}
function openSharedRoute(hash,{tour=false,reset=false,anchor=''}={}){disarm();savePosition();if(reset)routeScroll.delete(hash);const id=isFocused(hash)&&/^#[A-Za-z][\w-]*$/.test(anchor)?anchor.slice(1):null;if(id)routeScroll.delete(hash);go(hash,false,id);if(tour){state.scenario='check';renderScenario();void beginBoundaryTour();}}
window.GarnetNavigation=Object.freeze({open:openSharedRoute,disarm});
const pageRoutes={'why.html':'#page-why','minispec.html':'#page-docs','status.html':'#page-status','models.html':'#page-models'};
document.addEventListener('keydown',event=>{if(event.key==='Enter'&&event.target.closest('header a'))lastActivationHow='key';});
document.addEventListener('pointerdown',event=>{lastActivationHow=event.pointerType==='touch'||event.pointerType==='pen'?'touch':'click';});
document.addEventListener('click',event=>{
 const link=event.target.closest('a[href]');if(!link||event.button!==0||event.metaKey||event.ctrlKey||event.shiftKey||event.altKey)return;
 if(link===brand||link.closest('#navigation')&&link.dataset.referenceChapter){event.preventDefault();const deep=link===brand?'#reference':link.dataset.detailRoute||link.dataset.referenceChapter;const second=armedLink===link||event.detail>=2;
  if(second){savePosition();const replace=armCreatedEntry;disarm();routeScroll.delete(deep);if(deep==='#reference')routeScroll.set(deep,0);go(deep,replace);}
  else{const first=link===brand?'#home':link.hash;const created=location.hash!==first;savePosition();routeScroll.delete(first);arm(link,created,event.detail===0?'key':lastActivationHow);if(first===activeHash){$('.navlinks').classList.remove('open');$('.menu-toggle').setAttribute('aria-expanded','false');if(first==='#home')window.scrollTo({top:0,behavior:'instant'});else if(first==='#use-cases')$('#use-cases').scrollIntoView({behavior:'instant'});else if(isReference(first)&&referenceLoaded)restoreReference(first);}else go(first);}
  return;
 }
 if(link.dataset.sharedRoute){event.preventDefault();openSharedRoute(link.dataset.sharedRoute);return;}
 const destination=new URL(link.href,location.href),local=destination.origin===location.origin&&new URL('.',destination).pathname===new URL('.',location.href).pathname,pageRoute=local?pageRoutes[destination.pathname.split('/').pop()]:null;if(pageRoute){event.preventDefault();openSharedRoute(pageRoute,{anchor:destination.hash});return;}disarm();const hash=link.getAttribute('href');if(hash?.startsWith('#')&&(isReference(hash)||isFocused(hash)||['#home','#install','#use-cases'].includes(hash))){event.preventDefault();savePosition();if(hash==='#home'&&link.classList.contains('logo'))routeScroll.delete(hash);if(hash==='#use-cases')routeScroll.delete(hash);go(hash);if(link.hasAttribute('data-boundary-tour')){state.scenario='check';renderScenario();void beginBoundaryTour();}}
});
async function beginBoundaryTour(){await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));while(routing&&requestedHash==='#use-cases')await new Promise(resolve=>requestAnimationFrame(resolve));if(state.scenario!=='check'||activeHash!=='#use-cases')return;$('#allow-fs').checked=false;$('#allow-fs').dispatchEvent(new Event('change'));$('#run-demo').click();}
document.addEventListener('keydown',event=>{if(event.key!=='Escape')return;disarm();if($('.navlinks').classList.contains('open')){$('.navlinks').classList.remove('open');$('.menu-toggle').setAttribute('aria-expanded','false');$('.menu-toggle').focus();}});
document.addEventListener('click',event=>{if(!event.target.closest('header')&&$('.navlinks').classList.contains('open')){$('.navlinks').classList.remove('open');$('.menu-toggle').setAttribute('aria-expanded','false');}});
window.addEventListener('hashchange',()=>{disarm();void route();});
window.addEventListener('popstate',()=>{disarm();void route();});
window.addEventListener('pageshow',event=>{if(!event.persisted)return;disarm();routeEpoch++;$('#mode-veil').getAnimations().forEach(a=>a.cancel());$('#mode-veil').style.opacity='0';routing=false;activeHash=null;void route();});
$('.menu-toggle').onclick=()=>{const open=$('.navlinks').classList.toggle('open');$('.menu-toggle').setAttribute('aria-expanded',String(open));if(open)$('#navigation a').focus();};
// Keep the original document ready without revealing a presentation selector,
// unless the visitor asked the browser to save data.
if(!navigator.connection?.saveData){if('requestIdleCallback' in window)requestIdleCallback(()=>void ensureReference(),{timeout:2500});else setTimeout(()=>void ensureReference(),1200);}

function motion() {
  state.motion = motionPaused || reducedMotion.matches;
  document.documentElement.classList.toggle('motion-off', state.motion);document.body.classList.toggle('motion-off', state.motion);referenceDocument()?.body.classList.toggle('motion-off',state.motion);
  const control = $('#motion-toggle');
  control.disabled = reducedMotion.matches;
  control.textContent = reducedMotion.matches ? 'Reduced motion' : state.motion ? 'Enable motion' : 'Pause motion';
}
$('#motion-toggle').onclick = () => { motionPaused = !motionPaused; motion(); };
document.addEventListener('pointerdown',event=>{if(!event.target.closest('header'))disarm();});
reducedMotion.addEventListener('change', () => {
  motion();
  if (state.motion) $('#mode-veil').getAnimations().forEach(animation => animation.cancel());
});
motion();
// Reveals run once. Content already on screen at load, and every visit with motion off,
// shows at once; without IntersectionObserver nothing is hidden.
if('IntersectionObserver' in window){
  const revealer=new IntersectionObserver(entries=>entries.forEach(entry=>{if(entry.isIntersecting){entry.target.classList.add('in-view');revealer.unobserve(entry.target)}}),{rootMargin:'0px 0px -12% 0px'});
  $$('.evidence-chapter,.inscription').forEach(el=>{if(state.motion||el.getBoundingClientRect().top<innerHeight*.9)el.classList.add('in-view');else revealer.observe(el)});
  // Hide what waits below the fold without animating the hide: one flush with transitions off.
  const root=document.documentElement;root.classList.add('reveal-setup','has-reveal');void document.body.offsetHeight;root.classList.remove('reveal-setup');
}
// A verdict line settles in when the visitor can see it.
function settle(el){if(!el||state.motion||!el.animate)return;const r=el.getBoundingClientRect();if(r.bottom<0||r.top>innerHeight)return;el.animate([{opacity:.35,translate:'0 4px'},{opacity:1,translate:'0 0'}],{duration:320,easing:'cubic-bezier(.2,.75,.25,1)'})}
window.GarnetSettle=settle;
const commonLimit='Checks this example’s annotated call. It does not execute the filesystem call or prove general safety.';
const scenarios={
check:{title:'Declare access. Check the boundary.',kind:'Live Garnet checker',source:'C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md',limit:commonLimit},
diff:{title:'One change. A wider permission surface.',kind:'Live capability diff',source:'garnet-check-v0.3/src/caps_diff.rs',limit:'Compares the declared capability surfaces of the two versions.'},
provenance:{title:'Keep the origin attached to the source.',kind:'Illustrated provenance',source:'garnet-cli/src/cmd/verify.rs',limit:'Binds the source digest to the declared authority and its stated origin.'},
depth:{title:'Set the recursion ceiling.',kind:'Illustrated recursion guard',source:'C_Language_Specification/GARNET_BOUNDED_ENFORCEMENT.md',limit:'@max_depth caps recursion of the annotated function.'},
memory:{title:'Give memory a purpose.',kind:'Illustrated memory model',source:'C_Language_Specification/MEMORY_CORE_ROADMAP.md',limit:'Working, episodic, semantic and procedural memory are each a declaration kind.'},
actor:{title:'Change the behavior. Keep the queue.',kind:'Illustrated Rust actor runtime',source:'garnet-actor-runtime/src/lib.rs',limit:'Hot reload in the Rust actor runtime swaps the behavior and replays buffered messages against it.'}
};
function editorBar(name){return '<div class="filebar"><span><span class="file-dot">◇</span>'+name+'</span><span>GARNET</span></div>'}
function highlightCode(source){return source.split('\n').map((s,i)=>'<span class="code-line"><span class="code-no">'+(i+1)+'</span>'+escapeHtml(s).replace(/(@caps\([^)]*\))/g,'<span class="code-pink">$1</span>').replace(/(&quot;.*?&quot;)/g,'<span class="code-green">$1</span>')+'</span>').join('')}
function renderScenario(){window.GarnetSpectrum?.clear('Run a live check or compare a change to reveal its authority.');window.GarnetAcceptance.invalidate();const k=state.scenario,d=scenarios[k];if(k==='memory')state.memory=0;$$('.scenario-tab').forEach(b=>{let active=b.dataset.scenario===k;b.setAttribute('aria-selected',String(active));b.tabIndex=active?0:-1});$('#scenario-panel').setAttribute('aria-labelledby','tab-'+k);
if(k==='check'){window.GarnetAcceptance.mount({panel:$('#scenario-panel'),engine:getEngine,highlight:highlightCode,sourceLink:liveSource+'C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md',openRecord:()=>{state.scenario='provenance';renderScenario();$('#tab-provenance').focus({preventScroll:true})}});window.GarnetScenes.layout();return;}
if(k==='provenance'){void window.GarnetAcceptance.record({panel:$('#scenario-panel'),escape:escapeHtml}).then(()=>window.GarnetScenes.layout());return;}
let content='';
if(k==='diff'){content='<div class="lab-body"><div class="editor">'+editorBar('garnet diff-caps / before → after')+'<pre class="source" id="demo-code"></pre><div class="editor-tools"><label class="toggle-label"><input type="checkbox" id="add-authority" checked>Add network declaration</label><button class="btn small primary" id="run-demo">Compare authority <span aria-hidden="true">→</span></button></div></div><div class="live-output">'+window.GarnetScenes.diff()+'<h4 class="result-title" id="result-title">Review the authority, not just the lines.</h4><p class="result-copy" id="result-copy">Compare authority before and after the change.</p></div></div>'}
else{content=illustration(k)}
$('#scenario-panel').innerHTML='<div class="lab-top"><h3>'+d.title+'</h3><span class="tag '+(k==='diff'?'':'illustration')+'"><span class="dot"></span>'+d.kind+'</span></div>'+content+(k==='diff'?'<details class="raw-output"><summary>Inspect result JSON</summary><pre id="raw-result">Run a check to inspect its result.</pre></details>':'')+'<details class="lab-foot scope-details"><summary>Scope · '+({check:'Annotated network calls',diff:'Declared capabilities',provenance:'Source and manifest',depth:'Per-function depth',memory:'Four memory kinds',actor:'Rust actor runtime'}[k])+'</summary><p>'+d.limit+' <a href="status.html#workbench-scope">Full scope on Status</a></p><a href="'+liveSource+d.source+'" target="_blank" rel="noopener">'+newTab('Read the source')+'</a></details>';
if(k==='diff'){const raw=$('.raw-output');if(raw)$('.live-output').append(raw);}
if(k==='diff'){updateDiff();$('#add-authority').onchange=updateDiff;$('#run-demo').onclick=runDemo}else{$('#illustrate').onclick=()=>animateIllustration(k)}
window.GarnetScenes.layout();
}
function resetResult(title,copy){$('#result-title').textContent=title;$('#result-copy').textContent=copy;if($('#raw-result'))$('#raw-result').textContent='Run a check to inspect its result.'}
function updateDiff(){window.GarnetSpectrum?.clear('The source changed. Compare authority again.');const added=$('#add-authority').checked;$('#demo-code').innerHTML='<span class="code-comment"># before.garnet</span>\n<span class="code-pink">@caps(fs)</span>\ndef main() { 0 }\n\n<span class="code-comment"># after.garnet</span>\n<span class="code-pink">@caps('+ (added?'fs, net':'fs')+')</span>\ndef main() { 0 }';$('.permission-added').style.visibility=added?'visible':'hidden';window.GarnetScenes.phase('idle','Compare the authority declarations.');resetResult('Ready to compare.','Compare declared authority before and after the change.')}
let engineModuleAttempt = 0;
let enginePromise = null;
// One 20 s bound covers the glue import and the wasm init. A load that times out keeps
// running: if it finishes later it becomes the engine, and a retry races it against a fresh
// module instance, so neither a stalled request nor a slow one can strand the visitor.
const ENGINE_TIMEOUT_MS = 20000;
let lateLoads = [];
const checkerFailureCopy = error => /timed out/.test(String(error)) ? 'The checker took too long to load. Try again, or install the CLI and run garnet check.' : 'Try again, or install the CLI and run garnet check.';
window.GarnetCheckerFailureCopy = checkerFailureCopy;
function loadEngine(attempt) {
  const load = (async () => {
    const api = await import(attempt ? './playground/pkg/garnet_wasm.js?retry=' + attempt : './playground/pkg/garnet_wasm.js');
    await api.default({module_or_path:'playground/pkg/garnet_wasm_bg.wasm'});
    return api;
  })();
  load.then(api => { state.engine ??= api; }, () => {});
  return load;
}
async function getEngine() {
  if (state.engine) return state.engine;
  if (enginePromise) return enginePromise;
  enginePromise = (async () => {
    const loads = [loadEngine(engineModuleAttempt), ...lateLoads];
    let timer;
    try {
      const api = await Promise.race([Promise.any(loads), new Promise((_, reject) => { timer = setTimeout(() => reject(Error('Checker load timed out')), ENGINE_TIMEOUT_MS); })]);
      state.engine ??= api; lateLoads = [];
      return state.engine;
    } catch (error) {
      engineModuleAttempt++;
      lateLoads = /timed out/.test(String(error)) ? loads.slice(0, 2) : [];
      throw error;
    } finally { clearTimeout(timer); }
  })();
  try { return await enginePromise; } finally { enginePromise=null; }
}
async function runDemo() {
  if(state.busy) return;
  state.busy=true;
  const key=state.scenario, button=$('#run-demo'), original=button.innerHTML;
  const added=$('#add-authority').checked;
  button.disabled=true; button.textContent='Comparing…';window.GarnetScenes.phase('checking','Comparing the declared capability surface…');announce('Comparing declared authority…');
  try {
    const api=await getEngine();
    if(state.scenario!==key || !button.isConnected || $('#add-authority').checked!==added) return;
    const before='@caps(fs)\ndef main() { 0 }';
    const after='@caps('+(added?'fs, net':'fs')+')\ndef main() { 0 }';
    const result=JSON.parse(api.diff_caps_source(before,after));
    window.GarnetSpectrum?.set(window.GarnetSpectrum.fromDiff(result));
    window.GarnetScenes.phase(result.authority_expanded?'expanded':'unchanged',result.authority_expanded?'A network declaration was added. Human review is needed.':'No declared authority was added.');
    $('#result-title').textContent=result.authority_expanded?'Authority expanded: + net.':'No declared authority expansion.';
    $('#result-copy').textContent=result.authority_expanded?'The change adds network authority.':'The declared capability surface is unchanged.';
    $('#raw-result').textContent=JSON.stringify(result,null,2);settle($('#result-title'));
    announce(result.authority_expanded?'Authority expanded. The change adds network authority. Human review is needed.':'No declared authority expansion. The declared capability surface is unchanged.');
  } catch(error) {
    if(state.scenario===key && button.isConnected) {
      window.GarnetScenes.phase('error','Checker unavailable. No diff verdict was produced.');
      $('#result-title').textContent='The checker could not initialize.';
      $('#result-copy').textContent=checkerFailureCopy(error);
      $('#raw-result').textContent=String(error);window.GarnetSpectrum?.clear('Checker unavailable. No authority verdict was produced.');announce('Checker unavailable. No diff verdict was produced. '+checkerFailureCopy(error));
    }
  } finally {
    state.busy=false;
    if(button.isConnected) { button.disabled=false; button.innerHTML=original; }
  }
}
function illustration(k){
 const config={depth:{file:'recursion / illustrated ceiling',code:'@max_depth(3)\ndef recurse(n) { … }',comment:'A visual example of a per-function ceiling.',title:'Every call moves toward a boundary.',copy:'Three calls enter. The fourth reaches the limit.',button:'Trace four calls'},memory:{file:'memory / illustrated roles',code:'working     → current task\nepisodic    → events\nsemantic    → facts\nprocedural  → procedures',comment:'Sort information by the job it serves.',title:'Give each kind of memory a place.',copy:'Task. Event. Fact. Procedure.',button:'Follow the next card'},actor:{file:'ActorAddress::reload / illustrated workflow',code:'buffer messages\n  → swap behavior\n  → keep arrival order\n  → replay the queue',comment:'A visual explanation of the Rust actor-runtime workflow.',title:'New behavior. The same address.',copy:'Messages wait, then continue in order.',button:'Swap the behavior'}}[k];
 return '<div class="lab-body"><div class="editor">'+editorBar(config.file)+'<div class="illustration-text"><pre>'+config.code+'</pre><p class="comment">'+config.comment+'</p></div><div class="editor-tools"><span class="tiny">ILLUSTRATED WORKFLOW</span><button class="btn small" id="illustrate">'+config.button+' <span aria-hidden="true">→</span></button></div></div><div class="live-output">'+window.GarnetScenes[k]()+'<h4 class="result-title" id="result-title">'+config.title+'</h4><p class="result-copy" id="result-copy">'+config.copy+'</p></div></div>';
}
function animateIllustration(k){
 if(k==='memory')state.memory=(state.memory+1)%4;
 window.GarnetScenes.replay(k,state.memory);
 if(k==='depth'){$('#result-title').textContent='Call four meets the ceiling.';$('#result-copy').textContent='Illustrated limit: three calls enter; the next is refused.';}
 else if(k==='memory'){$('#result-title').textContent=['Working: the task at hand.','Episodic: what happened.','Semantic: what is known.','Procedural: how to repeat it.'][state.memory];}
 else{$('#result-title').textContent='Behavior v2. Messages 01 → 02 → 03.';$('#result-copy').textContent='Illustrated swap: the address and arrival order remain.';}
 announce({depth:'Call four meets the ceiling. Three calls enter; the next is refused.',actor:'Behavior v2. Messages one, two and three replay in order at the same address.'}[k]||$('#result-title').textContent);
}
$$('.scenario-tab').forEach((b,i)=>{b.onclick=()=>{state.scenario=b.dataset.scenario;renderScenario();b.scrollIntoView({block:'nearest',inline:'nearest',behavior:'instant'});const panel=$('#scenario-panel');if(!state.motion&&panel.animate)panel.animate([{opacity:.001,translate:'0 6px'},{opacity:1,translate:'0 0'}],{duration:220,easing:'cubic-bezier(.2,.75,.25,1)'})};b.onkeydown=e=>{const tabs=$$('.scenario-tab');let index=i;if(e.key==='ArrowRight')index=(i+1)%tabs.length;else if(e.key==='ArrowLeft')index=(i+tabs.length-1)%tabs.length;else if(e.key==='Home')index=0;else if(e.key==='End')index=tabs.length-1;else return;e.preventDefault();tabs[index].click();tabs[index].focus()}});
const platformText={mac:'macOS',windows:'Windows',linux:'Linux'};
function detectOS(){const p=(navigator.userAgentData?.platform||navigator.platform||'').toLowerCase(),ua=navigator.userAgent.toLowerCase();if(/iphone|ipad|android/.test(ua)||(p.includes('mac')&&navigator.maxTouchPoints>1))return null;if(p.includes('mac'))return'mac';if(p.includes('win'))return'windows';if(p.includes('linux'))return'linux';return null}
const detected=detectOS();state.os=detected||'mac';
function installCommand(){if(state.os==='windows')return state.mode==='release'?'irm https://garnet-lang.org/install.ps1 | iex':'cargo install --locked --git https://github.com/Island-Dev-Crew/garnet --tag v0.8.2 garnet-cli';return "curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | GARNET_INSTALL_MODE="+state.mode+' sh'}
function renderInstall(){const os=state.os,source=state.mode==='source';$$('[data-os]').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.os===os)));$$('[data-mode]').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.mode===state.mode)));$('#detect-message').textContent=detected?'Platform · '+platformText[detected]:'Select your platform.';$('#platform-heading').textContent='Garnet CLI for '+platformText[os];$('#shell-label').textContent=os==='windows'?'PowerShell 5.1 / 7+':'Terminal · sh';$('#install-command').textContent=installCommand();$('#install-promise').innerHTML='<span class="promise-icon" aria-hidden="true">'+(source?'◇':'✓')+'</span>'+(source?'Source build · Rust 1.95+':'Garnet CLI · v0.8.2');$('#install-detail').textContent=source?'Build Garnet CLI from the v0.8.2 source.':os==='windows'?'Windows x64 · SHA-256 verified':'Platform release · SHA-256 verified';$('#script-link').href=source?'https://rust-lang.org/tools/install/':os==='windows'?'https://garnet-lang.org/install.ps1':'https://garnet-lang.org/install.sh';$('#script-link').innerHTML=newTab(source?'Install Rust first':'Inspect installer');$('#install-requirements').innerHTML='<p>'+ (os==='linux'?'Linux packages need glibc 2.39 or newer; on older systems, build from source. ':os==='mac'?'Builds for Apple Silicon and Intel. ':'The Windows build is x86_64. ')+'The installer verifies every package against SHA256SUMS. To confirm who signed the checksums, use the checksum and signing guide. <a class="text-link" href="status.html#release-details">Release details</a></p>';renderDownloadSelectors()}
function renderDownloadSelectors(){const os=state.os;$('#architecture').innerHTML='<option value="">Choose processor…</option>'+(os==='mac'?'<option value="arm64">Apple Silicon</option><option value="x64">Intel</option>':os==='linux'?'<option value="x64">x86_64 / AMD64</option><option value="arm64">ARM64 / AArch64</option>':'<option value="x64">Intel / AMD x86_64</option><option value="arm64">ARM64 · x64 emulation</option>');$('#package-format').innerHTML=os==='linux'?'<option value="tar">Tarball (.tar.gz)</option><option value="deb">Debian / Ubuntu (.deb)</option><option value="rpm">Fedora / RPM (.rpm)</option>':os==='mac'?'<option value="tar">CLI tarball (.tar.gz)</option>':'<option value="zip">CLI archive (.zip)</option>';updateDownload()}
function updateDownload(){const arch=$('#architecture').value,format=$('#package-format').value,os=state.os,link=$('#download-link');if(!arch){link.removeAttribute('href');link.setAttribute('aria-disabled','true');link.textContent='Choose a processor ↓';$('#download-meta').textContent='v0.8.2 · Release packages';return}let asset='';if(os==='mac')asset='garnet-0.8.2-'+(arch==='arm64'?'aarch64':'x86_64')+'-apple-darwin.tar.gz';else if(os==='windows')asset='garnet-0.8.2-x86_64-pc-windows-msvc.zip';else if(format==='deb')asset='garnet_0.8.2-1_'+(arch==='arm64'?'arm64':'amd64')+'.deb';else if(format==='rpm')asset='garnet-0.8.2-1.'+(arch==='arm64'?'aarch64':'x86_64')+'.rpm';else asset='garnet-0.8.2-'+(arch==='arm64'?'aarch64':'x86_64')+'-unknown-linux-gnu.tar.gz';const base='https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.2/';link.href=base+asset;link.setAttribute('aria-disabled','false');link.textContent='Download '+(format==='tar'?'.tar.gz':'.'+format)+' ↓';$('#download-meta').innerHTML=escapeHtml(asset)+(os==='windows'&&arch==='arm64'?' · x64 emulation required':'')+'<br><a href="'+base+'SHA256SUMS" target="_blank" rel="noopener">'+newTab('SHA256SUMS')+'</a> &nbsp; · &nbsp; <a href="'+base+'SHA256SUMS.asc" target="_blank" rel="noopener">'+newTab('GPG signature')+'</a>'}
$$('[data-os]').forEach(b=>b.onclick=()=>{state.os=b.dataset.os;renderInstall()});$$('[data-mode]').forEach(b=>b.onclick=()=>{state.mode=b.dataset.mode;renderInstall()});$('#architecture').onchange=updateDownload;$('#package-format').onchange=updateDownload;$('#copy-install').onclick=()=>copyText(installCommand());$('#verification-link').href='https://github.com/Island-Dev-Crew/garnet/blob/aba11b329e3284845ba7c9cd4c870998503a9030/docs/release-signing.md';// A workbench scene that fails to load must not stop the install view or the router.
try{renderScenario()}catch(error){$('#scenario-panel').innerHTML='<p class="result-copy">The workbench could not load here. <a href="playground.html">Open the playground</a> instead.</p>';reportError?.(error)}
renderInstall();route();

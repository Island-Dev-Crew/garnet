"use strict";
(() => {
  window.GarnetSite = { pageUrl: page => new URL(page, location.href).href };
  document.documentElement.dataset.garnetPage = location.pathname.split('/').pop() || 'index.html';
  let embedded = false;
  try {
    embedded = parent !== window;
    if (window.frameElement?.id === 'original-frame') document.documentElement.classList.add('original-embedded');
    if (window.frameElement?.id === 'focused-frame') document.documentElement.classList.add('focused-embedded');
  } catch {}
  if (!embedded) return;
  const prepareLink = link => {
    const href = link?.getAttribute('href');
    if (href && !href.startsWith('#')) link.target = '_top';
  };
  document.querySelectorAll('a[href]').forEach(prepareLink);
  document.addEventListener('click', event => prepareLink(event.target.closest('a[href]')), true);
  const pageRoutes={'why.html':'#page-why','minispec.html':'#page-docs','status.html':'#page-status','models.html':'#page-models'};
  document.addEventListener('pointerdown',()=>{try{parent.GarnetNavigation?.disarm()}catch{}});
  document.addEventListener('click',event=>{
    const link=event.target.closest('a[href]');
    if(!link||event.defaultPrevented||event.button!==0||event.metaKey||event.ctrlKey||event.shiftKey||event.altKey||link.hasAttribute('download'))return;
    const href=link.getAttribute('href');if(!href||href.startsWith('#'))return;
    const destination=new URL(link.href,location.href);if(destination.origin!==location.origin||new URL('.',destination).pathname!==new URL('.',location.href).pathname)return;
    const file=destination.pathname.split('/').pop(),hash=pageRoutes[file]||(file==='index.html'&&['','#home','#use-cases','#install','#why','#docs','#readiness','#reference'].includes(destination.hash)?destination.hash||'#home':null);
    if(!hash)return;
    try{if(parent.GarnetNavigation){event.preventDefault();parent.GarnetNavigation.open(hash,{tour:link.hasAttribute('data-boundary-tour'),reset:hash==='#home'||hash==='#use-cases',anchor:pageRoutes[file]?destination.hash:''});}}catch{}
  });
  document.addEventListener('keydown', event => {
    if (event.key !== 'Escape') return;
    try { parent.GarnetNavigation?.disarm(); } catch {}
    try {
      const link = parent.document.querySelector('#garnet-brand');
      (link?.getClientRects().length ? link : parent.document.querySelector('.menu-toggle'))?.focus();
    } catch {}
  });
})();

'use strict';
// Dedicated documentation and playground pages retain a visible shell return.
try {
  const saved = JSON.parse(sessionStorage.getItem('garnet-front-door-position') || '{}');
  const hash = ['#home','#use-cases','#install'].includes(saved.experience) ? saved.experience : '#home';
  document.querySelectorAll('[data-experience-return]').forEach(link => { link.href = 'index.html' + hash; });
} catch { /* The normal home link remains usable without storage. */ }

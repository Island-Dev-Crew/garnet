/* Shared visual preferences only. No runtime, verdict, or OS authority changes. */
(() => {
  'use strict';
  const themes = [
    ['garnet', 'Garnet', '#0c0d0f', '#101114', '#211c24', '#f2e8d8', '#b0adb3', '#d7b572', '#f37d9a', '#abcbb6', 'dark'],
    ['matte-black', 'Matte Black', '#121212', '#191919', '#262626', '#f2f2f2', '#b5b5b5', '#e7ac65', '#ff91a0', '#9fd69e', 'dark'],
    ['tokyo-night', 'Tokyo Night', '#1a1b26', '#202231', '#292e42', '#c0caf5', '#a9b4df', '#9db9ff', '#ff9db6', '#99d6c2', 'dark'],
    ['catppuccin', 'Catppuccin', '#1e1e2e', '#242438', '#313244', '#cdd6f4', '#bac2de', '#cba6f7', '#f5a6c9', '#a6e3a1', 'dark'],
    ['nord', 'Nord', '#2e3440', '#3b4252', '#434c5e', '#eceff4', '#d8dee9', '#92cddd', '#ffa8b4', '#a3d4b1', 'dark'],
    ['everforest', 'Everforest', '#272e33', '#2f383d', '#374247', '#d3c6aa', '#d0c2ac', '#aecba6', '#ffa5aa', '#a7d6a4', 'dark'],
    ['gruvbox', 'Gruvbox', '#282828', '#32302f', '#3c3836', '#fbf1c7', '#d5c4a1', '#fabd2f', '#fe9b90', '#9aca7d', 'dark'],
    ['latte', 'Catppuccin Latte', '#eff1f5', '#e6e9ef', '#dce0e8', '#4c4f69', '#5c5f77', '#3d3cb0', '#800f48', '#40652b', 'light'],
  ];
  const key = 'garnet.playground.theme.v2';
  const scopes = [...document.querySelectorAll('[data-playground-theme]')];
  if (!scopes.length) return;
  let selected = 'garnet';
  try { selected = localStorage.getItem(key) || selected; } catch { /* Private/blocked storage remains usable. */ }
  const valid = id => themes.some(theme => theme[0] === id);
  const controls = [];
  const apply = (id, persist = true, announce = true) => {
    const theme = themes.find(item => item[0] === id) || themes[0];
    selected = theme[0];
    const [slug, label, bg, surface, surface2, ink, muted, accent, danger, ok, scheme] = theme;
    const values = { '--bg': bg, '--surface': surface, '--surface2': surface2, '--surface-2': surface2,
      '--ink': ink, '--ink-soft': muted, '--muted': muted, '--gold': accent, '--bright': danger,
      '--green': ok, '--ok': ok, '--danger': danger, '--garnet': accent, '--garnet-dark': surface2,
      '--line': muted, '--rule': muted, '--theme-accent': accent };
    for (const scope of scopes) {
      scope.dataset.playgroundTheme = slug;
      scope.style.colorScheme = scheme;
      Object.entries(values).forEach(([name, value]) => scope.style.setProperty(name, value));
    }
    for (const control of controls) {
      control.select.value = slug;
      if (announce) control.status.textContent = `${label} theme`;
    }
    if (persist) try { localStorage.setItem(key, slug); } catch { /* Never disable controls on storage failure. */ }
    document.dispatchEvent(new CustomEvent('garnet:themechange', { detail: { id: slug, name: label } }));
  };
  for (const [index, slot] of [...document.querySelectorAll('[data-theme-controls]')].entries()) {
    const label = document.createElement('label');
    label.textContent = 'Theme';
    const select = document.createElement('select');
    select.id = `playground-theme-${index}`;
    select.setAttribute('aria-label', 'Playground theme');
    label.htmlFor = select.id;
    for (const [value, name] of themes) select.add(new Option(name, value));
    select.addEventListener('change', () => apply(select.value));
    const cycle = document.createElement('button');
    cycle.type = 'button';
    cycle.className = 'theme-cycle';
    cycle.textContent = 'Next theme';
    cycle.setAttribute('aria-label', 'Next playground theme');
    cycle.title = 'Next theme (T, outside the editor)';
    cycle.addEventListener('click', () => next());
    const hint = document.createElement('span');
    hint.className = 'theme-hint';
    hint.textContent = 'or press T';
    const status = document.createElement('span');
    status.className = 'theme-status';
    status.setAttribute('role', 'status');
    status.setAttribute('aria-live', 'polite');
    slot.append(label, select, cycle, hint, status);
    controls.push({ select, status });
  }
  const next = () => apply(themes[(themes.findIndex(theme => theme[0] === selected) + 1) % themes.length][0]);
  document.addEventListener('keydown', event => {
    if (event.defaultPrevented || event.repeat || event.isComposing || event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return;
    const target = event.target;
    if (target instanceof Element && target.closest('input, textarea, select, button, a, [contenteditable]:not([contenteditable="false"]), [role="textbox"]')) return;
    if (event.key.toLowerCase() !== 't') return;
    // The front door shortcut exists only while its playground is visible.
    if (!scopes.some(scope => scope.getClientRects().length && scope.getBoundingClientRect().bottom > 0 && scope.getBoundingClientRect().top < innerHeight)) return;
    event.preventDefault(); next();
  });
  window.addEventListener('storage', event => {
    if (event.key === key) apply(valid(event.newValue) ? event.newValue : 'garnet', false);
  });
  apply(valid(selected) ? selected : 'garnet', false, false);
})();

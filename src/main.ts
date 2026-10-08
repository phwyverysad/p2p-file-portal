import App from './App.svelte';

// Prevent default browser context menu (right-click)
window.addEventListener('contextmenu', (e) => {
  e.preventDefault();
}, true);

// Prevent F12 and DevTools shortcuts
window.addEventListener('keydown', (e) => {
  if (
    e.key === 'F12' ||
    (e.ctrlKey && e.shiftKey && ['I', 'i', 'J', 'j', 'C', 'c'].includes(e.key)) ||
    (e.ctrlKey && ['u', 'U'].includes(e.key))
  ) {
    e.preventDefault();
    e.stopPropagation();
  }
}, true);

const app = new App({
  target: document.getElementById('app')!,
});

export default app;

import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'light' | 'dark' | 'space' | 'nord' | 'forest' | 'vintage' | 'slate' | 'cloudy';
const THEMES: Theme[] = ['light', 'dark', 'space', 'nord', 'forest', 'vintage', 'slate', 'cloudy'];
const THEME_CLASSES = ['dark', 'space', 'nord', 'forest', 'vintage', 'slate', 'cloudy'];

function readInitial(): Theme {
  if (!browser) return 'light';
  const stored = localStorage.getItem('theme') as Theme | null;
  if (stored && THEMES.includes(stored)) return stored;
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function apply(next: Theme) {
  if (!browser) return;
  const root = document.documentElement;
  root.classList.remove(...THEME_CLASSES);
  if (next !== 'light') root.classList.add(next);
  localStorage.setItem('theme', next);
}

function createTheme() {
  const initial = readInitial();
  const { subscribe, set, update } = writable<Theme>(initial);
  if (browser) apply(initial);

  return {
    subscribe,
    set(next: Theme) {
      apply(next);
      set(next);
    },
    cycle() {
      update((curr) => {
        const idx = THEMES.indexOf(curr);
        const next = THEMES[(idx + 1) % THEMES.length];
        apply(next);
        return next;
      });
    },
  };
}

export const theme = createTheme();

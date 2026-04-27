import { writable } from 'svelte/store';

export type VisualEffect = 'none' | 'aurora' | 'particles' | 'rain' | 'matrix';

export const visualEffect = writable<VisualEffect>('none');

import { writable } from 'svelte/store';

export type BackgroundEffect = 'none' | 'particles' | 'rain' | 'snow' | 'fireflies' | 'fog';
export type SectionEffect = 'none' | 'breathe' | 'scanlines';

export const backgroundEffect = writable<BackgroundEffect>('none');
export const sectionEffect = writable<SectionEffect>('none');

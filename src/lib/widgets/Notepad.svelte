<script lang="ts">
  import { save } from '@tauri-apps/plugin-dialog';
  import { saveToPath } from '$lib/services/notepad';

  const STORAGE_KEY = 'notepad-content';

  let content = $state(localStorage.getItem(STORAGE_KEY) ?? '');
  let savedFlash = $state(false);

  $effect(() => {
    localStorage.setItem(STORAGE_KEY, content);
  });

  async function saveAs() {
    const path = await save({
      defaultPath: 'notepad.md',
      filters: [
        { name: 'Markdown', extensions: ['md'] },
        { name: 'Text', extensions: ['txt'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    });
    if (!path) return;
    await saveToPath(content, path);
    savedFlash = true;
    setTimeout(() => (savedFlash = false), 1500);
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between mb-3">
    <h2 class="text-xs uppercase tracking-wider text-mute">Notepad</h2>
    <button
      class="text-xs px-2 py-0.5 rounded border border-line hover:bg-surface transition text-mute hover:text-ink"
      onclick={saveAs}
    >
      {savedFlash ? 'Saved ✓' : 'Save'}
    </button>
  </div>

  <textarea
    class="flex-1 resize-none bg-transparent text-ink placeholder:text-mute text-sm leading-relaxed outline-none w-full"
    placeholder="Quick notes…"
    bind:value={content}
    onkeydown={(e) => {
      if (e.key === 'Tab') {
        e.preventDefault();
        const el = e.currentTarget;
        const start = el.selectionStart;
        const end = el.selectionEnd;
        content = content.slice(0, start) + '    ' + content.slice(end);
        requestAnimationFrame(() => { el.selectionStart = el.selectionEnd = start + 4; });
      }
    }}
  ></textarea>
</div>

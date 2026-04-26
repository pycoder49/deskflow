<script lang="ts">
  import { createTask, LISTS, AREAS } from '$lib/services/clickup';
  import { bumpClickup } from '$lib/stores/refresh';

  let { open = $bindable(false), onCreated }: {
    open: boolean;
    onCreated?: () => void;
  } = $props();

  let title = $state('');
  let listId = $state<string>(LISTS[0].id);
  let areaSlug = $state<string>(AREAS[0].slug);
  let priority = $state(3);
  let saving = $state(false);
  let inputEl = $state<HTMLInputElement | null>(null);

  const selectedList = $derived(LISTS.find((l) => l.id === listId));
  const isDaily = $derived(selectedList?.slug === null);

  $effect(() => {
    if (open && inputEl) inputEl.focus();
  });

  function reset() {
    title = '';
    listId = LISTS[0].id;
    areaSlug = AREAS[0].slug;
    priority = 3;
  }

  function close() {
    open = false;
    reset();
  }

  async function save() {
    if (!title.trim() || saving) return;
    saving = true;
    try {
      const tagSlug = selectedList?.slug ?? areaSlug;
      const tags = tagSlug ? [tagSlug] : [];
      await createTask(title.trim(), listId, priority, tags);
      bumpClickup();
      onCreated?.();
      close();
    } catch (e) {
      console.error('create task failed:', e);
    } finally {
      saving = false;
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'Escape') close();
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) save();
  }
</script>

<svelte:window onkeydown={handleKey} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-start justify-center pt-32 bg-black/40 backdrop-blur-sm"
    onclick={close}
    role="presentation"
  >
    <div
      class="bg-surface border border-line rounded-xl w-[min(32rem,90vw)] p-5 shadow-2xl"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-label="Quick capture"
      tabindex="-1"
    >
      <h2 class="text-xs uppercase tracking-wider text-mute mb-3">Quick Capture</h2>

      <input
        bind:this={inputEl}
        class="w-full bg-transparent text-lg text-ink outline-none border-b border-line pb-2 mb-4 placeholder:text-mute"
        placeholder="What needs doing?"
        bind:value={title}
        disabled={saving}
      />

      <div class="flex flex-wrap items-center gap-3 text-sm">
        <select
          class="bg-surface border border-line rounded px-2 py-1 text-ink"
          bind:value={listId}
          aria-label="List"
          disabled={saving}
        >
          {#each LISTS as list}
            <option value={list.id}>{list.label}</option>
          {/each}
        </select>

        {#if isDaily}
          <select
            class="bg-surface border border-line rounded px-2 py-1 text-ink"
            bind:value={areaSlug}
            aria-label="Area"
            disabled={saving}
          >
            {#each AREAS as area}
              <option value={area.slug}>{area.label}</option>
            {/each}
          </select>
        {/if}

        <select
          class="bg-surface border border-line rounded px-2 py-1 text-ink"
          bind:value={priority}
          aria-label="Priority"
          disabled={saving}
        >
          <option value={1}>Urgent</option>
          <option value={2}>High</option>
          <option value={3}>Normal</option>
        </select>

        <div class="flex-1"></div>

        <button class="text-mute hover:text-ink text-sm" onclick={close} disabled={saving}>
          Cancel
        </button>
        <button
          class="bg-accent text-white px-3 py-1 rounded text-sm disabled:opacity-50"
          onclick={save}
          disabled={!title.trim() || saving}
        >
          {saving ? 'Saving…' : 'Save'}
        </button>
      </div>

      <p class="text-xs text-mute mt-3">Ctrl+Enter to save · Esc to dismiss</p>
    </div>
  </div>
{/if}

<script lang="ts">
  import { createTask, PRIORITY_META } from '$lib/services/clickup';
  import { lists, areas } from '$lib/stores/config';
  import { bumpClickup, pushNewTask, suppressNextTodayLoad } from '$lib/stores/refresh';

  let { open = $bindable(false), onCreated }: {
    open: boolean;
    onCreated?: () => void;
  } = $props();

  let title = $state('');
  let listId = $state<string>('');
  let areaSlug = $state<string>('');
  let priority = $state(3);
  let saving = $state(false);
  let inputEl = $state<HTMLInputElement | null>(null);

  // Initialise from config once it loads (config is fetched in +layout).
  $effect(() => {
    if (!listId && $lists.length) listId = $lists[0].id;
    if (!areaSlug && $areas.length) areaSlug = $areas[0].slug;
  });

  const selectedList = $derived($lists.find((l) => l.id === listId));
  const isDaily = $derived(selectedList?.slug === null);

  $effect(() => {
    if (open && inputEl) inputEl.focus();
  });

  function reset() {
    title = '';
    listId = $lists[0]?.id ?? '';
    areaSlug = $areas[0]?.slug ?? '';
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
      const created = await createTask(title.trim(), listId, priority, tags);
      if (isDaily) {
        // Inject directly into Today; suppress the reload bumpClickup would trigger.
        suppressNextTodayLoad.set(true);
        pushNewTask(created);
      }
      bumpClickup(); // NowNext re-picks; Today's load suppressed for daily creates.
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
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
    onclick={(e) => { if (e.target === e.currentTarget) close(); }}
  >
    <div class="bg-surface border border-line rounded-2xl shadow-2xl w-[500px] overflow-hidden">

      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-line">
        <div class="flex items-center gap-2.5">
          <svg class="w-3.5 h-3.5 text-accent" fill="none" viewBox="0 0 14 14" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M7 2v10M2 7h10"/>
          </svg>
          <h3 class="text-sm font-semibold text-ink">New Task</h3>
        </div>
        <button
          class="text-mute hover:text-ink transition p-1.5 rounded-lg hover:bg-line"
          onclick={close}
          aria-label="Close"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
            <path d="M2 2l10 10M12 2L2 12"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="px-6 py-5 space-y-5">

        <!-- Title -->
        <div>
          <span class="text-[10px] font-semibold text-mute uppercase tracking-widest block mb-1.5">Task</span>
          <input
            bind:this={inputEl}
            class="w-full bg-base border border-line rounded-xl px-4 py-2.5 text-sm text-ink focus:outline-none focus:border-accent transition placeholder:text-mute"
            placeholder="What needs doing?"
            bind:value={title}
            disabled={saving}
            onkeydown={(e) => { if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) save(); }}
          />
        </div>

        <!-- List -->
        <div>
          <span class="text-[10px] font-semibold text-mute uppercase tracking-widest block mb-1.5">List</span>
          <div class="grid grid-cols-3 gap-1.5">
            {#each $lists as list}
              <button
                class="px-2 py-2 rounded-xl text-xs font-medium border transition text-left truncate
                       {listId === list.id
                         ? 'bg-accent/10 border-accent text-accent'
                         : 'bg-base border-line text-mute hover:border-line/70 hover:text-ink'}"
                onclick={() => listId = list.id}
                disabled={saving}
              >{list.label}</button>
            {/each}
          </div>
        </div>

        <!-- Area (Daily To-Do only) -->
        {#if isDaily}
          <div>
            <span class="text-[10px] font-semibold text-mute uppercase tracking-widest block mb-1.5">Area</span>
            <div class="flex flex-wrap gap-1.5">
              {#each $areas as area}
                <button
                  class="px-3 py-2 rounded-xl text-xs font-medium border transition
                         {areaSlug === area.slug
                           ? 'bg-accent/10 border-accent text-accent'
                           : 'bg-base border-line text-mute hover:border-line/70 hover:text-ink'}"
                  onclick={() => areaSlug = area.slug}
                  disabled={saving}
                >{area.label}</button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Priority -->
        <div>
          <span class="text-[10px] font-semibold text-mute uppercase tracking-widest block mb-1.5">Priority</span>
          <div class="flex gap-1.5">
            {#each Object.entries(PRIORITY_META).sort(([a], [b]) => Number(b) - Number(a)) as [id, meta]}
              <button
                class="flex-1 px-2 py-2 rounded-xl text-xs font-medium border transition"
                style="{priority === Number(id)
                  ? `background:${meta.color}22;border-color:${meta.color};color:${meta.color}`
                  : 'background:var(--color-base);border-color:var(--color-line);color:var(--color-mute)'}"
                onmouseenter={(e) => {
                  if (priority !== Number(id)) {
                    (e.currentTarget as HTMLElement).style.borderColor = meta.color;
                    (e.currentTarget as HTMLElement).style.color = meta.color;
                  }
                }}
                onmouseleave={(e) => {
                  if (priority !== Number(id)) {
                    (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-line)';
                    (e.currentTarget as HTMLElement).style.color = 'var(--color-mute)';
                  }
                }}
                onclick={() => priority = Number(id)}
                disabled={saving}
              >{meta.label}</button>
            {/each}
          </div>
        </div>

      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between px-6 py-3.5 border-t border-line bg-base/40">
        <span class="text-[11px] text-mute/60">⌘↵ save · esc cancel</span>
        <div class="flex gap-2">
          <button
            class="px-4 py-1.5 text-sm text-mute hover:text-ink transition rounded-lg hover:bg-line"
            onclick={close}
            disabled={saving}
          >Cancel</button>
          <button
            class="px-4 py-1.5 text-sm bg-accent text-white rounded-lg hover:opacity-90 transition disabled:opacity-50 font-medium"
            onclick={save}
            disabled={!title.trim() || saving}
          >{saving ? 'Saving…' : 'Add task'}</button>
        </div>
      </div>

    </div>
  </div>
{/if}

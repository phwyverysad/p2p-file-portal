<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import { type ShareItem, formatBytes, getFileCategoryIcon } from '../lib/api';

  export let isOpen = false;
  export let item: ShareItem | null = null;
  export let lang: Language = 'th';
  export let onClose: () => void;
  export let onConfirm: (id: string, dontShowAgain?: boolean) => Promise<void>;

  let dontShowAgain = false;
  let thumbFailed = false;

  $: t = I18N[lang];
  $: iconType = item ? getFileCategoryIcon(item.originalName, item.isDirectory) : 'file';

  // Reset checkbox when modal closes or item changes
  $: if (!isOpen) {
    dontShowAgain = false;
    thumbFailed = false;
  }

  async function handleConfirm() {
    if (!item) return;
    await onConfirm(item.id, dontShowAgain);
    onClose();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

{#if isOpen && item}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
  <div 
    class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4 animate-fade-in"
    on:click={handleBackdropClick}
    role="dialog"
    tabindex="-1"
  >
    <div class="bg-white rounded-2xl border border-slate-200/90 shadow-2xl max-w-md w-full overflow-hidden animate-scale-up">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-slate-100 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-rose-50 text-rose-600 flex items-center justify-center">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="3 6 5 6 21 6"></polyline>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
            </svg>
          </div>
          <span class="font-bold text-slate-900 text-sm">{t.deleteModalTitle}</span>
        </div>
        <button type="button" on:click={onClose} class="text-slate-400 hover:text-slate-600 p-1.5 rounded-lg hover:bg-slate-100">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>

      <!-- Body -->
      <div class="p-5 space-y-4">
        <p class="text-xs text-slate-600">{t.deleteModalPrompt}</p>

        <!-- Preview Card -->
        <div class="p-3 bg-slate-50 border border-slate-200/90 rounded-xl flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-white border border-slate-200 text-slate-600 flex items-center justify-center shrink-0 p-1 overflow-hidden">
            {#if item.thumbnailUrl && !thumbFailed}
              <img src={item.thumbnailUrl} alt="" class="w-full h-full object-contain" on:error={() => thumbFailed = true} />
            {:else if item.isDirectory || iconType === 'folder'}
              <svg class="w-5 h-5 text-amber-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
            {:else}
              <svg class="w-5 h-5 text-sky-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>
            {/if}
          </div>
          <div class="min-w-0">
            <div class="text-xs font-bold text-slate-900 truncate">{item.title}</div>
            <div class="text-[11px] text-slate-500 mt-0.5">{formatBytes(item.size)} • {item.originalName}</div>
          </div>
        </div>

        <!-- Warning Note -->
        <div class="p-2.5 bg-amber-50 border border-amber-200 rounded-lg flex items-start gap-2 text-[11px] text-amber-800">
          <svg class="w-4 h-4 text-amber-600 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
            <line x1="12" y1="9" x2="12" y2="13"></line>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
          <span>{t.deleteModalNote}</span>
        </div>

        <!-- Don't ask again checkbox -->
        <label class="flex items-center gap-2.5 pt-1 cursor-pointer select-none text-xs text-slate-600 hover:text-slate-900">
          <input 
            type="checkbox" 
            bind:checked={dontShowAgain}
            class="w-4 h-4 rounded border-slate-300 text-sky-600 focus:ring-sky-500 cursor-pointer"
          />
          <span class="font-medium">{t.deleteModalDontShowAgain}</span>
        </label>
      </div>

      <!-- Footer -->
      <div class="px-5 py-3.5 bg-slate-50 border-t border-slate-100 flex justify-end gap-2">
        <button type="button" on:click={onClose} class="btn-secondary !text-xs !py-1.5 !px-3">
          {t.btnCancel}
        </button>
        <button 
          type="button" 
          on:click={handleConfirm}
          class="px-3.5 py-1.5 text-xs font-semibold rounded-lg bg-rose-600 hover:bg-rose-700 text-white transition-colors flex items-center gap-1.5"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
          <span>{t.btnConfirmDelete}</span>
        </button>
      </div>
    </div>
  </div>
{/if}

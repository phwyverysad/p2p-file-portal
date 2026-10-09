<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import type { ShareItem } from '../lib/api';
  import ShareItemRow from './ShareItemRow.svelte';

  export let shares: ShareItem[] = [];
  export let lang: Language = 'th';
  export let onEdit: (item: ShareItem) => void;
  export let onDelete: (item: ShareItem) => void;
  export let onShowToast: (msg: string) => void;
  export let isProcessing = false;
  export let processingFileName = '';

  $: t = I18N[lang];
</script>

<section class="card mb-6">
  <div class="flex items-center justify-between mb-4">
    <div class="flex items-center gap-2">
      <h2 class="text-[15px] font-bold text-slate-800">{t.sectionAllFiles}</h2>
      <span class="badge text-sky-700 bg-sky-50 border-sky-200">
        {shares.length} {t.filesUnit}
      </span>
    </div>
  </div>

  {#if isProcessing}
    <!-- Active Processing / Right-Click Loading Card -->
    <div class="p-3.5 bg-sky-50/80 border border-sky-200 rounded-2xl flex items-center justify-between gap-3 mb-3 animate-pulse">
      <div class="flex items-center gap-3 min-w-0">
        <div class="w-10 h-10 rounded-xl bg-white border border-sky-200 flex items-center justify-center shrink-0 shadow-sm text-sky-600">
          <svg class="w-5 h-5 animate-spin" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
          </svg>
        </div>
        <div class="min-w-0">
          <div class="text-xs font-bold text-sky-950 truncate">
            {processingFileName || t.loading}
          </div>
          <div class="text-[11px] text-sky-700 font-medium mt-0.5">
            {t.loadingSharePrompt}
          </div>
        </div>
      </div>
      <span class="badge text-[10px] bg-sky-100 text-sky-700 border-sky-300 shrink-0">
        {t.loading}
      </span>
    </div>
  {/if}

  {#if shares.length === 0 && !isProcessing}
    <!-- Empty State -->
    <div class="py-12 px-4 text-center flex flex-col items-center justify-center">
      <div class="w-14 h-14 rounded-2xl bg-slate-50 border border-slate-100 text-slate-300 flex items-center justify-center mb-3">
        <svg class="w-7 h-7" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="12" y1="18" x2="12" y2="12"></line>
          <line x1="9" y1="15" x2="15" y2="15"></line>
        </svg>
      </div>
      <h3 class="text-sm font-semibold text-slate-700 mb-1">{t.emptyState}</h3>
      <p class="text-xs text-slate-400 max-w-sm leading-relaxed">{t.emptyStateSub}</p>
    </div>
  {:else if shares.length > 0}
    <div class="space-y-3">
      {#each shares as item (item.id)}
        <ShareItemRow 
          {item} 
          {lang} 
          {onEdit} 
          {onDelete} 
          {onShowToast} 
        />
      {/each}
    </div>
  {/if}
</section>

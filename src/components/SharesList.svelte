<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import type { ShareItem } from '../lib/api';
  import ShareItemRow from './ShareItemRow.svelte';

  export let shares: ShareItem[] = [];
  export let lang: Language = 'th';
  export let onEdit: (item: ShareItem) => void;
  export let onDelete: (item: ShareItem) => void;
  export let onShowToast: (msg: string) => void;

  $: t = I18N[lang];
</script>

<section class="card mb-12">
  <div class="flex items-center justify-between mb-4">
    <div class="flex items-center gap-2">
      <h2 class="text-[15px] font-bold text-slate-800">{t.sectionAllFiles}</h2>
      <span class="badge text-sky-700 bg-sky-50 border-sky-200">
        {shares.length} {t.filesUnit}
      </span>
    </div>
  </div>

  {#if shares.length === 0}
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
  {:else}
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

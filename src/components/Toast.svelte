<script lang="ts">
  export let message = '';
  export let visible = false;
  export let actionText: string | undefined = undefined;
  export let onAction: (() => void) | undefined = undefined;
</script>

{#if visible}
  <div class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-2.5 bg-white/95 backdrop-blur-md text-slate-800 text-xs font-semibold px-4 py-2 rounded-full border border-slate-200/90 shadow-xl shadow-slate-300/40 animate-bounce-in max-w-[92vw] sm:max-w-lg select-none">
    <div class="w-4 h-4 rounded-full bg-emerald-100 text-emerald-600 flex items-center justify-center shrink-0">
      <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="20 6 9 17 4 12"></polyline>
      </svg>
    </div>
    <span class="tracking-wide text-slate-800 truncate">{message}</span>
    {#if actionText && onAction}
      <button
        type="button"
        on:click={onAction}
        class="ml-1.5 px-2.5 py-1 bg-sky-600 hover:bg-sky-700 active:scale-95 text-white font-bold rounded-full transition-all duration-150 shrink-0 shadow-sm shadow-sky-600/30 flex items-center gap-1 cursor-pointer"
      >
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="1 4 1 10 7 10"></polyline>
          <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"></path>
        </svg>
        <span>{actionText}</span>
      </button>
    {/if}
  </div>
{/if}

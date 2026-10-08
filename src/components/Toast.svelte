<script lang="ts">
  export let message = '';
  export let visible = false;
</script>

{#if visible}
  <div class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-2.5 bg-white/95 backdrop-blur-md text-slate-800 text-xs font-semibold px-4 py-2.5 rounded-full border border-slate-200/90 shadow-xl shadow-slate-300/40 animate-bounce-in max-w-[92vw] sm:max-w-lg select-none">
    <div class="w-4 h-4 rounded-full bg-emerald-100 text-emerald-600 flex items-center justify-center shrink-0">
      <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="20 6 9 17 4 12"></polyline>
      </svg>
    </div>
    <span class="tracking-wide text-slate-800 truncate">{message}</span>
  </div>
{/if}

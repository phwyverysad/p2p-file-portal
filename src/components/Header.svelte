<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import type { ServerStatus } from '../lib/api';

  export let lang: Language = 'th';
  export let status: ServerStatus | null = null;
  export let onRetryTunnel: () => void = () => {};

  $: t = I18N[lang];
  $: tunnel = status?.tunnel;
  $: isOnline = tunnel?.status === 'online';
  $: isConnecting = tunnel?.status === 'starting';
  $: isError = tunnel?.status === 'error';
</script>

<header class="w-full flex items-center justify-between mb-6">
  <div class="flex items-center gap-3">
    <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-sky-500 to-sky-700 flex items-center justify-center shadow-md shadow-sky-500/20 text-white font-bold text-lg">
      <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"></path>
        <polyline points="16 6 12 2 8 6"></polyline>
        <line x1="12" y1="2" x2="12" y2="15"></line>
      </svg>
    </div>
    <div>
      <h1 class="text-xl font-bold tracking-tight text-slate-900 leading-none">{t.appTitle}</h1>
      <p class="text-xs text-slate-500 mt-1">{t.headerShare}</p>
    </div>
  </div>

  <div class="flex items-center gap-3">
    <!-- Tunnel status indicator -->
    <div class="status-indicator {isConnecting ? '!bg-amber-50 !border-amber-200 !text-amber-700' : isError ? '!bg-rose-50 !border-rose-200 !text-rose-700' : ''}">
      <span class="status-dot {isConnecting ? '!bg-amber-500' : isError ? '!bg-rose-500' : ''}"></span>
      <span>
        {#if isOnline}
          {t.statusPublic}
        {:else if isConnecting}
          {t.statusConnecting}
        {:else if isError}
          {t.statusError}
        {:else}
          {t.statusLocal}
        {/if}
      </span>
    </div>

    {#if isError || !isOnline}
      <button 
        type="button" 
        class="text-xs text-sky-600 hover:text-sky-800 font-medium underline cursor-pointer"
        on:click={onRetryTunnel}
        title="Retry connection"
      >
        Retry
      </button>
    {/if}
  </div>
</header>

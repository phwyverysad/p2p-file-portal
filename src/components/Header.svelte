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
    <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-sky-50 to-blue-50 border border-sky-100 flex items-center justify-center shadow-md shadow-sky-500/10 p-1 shrink-0 transition-transform hover:scale-105">
      <img src="upload.png" alt="File Sharer" class="w-full h-full object-contain filter drop-shadow-sm select-none" />
    </div>
    <div>
      <h1 class="text-xl font-bold tracking-tight text-slate-900 leading-none">{t.appTitle}</h1>
      <p class="text-xs text-slate-500 mt-1">{t.headerShare}</p>
    </div>
  </div>

  <div class="flex items-center gap-3">
    <!-- Tunnel status indicator -->
    <div class="status-indicator {isOnline ? '' : isConnecting ? '!bg-emerald-50/90 !border-emerald-200 !text-emerald-700' : isError ? '!bg-rose-50 !border-rose-200 !text-rose-700' : ''}">
      <span class="status-dot {isOnline ? '' : isConnecting ? '!bg-emerald-500 animate-pulse' : isError ? '!bg-rose-500' : ''}"></span>
      <span class="flex items-center gap-1.5">
        {#if isOnline}
          {t.statusPublic}
        {:else if isConnecting}
          <span>{t.statusConnecting}</span>
          <svg class="animate-spin w-3 h-3 text-emerald-600 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
          </svg>
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

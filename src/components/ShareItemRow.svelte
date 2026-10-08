<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import { 
    type ShareItem, 
    formatBytes, 
    getFileCategoryIcon,
    copyToClipboard,
    shortenShareUrl,
    showInFolder
  } from '../lib/api';

  export let item: ShareItem;
  export let lang: Language = 'th';
  export let onEdit: (item: ShareItem) => void;
  export let onDelete: (item: ShareItem) => void;
  export let onShowToast: (msg: string) => void;

  $: t = I18N[lang];
  $: iconType = getFileCategoryIcon(item.originalName);

  let isShortening = false;
  let copiedPublic = false;
  let copiedLan = false;

  async function handleCopyPublic() {
    const url = item.shortUrl || item.publicUrl;
    await copyToClipboard(url);
    copiedPublic = true;
    onShowToast(t.toastCopiedPublic);
    setTimeout(() => copiedPublic = false, 2000);
  }

  async function handleCopyLan() {
    await copyToClipboard(item.localUrl);
    copiedLan = true;
    onShowToast(t.toastCopiedLan);
    setTimeout(() => copiedLan = false, 2000);
  }

  async function handleShorten() {
    if (item.shortUrl) {
      await handleCopyPublic();
      return;
    }
    isShortening = true;
    try {
      const short = await shortenShareUrl(item.id);
      item.shortUrl = short;
      await copyToClipboard(short);
      onShowToast(t.toastShortened);
    } catch (err: any) {
      console.error(err);
    } finally {
      isShortening = false;
    }
  }

  async function handleShowInFolder() {
    await showInFolder(item.filePath);
  }
</script>

<div class="p-3.5 bg-white border border-slate-200/90 rounded-xl hover:border-slate-300 transition-all hover:shadow-sm flex flex-col gap-2.5">
  <div class="flex items-start justify-between gap-3">
    <!-- Icon & Title -->
    <div class="flex items-start gap-3 min-w-0">
      <div class="w-10 h-10 rounded-lg bg-sky-50 border border-sky-100 text-sky-600 flex items-center justify-center shrink-0 mt-0.5">
        {#if iconType === 'video'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="23 7 16 12 23 17 23 7"></polygon><rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect></svg>
        {:else if iconType === 'audio'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg>
        {:else if iconType === 'archive'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 8v13H3V8"></path><path d="M1 3h22v5H1z"></path><path d="M10 12h4"></path></svg>
        {:else if iconType === 'document'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line></svg>
        {:else if iconType === 'executable'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="2"></rect><rect x="9" y="9" width="6" height="6"></rect></svg>
        {:else if iconType === 'image'}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>
        {:else}
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path><polyline points="13 2 13 9 20 9"></polyline></svg>
        {/if}
      </div>

      <div class="min-w-0">
        <h3 class="text-sm font-bold text-slate-900 truncate leading-tight" title={item.title}>{item.title}</h3>
        <p class="text-xs text-slate-500 truncate mt-0.5" title={item.originalName}>{item.originalName}</p>
        
        <!-- Badges -->
        <div class="flex items-center gap-2 mt-1.5 flex-wrap">
          <span class="badge">{formatBytes(item.size)}</span>
          <span class="badge text-emerald-700 bg-emerald-50 border-emerald-200">
            {t.downloadsCount.replace('{n}', item.downloads.toString())}
          </span>
          <span class="badge text-slate-600">
            {t.viewsCount.replace('{n}', item.views.toString())}
          </span>
        </div>
      </div>
    </div>

    <!-- Quick Edit & Delete -->
    <div class="flex items-center gap-1 shrink-0">
      <button 
        type="button" 
        on:click={() => onEdit(item)}
        class="p-1.5 text-slate-400 hover:text-sky-600 rounded-md hover:bg-slate-100 transition-colors"
        title={t.btnEdit}
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
        </svg>
      </button>
      <button 
        type="button" 
        on:click={() => onDelete(item)}
        class="p-1.5 text-slate-400 hover:text-rose-600 rounded-md hover:bg-rose-50 transition-colors"
        title={t.btnDelete}
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
      </button>
    </div>
  </div>

  <!-- Action Bar -->
  <div class="flex items-center gap-1.5 pt-2 border-t border-slate-100 flex-wrap">
    <button 
      type="button" 
      on:click={handleCopyPublic}
      class="btn-primary !py-1.5 !px-2.5 !text-xs {copiedPublic ? '!bg-emerald-600' : ''}"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
      <span>{copiedPublic ? t.btnCopied : (item.shortUrl ? t.btnCopiedPublic : t.btnCopyPublic)}</span>
    </button>

    <button 
      type="button" 
      on:click={handleCopyLan}
      class="btn-secondary !py-1.5 !px-2.5 !text-xs {copiedLan ? '!text-emerald-700 !bg-emerald-50' : ''}"
      title="Copy direct LAN IP URL for local network"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2"></rect><rect x="2" y="14" width="20" height="8" rx="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>
      <span>{copiedLan ? t.btnCopied : t.btnCopyLan}</span>
    </button>

    <button 
      type="button" 
      on:click={handleShorten}
      disabled={isShortening}
      class="btn-secondary !py-1.5 !px-2.5 !text-xs text-sky-700 border-sky-200 bg-sky-50/50 hover:bg-sky-100/50"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path></svg>
      <span>{isShortening ? t.loading : item.shortUrl ? t.btnShortened : t.btnShorten}</span>
    </button>

    <button 
      type="button" 
      on:click={handleShowInFolder}
      class="btn-secondary !py-1.5 !px-2.5 !text-xs ml-auto"
      title="Show file location in Windows Explorer"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
      <span>{t.btnOpenFolder}</span>
    </button>
  </div>
</div>

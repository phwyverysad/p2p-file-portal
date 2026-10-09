<script lang="ts">
  import { onMount } from 'svelte';
  import { I18N, type Language } from '../lib/i18n';
  import { 
    type HistoryItem, 
    formatBytes, 
    getFileCategoryIcon,
    listHistory, 
    deleteHistoryItems, 
    clearHistory,
    restoreFromHistory
  } from '../lib/api';

  export let isOpen = false;
  export let lang: Language = 'th';
  export let onClose: () => void;
  export let onRestore: (item: HistoryItem) => Promise<void>;
  export let onShowToast: (msg: string) => void;
  export let onHistoryCountChanged: ((count: number) => void) | undefined = undefined;

  $: t = I18N[lang];

  let items: (HistoryItem & { _thumbFailed?: boolean })[] = [];
  let loading = false;
  let searchQuery = '';
  let isSelectMode = false;
  let selectedIds = new Set<string>();
  let isActionRunning = false;

  $: if (isOpen) {
    loadData();
  } else {
    // Reset selection state when closing
    selectedIds.clear();
    isSelectMode = false;
    searchQuery = '';
  }

  async function loadData() {
    loading = true;
    try {
      const historyList = await listHistory();
      items = historyList.map(h => ({ ...h, _thumbFailed: false }));
      if (onHistoryCountChanged) {
        onHistoryCountChanged(items.length);
      }
    } catch (err) {
      console.error('Failed to load history:', err);
    } finally {
      loading = false;
    }
  }

  $: filteredItems = items.filter(item => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase().trim();
    return (
      (item.title && item.title.toLowerCase().includes(q)) ||
      (item.originalName && item.originalName.toLowerCase().includes(q)) ||
      (item.filePath && item.filePath.toLowerCase().includes(q))
    );
  });

  $: allFilteredSelected = filteredItems.length > 0 && filteredItems.every(i => selectedIds.has(i.id));

  function toggleSelectMode() {
    isSelectMode = !isSelectMode;
    if (!isSelectMode) {
      selectedIds.clear();
      selectedIds = selectedIds;
    }
  }

  function toggleItemSelection(id: string) {
    if (selectedIds.has(id)) {
      selectedIds.delete(id);
    } else {
      selectedIds.add(id);
    }
    selectedIds = selectedIds;
  }

  function toggleSelectAll() {
    if (allFilteredSelected) {
      for (const item of filteredItems) {
        selectedIds.delete(item.id);
      }
    } else {
      for (const item of filteredItems) {
        selectedIds.add(item.id);
      }
    }
    selectedIds = selectedIds;
  }

  async function handleDeleteSingle(id: string) {
    isActionRunning = true;
    try {
      await deleteHistoryItems([id]);
      items = items.filter(i => i.id !== id);
      selectedIds.delete(id);
      selectedIds = selectedIds;
      if (onHistoryCountChanged) onHistoryCountChanged(items.length);
      onShowToast(t.toastHistoryDeleted);
    } catch (err) {
      console.error(err);
    } finally {
      isActionRunning = false;
    }
  }

  async function handleDeleteSelected() {
    if (selectedIds.size === 0) return;
    const ids = Array.from(selectedIds);
    isActionRunning = true;
    try {
      await deleteHistoryItems(ids);
      items = items.filter(i => !selectedIds.has(i.id));
      selectedIds.clear();
      selectedIds = selectedIds;
      if (onHistoryCountChanged) onHistoryCountChanged(items.length);
      onShowToast(t.toastHistoryDeleted);
    } catch (err) {
      console.error(err);
    } finally {
      isActionRunning = false;
    }
  }

  async function handleClearAll() {
    if (!confirm(t.confirmClearHistory)) return;
    isActionRunning = true;
    try {
      await clearHistory();
      items = [];
      selectedIds.clear();
      selectedIds = selectedIds;
      if (onHistoryCountChanged) onHistoryCountChanged(0);
      onShowToast(t.toastHistoryDeleted);
    } catch (err) {
      console.error(err);
    } finally {
      isActionRunning = false;
    }
  }

  async function handleRestoreClick(item: HistoryItem) {
    if (!item.fileExists) {
      alert(t.tipFileDeletedCannotRestore);
      return;
    }
    isActionRunning = true;
    try {
      await onRestore(item);
      // Mark as active in local list
      items = items.map(i => i.id === item.id ? { ...i, isActive: true } : i);
      onShowToast(t.toastHistoryRestored);
    } catch (err: any) {
      alert(err.message || 'Failed to restore file');
    } finally {
      isActionRunning = false;
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function formatDate(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString(lang === 'th' ? 'th-TH' : 'en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return iso;
    }
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
  <div 
    class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-3 sm:p-5 animate-fade-in"
    on:click={handleBackdropClick}
    role="dialog"
    tabindex="-1"
  >
    <div class="bg-white rounded-2xl border border-slate-200/90 shadow-2xl max-w-2xl w-full flex flex-col max-h-[88vh] overflow-hidden animate-scale-up">
      <!-- HEADER -->
      <div class="px-5 py-3.5 border-b border-slate-100 flex items-center justify-between shrink-0 bg-slate-50/70">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-sky-50 text-sky-600 border border-sky-100 flex items-center justify-center">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
              <path d="M3 3v5h5"/>
              <path d="M12 7v5l4 2"/>
            </svg>
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="font-bold text-slate-900 text-sm">{t.historyTitle}</h2>
              <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-slate-200/80 text-slate-600">
                {items.length}
              </span>
            </div>
            <p class="text-[11px] text-slate-400">บันทึกประวัติการแชร์ทั้งหมด สามารถกู้คืนได้ตลอดเวลา</p>
          </div>
        </div>

        <button 
          type="button" 
          on:click={onClose} 
          class="text-slate-400 hover:text-slate-600 p-1.5 rounded-lg hover:bg-slate-200/60 transition-colors"
          title={t.btnClose}
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>

      <!-- SEARCH & TOOLBAR -->
      <div class="p-3 border-b border-slate-100 bg-white flex flex-wrap items-center justify-between gap-2.5 shrink-0">
        <!-- Search input -->
        <div class="relative flex-1 min-w-[200px]">
          <svg class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
          <input 
            type="text" 
            bind:value={searchQuery} 
            placeholder={t.searchHistoryPlaceholder} 
            class="w-full pl-9 pr-8 py-1.5 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:border-sky-500 focus:bg-white text-slate-800 transition-colors"
          />
          {#if searchQuery}
            <button 
              type="button" 
              on:click={() => searchQuery = ''}
              class="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 p-0.5"
            >
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
            </button>
          {/if}
        </div>

        <!-- Action buttons -->
        <div class="flex items-center gap-1.5 shrink-0">
          {#if items.length > 0}
            <!-- Toggle Select Mode -->
            <button 
              type="button" 
              on:click={toggleSelectMode}
              class="px-2.5 py-1.5 text-xs font-semibold rounded-lg border transition-all {isSelectMode ? 'bg-sky-50 border-sky-300 text-sky-700' : 'bg-slate-50 border-slate-200 text-slate-600 hover:bg-slate-100'}"
            >
              {isSelectMode ? t.btnCancelSelect : t.btnSelectMode}
            </button>

            <!-- Select All (when in select mode) -->
            {#if isSelectMode}
              <button 
                type="button" 
                on:click={toggleSelectAll}
                class="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-slate-200 bg-slate-50 hover:bg-slate-100 text-slate-600 transition-colors"
              >
                {allFilteredSelected ? t.btnDeselectAll : t.btnSelectAll}
              </button>

              <!-- Delete Selected button -->
              {#if selectedIds.size > 0}
                <button 
                  type="button" 
                  on:click={handleDeleteSelected}
                  disabled={isActionRunning}
                  class="px-3 py-1.5 text-xs font-semibold rounded-lg bg-rose-600 hover:bg-rose-700 text-white transition-all shadow-xs flex items-center gap-1 disabled:opacity-50"
                >
                  <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                  <span>{t.btnDeleteSelected} ({selectedIds.size})</span>
                </button>
              {/if}
            {/if}
          {/if}
        </div>
      </div>

      <!-- LIST CONTENT -->
      <div class="p-3 sm:p-4 overflow-y-auto flex-1 space-y-2">
        {#if loading}
          <div class="py-12 flex flex-col items-center justify-center text-slate-400 gap-2">
            <svg class="w-6 h-6 animate-spin text-sky-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"></path></svg>
            <span class="text-xs">{t.loading}</span>
          </div>
        {:else if filteredItems.length === 0}
          <div class="py-14 text-center flex flex-col items-center justify-center">
            <div class="w-12 h-12 rounded-full bg-slate-100 text-slate-400 flex items-center justify-center mb-2.5">
              <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                <path d="M3 3v5h5"/>
                <path d="M12 7v5l4 2"/>
              </svg>
            </div>
            <div class="text-xs font-bold text-slate-700">{searchQuery ? 'ไม่พบรายการที่ค้นหา' : t.historyEmpty}</div>
            <div class="text-[11px] text-slate-400 mt-0.5">{searchQuery ? 'ลองค้นหาด้วยคำอื่น' : t.historyEmptySub}</div>
          </div>
        {:else}
          {#each filteredItems as item (item.id)}
            {@const iconType = getFileCategoryIcon(item.originalName, item.isDirectory)}
            {@const isSelected = selectedIds.has(item.id)}
            <div 
              class="p-2.5 sm:p-3 rounded-xl border transition-all duration-200 flex items-center gap-3 {isSelected ? 'bg-sky-50/60 border-sky-300 ring-1 ring-sky-300' : 'bg-white border-slate-200 hover:border-slate-300 hover:shadow-xs'}"
            >
              <!-- Checkbox (in select mode or on click) -->
              {#if isSelectMode}
                <label class="cursor-pointer select-none p-1 shrink-0">
                  <input 
                    type="checkbox" 
                    checked={isSelected} 
                    on:change={() => toggleItemSelection(item.id)}
                    class="w-4 h-4 rounded border-slate-300 text-sky-600 focus:ring-sky-500 cursor-pointer"
                  />
                </label>
              {/if}

              <!-- Icon / Thumbnail -->
              <div class="w-10 h-10 rounded-lg {item.fileExists ? (item.isDirectory ? 'bg-amber-50 border-amber-200 text-amber-600' : 'bg-sky-50 border-sky-100 text-sky-600') : 'bg-rose-50 border-rose-200 text-rose-500'} border flex items-center justify-center shrink-0 p-1 overflow-hidden relative">
                {#if item.thumbnailUrl && !item._thumbFailed}
                  <img 
                    src={item.thumbnailUrl} 
                    alt="" 
                    class="w-full h-full object-contain" 
                    on:error={() => item._thumbFailed = true}
                  />
                {:else if item.isDirectory || iconType === 'folder'}
                  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                  </svg>
                {:else if iconType === 'video'}
                  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="23 7 16 12 23 17 23 7"></polygon><rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect></svg>
                {:else if iconType === 'audio'}
                  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"></path><circle cx="6" cy="18" r="3"></circle><circle cx="18" cy="16" r="3"></circle></svg>
                {:else if iconType === 'archive'}
                  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 8v13H3V8"></path><path d="M1 3h22v5H1z"></path><path d="M10 12h4"></path></svg>
                {:else}
                  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>
                {/if}
              </div>

              <!-- Main Info -->
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                  {#if !item.fileExists}
                    <!-- STRIKETHROUGH IN RED AS REQUESTED! -->
                    <span class="text-xs font-bold text-rose-500 line-through truncate max-w-[280px] sm:max-w-md" title={item.title}>
                      {item.title}
                    </span>
                    <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-rose-50 text-rose-600 border border-rose-200">
                      {t.statusFileDeleted}
                    </span>
                  {:else}
                    <span class="text-xs font-bold text-slate-900 truncate max-w-[280px] sm:max-w-md" title={item.title}>
                      {item.title}
                    </span>
                    {#if item.isActive}
                      <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-emerald-50 text-emerald-600 border border-emerald-200 flex items-center gap-1">
                        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                        <span>{t.statusActiveSharing}</span>
                      </span>
                    {/if}
                  {/if}
                </div>

                <div class="text-[11px] text-slate-500 mt-0.5 flex items-center gap-2 flex-wrap">
                  <span>{formatBytes(item.size)}</span>
                  <span>•</span>
                  <span class="truncate max-w-[140px] sm:max-w-xs">{item.originalName}</span>
                  <span>•</span>
                  <span>{formatDate(item.lastSharedAt || item.createdAt)}</span>
                </div>

                {#if item.description}
                  <div class="text-[11px] text-slate-600 truncate mt-0.5" title={item.description}>
                    {item.description}
                  </div>
                {/if}

                <div class="text-[10px] text-slate-400 font-mono truncate mt-0.5" title={item.filePath}>
                  {item.filePath}
                </div>
              </div>

              <!-- Action buttons per row -->
              <div class="flex items-center gap-1.5 shrink-0">
                <!-- RESTORE BUTTON -->
                {#if !item.fileExists}
                  <button 
                    type="button" 
                    disabled 
                    class="px-2.5 py-1 text-xs font-medium rounded-lg bg-slate-100 text-slate-400 border border-slate-200 cursor-not-allowed opacity-70 flex items-center gap-1"
                    title={t.tipFileDeletedCannotRestore}
                  >
                    <span>{t.btnRestore}</span>
                  </button>
                {:else if item.isActive}
                  <div class="px-2 py-1 text-xs font-semibold text-emerald-600 flex items-center gap-1">
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>
                    <span>{t.statusActiveSharing}</span>
                  </div>
                {:else}
                  <!-- Active restore button -->
                  <button 
                    type="button" 
                    on:click={() => handleRestoreClick(item)}
                    disabled={isActionRunning}
                    class="px-2.5 py-1 text-xs font-bold rounded-lg bg-sky-50 hover:bg-sky-100 active:scale-95 text-sky-700 border border-sky-200 flex items-center gap-1 transition-all cursor-pointer shadow-2xs"
                    title="กู้คืนรายการนี้เข้าสู่การแชร์อีกครั้ง"
                  >
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
                      <path d="M21 3v5h-5"/>
                      <path d="M12 7v5l4 2"/>
                    </svg>
                    <span>{t.btnRestore}</span>
                  </button>
                {/if}

                <!-- Single Delete Button -->
                <button 
                  type="button" 
                  on:click={() => handleDeleteSingle(item.id)}
                  disabled={isActionRunning}
                  class="p-1.5 text-slate-400 hover:text-rose-600 hover:bg-rose-50 rounded-lg transition-colors cursor-pointer"
                  title="ลบออกจากประวัติ"
                >
                  <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>

      <!-- FOOTER -->
      <div class="px-5 py-3 border-t border-slate-100 bg-slate-50/80 flex items-center justify-between shrink-0">
        <div>
          {#if items.length > 0}
            <button 
              type="button" 
              on:click={handleClearAll}
              disabled={isActionRunning}
              class="text-xs font-semibold text-rose-600 hover:text-rose-700 hover:underline cursor-pointer"
            >
              {t.btnClearAllHistory}
            </button>
          {/if}
        </div>

        <button 
          type="button" 
          on:click={onClose} 
          class="btn-secondary !text-xs !py-1.5 !px-4"
        >
          {t.btnClose}
        </button>
      </div>
    </div>
  </div>
{/if}

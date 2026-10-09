<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import Header from './components/Header.svelte';
  import UploadCard from './components/UploadCard.svelte';
  import SharesList from './components/SharesList.svelte';
  import SettingsModal from './components/SettingsModal.svelte';
  import EditModal from './components/EditModal.svelte';
  import DeleteModal from './components/DeleteModal.svelte';
  import Toast from './components/Toast.svelte';
  import HistoryModal from './components/HistoryModal.svelte';

  import { 
    type ShareItem, 
    type HistoryItem,
    type AppSettings, 
    type ServerStatus,
    listShares, 
    listHistory,
    addLocalShare, 
    updateShare, 
    revokeShare, 
    restoreShare,
    restoreFromHistory,
    getStatus, 
    getAppSettings, 
    saveAppSettings,
    retryTunnel,
    copyToClipboard,
    getPendingShares,
    isTauri
  } from './lib/api';
  import { I18N } from './lib/i18n';

  let shares: ShareItem[] = [];
  let settings: AppSettings = {
    language: 'th',
    minimizeToTray: true,
    autoStart: false,
    autoCopy: true,
    contextMenu: true,
  };
  let serverStatus: ServerStatus | null = null;
  let isProcessing = false;
  let processingFileName = '';
  let isGlobalDragOver = false;

  // Modals state
  let isSettingsOpen = false;
  let isHistoryOpen = false;
  let isEditOpen = false;
  let isDeleteOpen = false;
  let historyCount = 0;
  let activeEditItem: ShareItem | null = null;
  let activeDeleteItem: ShareItem | null = null;

  // Toast
  let toastMessage = '';
  let toastVisible = false;
  let toastActionText: string | undefined = undefined;
  let toastOnAction: (() => void) | undefined = undefined;
  let toastTimer: any = null;

  function showToast(msg: string, actionText?: string, onAction?: () => void, duration = 2800) {
    toastMessage = msg;
    toastActionText = actionText;
    toastOnAction = onAction;
    toastVisible = true;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toastVisible = false;
      toastActionText = undefined;
      toastOnAction = undefined;
    }, duration);
  }

  async function loadData() {
    try {
      const [sh, st, set] = await Promise.all([
        listShares(),
        getStatus(),
        getAppSettings(),
        listHistory().catch(() => [] as HistoryItem[]),
      ]);
      shares = sh;
      serverStatus = st;
      settings = set;
      historyCount = Array.isArray(hist) ? hist.length : 0;
    } catch (err) {
      console.error('Initial load error:', err);
    }
  }

  async function handleAddFile(path: string, title?: string) {
    processingFileName = path.split(/[/\\]/).pop() || path;
    isProcessing = true;
    try {
      const newItem = await addLocalShare(path, title);
      shares = [newItem, ...shares.filter(s => s.id !== newItem.id)];
      historyCount += 1;
      if (settings.autoCopy) {
        const link = newItem.shortUrl || newItem.publicUrl;
        await copyToClipboard(link);
        showToast(`${I18N[settings.language].toastCopiedLink}${link}`);
      } else {
        showToast(I18N[settings.language].toastFileAdded);
      }
    } catch (err: any) {
      console.error(err);
      alert(err.message || 'Failed to add share');
    } finally {
      isProcessing = false;
      processingFileName = '';
    }
  }

  function handleExternalShare(newItem: ShareItem) {
    if (!newItem || !newItem.id) return;
    shares = [newItem, ...shares.filter(s => s.id !== newItem.id)];
    historyCount += 1;
    if (settings.autoCopy) {
      const link = newItem.shortUrl || newItem.publicUrl;
      copyToClipboard(link);
      showToast(`${I18N[settings.language].toastCopiedLink}${link}`);
    } else {
      showToast(I18N[settings.language].toastFileAdded);
    }
  }

  async function handleRestoreFromHistory(item: HistoryItem) {
    const restored = await restoreFromHistory(item.id);
    shares = [restored, ...shares.filter(s => s.id !== restored.id)];
  }

  function handleTunnelStatusUpdate(payload: any) {
    if (!payload) return;
    if (serverStatus) {
      serverStatus = {
        ...serverStatus,
        tunnel: {
          ...serverStatus.tunnel,
          status: payload.status,
          publicUrl: payload.publicUrl || serverStatus.tunnel.publicUrl,
        }
      };
    }
    // Instantly refresh shares so all rows reflect live public URLs
    listShares().then(sh => shares = sh).catch(() => {});
  }

  // Register on window for instant zero-latency eval from backend
  if (typeof window !== 'undefined') {
    (window as any).__onShareAdded = handleExternalShare;
    (window as any).__onTunnelStatus = handleTunnelStatusUpdate;
  }

  function handleOpenEdit(item: ShareItem) {
    activeEditItem = item;
    isEditOpen = true;
  }

  async function handleSaveEdit(id: string, title: string, desc: string, directDownload: boolean) {
    try {
      const updated = await updateShare(id, title, desc, directDownload);
      shares = shares.map(s => s.id === id ? updated : s);
      showToast(I18N[settings.language].toastUpdated);
    } catch (err) {
      console.error(err);
    }
  }

  function handleOpenDelete(item: ShareItem) {
    if (settings.confirmDelete === false) {
      handleConfirmDelete(item.id, false);
      return;
    }
    activeDeleteItem = item;
    isDeleteOpen = true;
  }

  async function handleConfirmDelete(id: string, dontShowAgain = false) {
    try {
      if (dontShowAgain) {
        settings.confirmDelete = false;
        await saveAppSettings(settings);
      }
      const itemToDelete = shares.find(s => s.id === id);
      await revokeShare(id);
      shares = shares.filter(s => s.id !== id);

      if (itemToDelete) {
        showToast(
          I18N[settings.language].toastRevoked,
          I18N[settings.language].btnUndo,
          async () => {
            try {
              const restored = await restoreShare(itemToDelete);
              shares = [restored, ...shares.filter(s => s.id !== restored.id)];
              toastVisible = false;
              showToast(I18N[settings.language].toastRestored);
            } catch (err) {
              console.error('Failed to undo delete:', err);
            }
          },
          6000
        );
      } else {
        showToast(I18N[settings.language].toastRevoked);
      }
    } catch (err) {
      console.error(err);
    }
  }

  async function handleSaveSettings(newSettings: AppSettings) {
    settings = await saveAppSettings(newSettings);
  }

  async function handleRetryTunnel() {
    await retryTunnel();
    setTimeout(async () => {
      serverStatus = await getStatus();
    }, 1500);
  }

  let unlistenShare: (() => void) | null = null;
  let unlistenReq: (() => void) | null = null;
  let unlistenDragDrop: (() => void) | null = null;
  let unlistenTunnel: (() => void) | null = null;

  onMount(async () => {
    // 0. Disable right-click & F12 developer tools
    const handleContextMenu = (e: MouseEvent) => e.preventDefault();
    const handleKeyDown = (e: KeyboardEvent) => {
      if (
        e.key === 'F12' ||
        (e.ctrlKey && e.shiftKey && (e.key === 'I' || e.key === 'i' || e.key === 'J' || e.key === 'j' || e.key === 'C' || e.key === 'c')) ||
        (e.ctrlKey && (e.key === 'u' || e.key === 'U'))
      ) {
        e.preventDefault();
      }
    };
    window.addEventListener('contextmenu', handleContextMenu);
    window.addEventListener('keydown', handleKeyDown);

    // Global drag-and-drop file detection
    let dragCounter = 0;
    const handleWindowDragEnter = (e: DragEvent) => {
      e.preventDefault();
      dragCounter++;
      isGlobalDragOver = true;
    };
    const handleWindowDragLeave = (e: DragEvent) => {
      e.preventDefault();
      dragCounter--;
      if (dragCounter <= 0) {
        dragCounter = 0;
        isGlobalDragOver = false;
      }
    };
    const handleWindowDragOver = (e: DragEvent) => {
      e.preventDefault();
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = 'copy';
      }
      isGlobalDragOver = true;
    };
    const handleWindowDrop = async (e: DragEvent) => {
      e.preventDefault();
      dragCounter = 0;
      isGlobalDragOver = false;
      if (e.dataTransfer && e.dataTransfer.files.length > 0) {
        for (let i = 0; i < e.dataTransfer.files.length; i++) {
          const file = e.dataTransfer.files[i] as any;
          const filePath = file.path || file.name;
          if (filePath) {
            await handleAddFile(filePath);
          }
        }
      }
    };

    window.addEventListener('dragenter', handleWindowDragEnter);
    window.addEventListener('dragleave', handleWindowDragLeave);
    window.addEventListener('dragover', handleWindowDragOver);
    window.addEventListener('drop', handleWindowDrop);

    // Tauri native webview drag-drop listener
    if (isTauri) {
      try {
        const { getCurrentWebview } = await import('@tauri-apps/api/webview');
        unlistenDragDrop = await getCurrentWebview().onDragDropEvent(async (event) => {
          if (event.payload.type === 'enter' || event.payload.type === 'over') {
            isGlobalDragOver = true;
          } else if (event.payload.type === 'leave') {
            isGlobalDragOver = false;
          } else if (event.payload.type === 'drop') {
            isGlobalDragOver = false;
            const paths = (event.payload as any).paths;
            if (Array.isArray(paths) && paths.length > 0) {
              for (const p of paths) {
                await handleAddFile(p);
              }
            }
          }
        });
      } catch (e) {
        console.warn('Webview drag drop error:', e);
      }
    }

    await loadData();

    // 1. Process any pending files from right-click context menu (initial launch)
    try {
      const initialPending = await getPendingShares();
      for (const filePath of initialPending) {
        await handleAddFile(filePath);
      }
    } catch (e) {
      console.warn('Pending shares fetch error:', e);
    }

    // 2. Listen to subsequent right-click share requests (while app is running)
    try {
      unlistenShare = await listen<ShareItem>('share-file-added', (event) => {
        if (event.payload) {
          handleExternalShare(event.payload);
        }
      });
    } catch (e) {
      console.warn('Failed to attach share-file-added listener:', e);
    }

    try {
      unlistenReq = await listen<string>('share-file-requested', async (event) => {
        if (event.payload && typeof event.payload === 'string') {
          await handleAddFile(event.payload);
        }
      });
    } catch (e) {
      console.warn('Failed to attach share-file-requested listener:', e);
    }

    try {
      unlistenTunnel = await listen<any>('tunnel-status-changed', (event) => {
        if (event.payload) {
          handleTunnelStatusUpdate(event.payload);
        }
      });
    } catch (e) {
      console.warn('Failed to attach tunnel-status-changed listener:', e);
    }

    // 3. Adaptive Fast Polling: checks every 350ms during startup/connecting, then 3000ms once online
    let pollTimer: any = null;
    let isDisposed = false;
    const schedulePoll = () => {
      if (isDisposed) return;
      const isOnline = serverStatus?.tunnel?.status === 'online';
      const delay = isOnline ? 3000 : 350;
      pollTimer = setTimeout(async () => {
        if (isDisposed) return;
        try {
          serverStatus = await getStatus();
          shares = await listShares();
        } catch (e) {}
        schedulePoll();
      }, delay);
    };
    schedulePoll();

    return () => {
      isDisposed = true;
      if (pollTimer) clearTimeout(pollTimer);
      window.removeEventListener('contextmenu', handleContextMenu);
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('dragenter', handleWindowDragEnter);
      window.removeEventListener('dragleave', handleWindowDragLeave);
      window.removeEventListener('dragover', handleWindowDragOver);
      window.removeEventListener('drop', handleWindowDrop);
      if (unlistenShare) unlistenShare();
      if (unlistenReq) unlistenReq();
      if (unlistenDragDrop) unlistenDragDrop();
      if (unlistenTunnel) unlistenTunnel();
    };
  });

  onDestroy(() => {
    if (unlistenShare) unlistenShare();
    if (unlistenReq) unlistenReq();
    if (unlistenDragDrop) unlistenDragDrop();
    if (unlistenTunnel) unlistenTunnel();
  });
</script>

<main class="container mx-auto">
  <Header 
    lang={settings.language} 
    status={serverStatus} 
    onRetryTunnel={handleRetryTunnel} 
  />

  <UploadCard 
    lang={settings.language} 
    onAddFile={handleAddFile} 
    {isProcessing} 
  />

  <SharesList 
    {shares} 
    lang={settings.language} 
    onEdit={handleOpenEdit} 
    onDelete={handleOpenDelete} 
    onShowToast={showToast} 
    {isProcessing}
    {processingFileName}
  />
</main>

<!-- GLOBAL FROSTED-GLASS DROP OVERLAY -->
{#if isGlobalDragOver}
  <div class="fixed inset-0 z-50 flex flex-col items-center justify-center bg-sky-950/40 backdrop-blur-md border-4 border-dashed border-sky-400 m-3 rounded-2xl pointer-events-none animate-scale-up">
    <div class="w-20 h-20 rounded-full bg-white text-sky-600 flex items-center justify-center mb-4 shadow-xl animate-bounce-gentle">
      <svg class="w-10 h-10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
        <polyline points="17 8 12 3 7 8"></polyline>
        <line x1="12" y1="2" x2="12" y2="15"></line>
      </svg>
    </div>
    <h2 class="text-2xl font-bold text-white drop-shadow-md mb-1.5">{I18N[settings.language].dragDropOverlayTitle}</h2>
    <p class="text-sm font-medium text-sky-100 drop-shadow-sm">{I18N[settings.language].dragDropOverlaySubtitle}</p>
  </div>
{/if}

<!-- FLOATING CONTROLS (Bottom-Left Glass Dock) -->
<div class="floating-bottom-dock" role="toolbar" aria-label="Controls">
  <!-- Settings Button -->
  <button 
    type="button" 
    class="dock-btn group" 
    on:click={() => isSettingsOpen = true}
    title={I18N[settings.language].settingsTitle}
    aria-label={I18N[settings.language].settingsTitle}
  >
    <svg class="w-5 h-5 transition-transform duration-300 group-hover:rotate-45" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="3"></circle>
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
    </svg>
  </button>

  <div class="dock-divider"></div>

  <!-- History Button -->
  <button 
    type="button" 
    class="dock-btn group relative" 
    on:click={() => isHistoryOpen = true}
    title={I18N[settings.language].historyTitle}
    aria-label={I18N[settings.language].historyTitle}
  >
    <svg class="w-5 h-5 transition-transform duration-200 group-hover:scale-110" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
      <path d="M3 3v5h5"/>
      <path d="M12 7v5l4 2"/>
    </svg>
    {#if historyCount > 0}
      <span class="absolute -top-1 -right-1 min-w-[17px] h-[17px] px-1 bg-sky-600 text-white text-[9.5px] font-bold rounded-full flex items-center justify-center border-2 border-white shadow-2xs animate-scale-up">
        {historyCount > 99 ? '99+' : historyCount}
      </span>
    {/if}
  </button>
</div>

<!-- Modals -->
<SettingsModal 
  isOpen={isSettingsOpen} 
  {settings} 
  {serverStatus} 
  onClose={() => isSettingsOpen = false} 
  onSaveSettings={handleSaveSettings} 
/>

<HistoryModal 
  isOpen={isHistoryOpen} 
  lang={settings.language} 
  onClose={() => isHistoryOpen = false} 
  onRestore={handleRestoreFromHistory} 
  onShowToast={showToast} 
  onHistoryCountChanged={(c) => historyCount = c} 
/>

<EditModal 
  isOpen={isEditOpen} 
  item={activeEditItem} 
  lang={settings.language} 
  onClose={() => isEditOpen = false} 
  onSave={handleSaveEdit} 
/>

<DeleteModal 
  isOpen={isDeleteOpen} 
  item={activeDeleteItem} 
  lang={settings.language} 
  onClose={() => isDeleteOpen = false} 
  onConfirm={handleConfirmDelete} 
/>

<Toast 
  message={toastMessage} 
  visible={toastVisible} 
  actionText={toastActionText} 
  onAction={toastOnAction} 
/>

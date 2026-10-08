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

  import { 
    type ShareItem, 
    type AppSettings, 
    type ServerStatus,
    listShares, 
    addLocalShare, 
    updateShare, 
    revokeShare, 
    getStatus, 
    getAppSettings, 
    saveAppSettings,
    retryTunnel,
    copyToClipboard,
    getPendingShares
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

  // Modals state
  let isSettingsOpen = false;
  let isEditOpen = false;
  let isDeleteOpen = false;
  let activeEditItem: ShareItem | null = null;
  let activeDeleteItem: ShareItem | null = null;

  // Toast
  let toastMessage = '';
  let toastVisible = false;
  let toastTimer: any = null;

  function showToast(msg: string) {
    toastMessage = msg;
    toastVisible = true;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => toastVisible = false, 2800);
  }

  async function loadData() {
    try {
      const [sh, st, set] = await Promise.all([
        listShares(),
        getStatus(),
        getAppSettings(),
      ]);
      shares = sh;
      serverStatus = st;
      settings = set;
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
    if (settings.autoCopy) {
      const link = newItem.shortUrl || newItem.publicUrl;
      copyToClipboard(link);
      showToast(`${I18N[settings.language].toastCopiedLink}${link}`);
    } else {
      showToast(I18N[settings.language].toastFileAdded);
    }
  }

  // Register on window for instant zero-latency eval from backend
  if (typeof window !== 'undefined') {
    (window as any).__onShareAdded = handleExternalShare;
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
      await revokeShare(id);
      shares = shares.filter(s => s.id !== id);
      showToast(I18N[settings.language].toastRevoked);
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

  onMount(async () => {
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

    // 3. Poll status and shares every 3 seconds
    const interval = setInterval(async () => {
      try {
        serverStatus = await getStatus();
        shares = await listShares();
      } catch (e) {}
    }, 3000);

    return () => {
      clearInterval(interval);
      if (unlistenShare) unlistenShare();
      if (unlistenReq) unlistenReq();
    };
  });

  onDestroy(() => {
    if (unlistenShare) unlistenShare();
    if (unlistenReq) unlistenReq();
  });
</script>

<main class="container">
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

<!-- FLOATING GEAR BUTTON (Bottom-Left) -->
<button 
  type="button" 
  class="floating-settings-btn" 
  on:click={() => isSettingsOpen = true}
  title="Settings"
>
  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="12" cy="12" r="3"></circle>
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
  </svg>
</button>

<!-- Modals -->
<SettingsModal 
  isOpen={isSettingsOpen} 
  {settings} 
  {serverStatus} 
  onClose={() => isSettingsOpen = false} 
  onSaveSettings={handleSaveSettings} 
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

<Toast message={toastMessage} visible={toastVisible} />

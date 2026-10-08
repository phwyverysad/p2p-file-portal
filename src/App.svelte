<script lang="ts">
  import { onMount } from 'svelte';
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
    copyToClipboard
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
    toastTimer = setTimeout(() => toastVisible = false, 2500);
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
    isProcessing = true;
    try {
      const newItem = await addLocalShare(path, title);
      shares = [newItem, ...shares];
      showToast(I18N[settings.language].toastFileAdded);

      if (settings.autoCopy) {
        const link = newItem.shortUrl || newItem.publicUrl;
        await copyToClipboard(link);
      }
    } catch (err: any) {
      console.error(err);
      alert(err.message || 'Failed to add share');
    } finally {
      isProcessing = false;
    }
  }

  function handleOpenEdit(item: ShareItem) {
    activeEditItem = item;
    isEditOpen = true;
  }

  async function handleSaveEdit(id: string, title: string, desc: string) {
    try {
      const updated = await updateShare(id, title, desc);
      shares = shares.map(s => s.id === id ? updated : s);
      showToast(I18N[settings.language].toastUpdated);
    } catch (err) {
      console.error(err);
    }
  }

  function handleOpenDelete(item: ShareItem) {
    activeDeleteItem = item;
    isDeleteOpen = true;
  }

  async function handleConfirmDelete(id: string) {
    try {
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

  onMount(() => {
    loadData();

    // Poll status and shares every 3 seconds
    const interval = setInterval(async () => {
      try {
        serverStatus = await getStatus();
        shares = await listShares();
      } catch (e) {}
    }, 3000);

    // Listen to Tauri single instance argument events
    if (typeof window !== 'undefined' && (window as any).__TAURI__) {
      const { listen } = (window as any).__TAURI__.event;
      listen('share-file-requested', async (event: any) => {
        if (event.payload && typeof event.payload === 'string') {
          await handleAddFile(event.payload);
        }
      });
    }

    return () => clearInterval(interval);
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

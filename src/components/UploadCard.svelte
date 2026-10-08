<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import { pickFileDialog } from '../lib/api';

  export let lang: Language = 'th';
  export let onAddFile: (path: string, title?: string) => Promise<void>;
  export let isProcessing = false;

  $: t = I18N[lang];

  let localPathInput = '';
  let customTitleInput = '';
  let isDragOver = false;

  async function handleBrowse() {
    try {
      const selected = await pickFileDialog();
      if (selected) {
        localPathInput = selected;
      }
    } catch (err) {
      console.error(err);
    }
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
    if (e.dataTransfer && e.dataTransfer.files.length > 0) {
      // In web/webview, file.path is available on Electron/Tauri
      const file = e.dataTransfer.files[0] as any;
      const filePath = file.path || file.name;
      if (filePath) {
        await onAddFile(filePath, customTitleInput || undefined);
        localPathInput = '';
        customTitleInput = '';
      }
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!localPathInput.trim()) return;
    await onAddFile(localPathInput.trim(), customTitleInput.trim() || undefined);
    localPathInput = '';
    customTitleInput = '';
  }
</script>

<section class="card upload-card mb-5">
  <div class="card-top-bar flex items-center justify-between mb-4">
    <div class="card-caption flex items-center gap-2 font-bold text-slate-800 text-[15px]">
      <svg class="w-[18px] h-[18px] text-sky-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"></path>
        <polyline points="16 6 12 2 8 6"></polyline>
        <line x1="12" y1="2" x2="12" y2="15"></line>
      </svg>
      <span>{t.headerShare}</span>
    </div>
  </div>

  <!-- Drag & Drop Zone -->
  <div 
    class="dropzone {isDragOver ? 'dragover' : ''}" 
    role="button" 
    tabindex="0"
    on:click={handleBrowse}
    on:keydown={(e) => e.key === 'Enter' && handleBrowse()}
    on:dragover|preventDefault={() => isDragOver = true}
    on:dragleave|preventDefault={() => isDragOver = false}
    on:drop={handleDrop}
  >
    <div class="w-12 h-12 rounded-full bg-sky-50 text-sky-600 flex items-center justify-center mb-3">
      <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
        <polyline points="17 8 12 3 7 8"></polyline>
        <line x1="12" y1="2" x2="12" y2="15"></line>
      </svg>
    </div>
    <div class="text-[14px] font-semibold text-slate-800 mb-1">{t.dropzonePrompt}</div>
    <div class="text-[12px] text-slate-500">{t.dropzoneFormats}</div>
  </div>

  <!-- Divider -->
  <div class="relative my-4 text-center">
    <div class="absolute inset-0 flex items-center"><div class="w-full border-t border-slate-200"></div></div>
    <span class="relative bg-white px-3 text-xs text-slate-400 font-medium">{t.dividerPath}</span>
  </div>

  <!-- Form for manual path input -->
  <form on:submit={handleSubmit} class="space-y-3">
    <div class="flex gap-2">
      <div class="relative flex-1">
        <input 
          type="text" 
          bind:value={localPathInput}
          placeholder="C:\Users\Documents\MyFile.zip" 
          class="w-full pl-9 pr-3 py-2 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:border-sky-500 focus:bg-white text-slate-800 transition-colors font-mono"
          required
        />
        <svg class="w-4 h-4 text-slate-400 absolute left-3 top-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
      </div>
      <button 
        type="button" 
        on:click={handleBrowse} 
        class="px-3 py-2 text-xs font-semibold bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg border border-slate-200 transition-colors flex items-center gap-1.5"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <span>{t.btnBrowse}</span>
      </button>
      <button 
        type="submit" 
        disabled={isProcessing || !localPathInput.trim()} 
        class="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <span>{isProcessing ? t.loading : t.btnShare}</span>
      </button>
    </div>
  </form>
</section>

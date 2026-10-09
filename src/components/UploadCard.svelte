<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import { pickFileDialog, pickFolderDialog } from '../lib/api';

  export let lang: Language = 'th';
  export let onAddFile: (path: string, title?: string) => Promise<void>;
  export let isProcessing = false;

  $: t = I18N[lang];

  let localPathInput = '';
  let customTitleInput = '';
  let isDragOver = false;
  let showPickerModal = false;

  function handleKeyDown(e: KeyboardEvent) {
    if (showPickerModal) {
      if (e.key === 'Escape') showPickerModal = false;
      if (e.key === '1') { showPickerModal = false; handleBrowseFile(); }
      if (e.key === '2') { showPickerModal = false; handleBrowseFolder(); }
    }
  }

  async function handleBrowseFile() {
    try {
      const selected = await pickFileDialog();
      if (selected) {
        await onAddFile(selected, customTitleInput.trim() || undefined);
        localPathInput = '';
        customTitleInput = '';
      }
    } catch (err) {
      console.error(err);
    }
  }

  async function handleBrowseFolder() {
    try {
      const selected = await pickFolderDialog();
      if (selected) {
        await onAddFile(selected, customTitleInput.trim() || undefined);
        localPathInput = '';
        customTitleInput = '';
      }
    } catch (err) {
      console.error(err);
    }
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
    if (e.dataTransfer && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0] as any;
      const filePath = file.path || file.name;
      if (filePath) {
        await onAddFile(filePath, customTitleInput.trim() || undefined);
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

<svelte:window on:keydown={handleKeyDown} />

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

  <!-- Drag & Drop Zone (Clickable to select files or folders) -->
  <div 
    class="dropzone {isDragOver ? 'dragover' : ''} group cursor-pointer transition-all duration-200 hover:border-sky-400 hover:bg-sky-50/30" 
    role="button"
    tabindex="0"
    on:click={() => showPickerModal = true}
    on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); showPickerModal = true; } }}
    on:dragover|preventDefault={() => isDragOver = true}
    on:dragleave|preventDefault={() => isDragOver = false}
    on:drop={handleDrop}
  >
    <div class="w-12 h-12 rounded-full bg-sky-50 text-sky-600 flex items-center justify-center mb-3 group-hover:scale-110 group-hover:bg-sky-100 transition-all duration-300">
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
          placeholder="C:\Users\Documents\MyFolder หรือ MyFile.zip" 
          class="w-full pl-9 pr-3 py-2 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:border-sky-500 focus:bg-white text-slate-800 transition-colors font-mono"
          required
        />
        <svg class="w-4 h-4 text-slate-400 absolute left-3 top-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
      </div>
      <button 
        type="button" 
        on:click={handleBrowseFile} 
        title={t.btnBrowse}
        class="px-2.5 py-2 text-xs font-semibold bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg border border-slate-200 transition-colors flex items-center gap-1 active:scale-95"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
        </svg>
        <span>{t.btnBrowse}</span>
      </button>
      <button 
        type="button" 
        on:click={handleBrowseFolder} 
        title={t.btnBrowseFolder}
        class="px-2.5 py-2 text-xs font-semibold bg-amber-50 hover:bg-amber-100 text-amber-800 rounded-lg border border-amber-200 transition-colors flex items-center gap-1 active:scale-95"
      >
        <svg class="w-3.5 h-3.5 text-amber-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        <span>{t.btnBrowseFolder}</span>
      </button>
      <button 
        type="submit" 
        disabled={isProcessing || !localPathInput.trim()} 
        class="btn-primary disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap"
      >
        <span>{isProcessing ? t.loading : t.btnShare}</span>
      </button>
    </div>
  </form>
</section>

<!-- Item Selector Modal (Triggered by clicking the dropzone) -->
{#if showPickerModal}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
  <div 
    class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4 animate-fade-in"
    on:click={() => showPickerModal = false}
    role="dialog"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions a11y-no-static-element-interactions -->
    <div 
      class="bg-white rounded-2xl border border-slate-200 shadow-2xl max-w-sm w-full p-5 text-center animate-scale-up"
      on:click|stopPropagation
      role="document"
    >
      <h3 class="text-sm font-bold text-slate-800 mb-1">{t.pickerTitle}</h3>
      <p class="text-xs text-slate-500 mb-4">{t.pickerSubtitle}</p>

      <div class="grid grid-cols-2 gap-3 mb-3">
        <!-- Choose File -->
        <button
          type="button"
          on:click={() => { showPickerModal = false; handleBrowseFile(); }}
          class="flex flex-col items-center justify-center p-4 rounded-xl border-2 border-slate-200 hover:border-sky-500 hover:bg-sky-50/60 transition-all group cursor-pointer active:scale-95 text-center"
        >
          <div class="w-12 h-12 rounded-full bg-sky-100 text-sky-600 flex items-center justify-center mb-2.5 group-hover:scale-110 group-hover:bg-sky-600 group-hover:text-white transition-all shadow-sm">
            <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
              <polyline points="14 2 14 8 20 8"></polyline>
            </svg>
          </div>
          <span class="text-xs font-bold text-slate-800 group-hover:text-sky-700">{t.pickerFile}</span>
          <span class="text-[10px] text-slate-400 mt-0.5">{t.pickerFileDesc}</span>
        </button>

        <!-- Choose Folder -->
        <button
          type="button"
          on:click={() => { showPickerModal = false; handleBrowseFolder(); }}
          class="flex flex-col items-center justify-center p-4 rounded-xl border-2 border-slate-200 hover:border-amber-500 hover:bg-amber-50/60 transition-all group cursor-pointer active:scale-95 text-center"
        >
          <div class="w-12 h-12 rounded-full bg-amber-100 text-amber-600 flex items-center justify-center mb-2.5 group-hover:scale-110 group-hover:bg-amber-600 group-hover:text-white transition-all shadow-sm">
            <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
            </svg>
          </div>
          <span class="text-xs font-bold text-slate-800 group-hover:text-amber-700">{t.pickerFolder}</span>
          <span class="text-[10px] text-slate-400 mt-0.5">{t.pickerFolderDesc}</span>
        </button>
      </div>

      <button
        type="button"
        on:click={() => showPickerModal = false}
        class="text-xs text-slate-400 hover:text-slate-600 font-medium py-1 px-3"
      >
        {t.btnCancel}
      </button>
    </div>
  </div>
{/if}

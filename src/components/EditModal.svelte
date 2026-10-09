<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import type { ShareItem } from '../lib/api';

  export let isOpen = false;
  export let item: ShareItem | null = null;
  export let lang: Language = 'th';
  export let onClose: () => void;
  export let onSave: (id: string, title: string, desc: string, directDownload: boolean) => Promise<void>;

  $: t = I18N[lang];

  let titleInput = '';
  let descInput = '';
  let directDownloadInput = false;
  let lastItemId: string | null = null;

  $: if (isOpen && item && item.id !== lastItemId) {
    lastItemId = item.id;
    titleInput = item.title;
    descInput = item.description || '';
    directDownloadInput = !!item.directDownload;
  }

  $: if (!isOpen) {
    lastItemId = null;
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!item) return;
    await onSave(item.id, titleInput.trim(), descInput.trim(), directDownloadInput);
    onClose();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

{#if isOpen && item}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-noninteractive-element-interactions -->
  <div 
    class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4 animate-fade-in"
    on:click={handleBackdropClick}
    role="dialog"
    tabindex="-1"
  >
    <div class="bg-white rounded-2xl border border-slate-200/90 shadow-2xl max-w-md w-full overflow-hidden animate-scale-up">
      <div class="px-5 py-4 border-b border-slate-100 flex items-center justify-between">
        <span class="font-bold text-slate-900 text-sm">{t.editModalTitle}</span>
        <button type="button" on:click={onClose} class="text-slate-400 hover:text-slate-600 p-1.5 rounded-lg hover:bg-slate-100">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>

      <form on:submit={handleSubmit} class="p-5 space-y-4">
        <div>
          <label for="edit-title" class="block text-xs font-bold text-slate-700 mb-1">{t.labelTitle}</label>
          <input 
            id="edit-title"
            type="text" 
            bind:value={titleInput} 
            required 
            class="w-full px-3 py-2 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:border-sky-500 focus:bg-white text-slate-800"
          />
        </div>

        <div>
          <label for="edit-desc" class="block text-xs font-bold text-slate-700 mb-1">{t.labelDesc}</label>
          <textarea 
            id="edit-desc"
            bind:value={descInput} 
            rows="2"
            class="w-full px-3 py-2 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:border-sky-500 focus:bg-white text-slate-800 resize-none"
            placeholder="คำอธิบายเพิ่มเติมเกี่ยวกับไฟล์นี้..."
          ></textarea>
        </div>

        <!-- Download Mode Selector -->
        <div>
          <span class="block text-xs font-bold text-slate-700 mb-1.5">{t.labelDownloadMode}</span>
          <div class="grid grid-cols-2 gap-2">
            <label class="flex items-start gap-2 p-2.5 rounded-xl border cursor-pointer transition-all {directDownloadInput === false ? 'border-sky-500 bg-sky-50/70 text-sky-950 shadow-xs' : 'border-slate-200 bg-slate-50 text-slate-600 hover:bg-slate-100'}">
              <input 
                type="radio" 
                name="downloadMode" 
                checked={directDownloadInput === false}
                on:change={() => directDownloadInput = false}
                class="mt-0.5 text-sky-600 focus:ring-sky-500 cursor-pointer"
              />
              <div class="text-[11px] leading-tight select-none">
                <div class="font-bold">{t.modeLandingPage}</div>
                <div class="text-[10px] text-slate-500 font-normal mt-0.5">{t.modeLandingPageDesc}</div>
              </div>
            </label>

            <label class="flex items-start gap-2 p-2.5 rounded-xl border cursor-pointer transition-all {directDownloadInput === true ? 'border-sky-500 bg-sky-50/70 text-sky-950 shadow-xs' : 'border-slate-200 bg-slate-50 text-slate-600 hover:bg-slate-100'}">
              <input 
                type="radio" 
                name="downloadMode" 
                checked={directDownloadInput === true}
                on:change={() => directDownloadInput = true}
                class="mt-0.5 text-sky-600 focus:ring-sky-500 cursor-pointer"
              />
              <div class="text-[11px] leading-tight select-none">
                <div class="font-bold">{t.modeDirectDownload}</div>
                <div class="text-[10px] text-slate-500 font-normal mt-0.5">{t.modeDirectDownloadDesc}</div>
              </div>
            </label>
          </div>
        </div>

        <div>
          <span class="block text-xs font-bold text-slate-500 mb-1">{t.labelOriginalPath}</span>
          <div class="text-[11px] font-mono bg-slate-50 border border-slate-200/80 p-2 rounded-lg text-slate-600 break-all select-all">
            {item.filePath}
          </div>
        </div>

        <div class="pt-3 border-t border-slate-100 flex justify-end gap-2">
          <button type="button" on:click={onClose} class="btn-secondary !text-xs !py-1.5 !px-3">
            {t.btnCancel}
          </button>
          <button type="submit" class="btn-primary !text-xs !py-1.5 !px-4">
            {t.btnSave}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

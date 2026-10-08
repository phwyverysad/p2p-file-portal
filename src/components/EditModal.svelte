<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import type { ShareItem } from '../lib/api';

  export let isOpen = false;
  export let item: ShareItem | null = null;
  export let lang: Language = 'th';
  export let onClose: () => void;
  export let onSave: (id: string, title: string, desc: string) => Promise<void>;

  $: t = I18N[lang];

  let titleInput = '';
  let descInput = '';

  $: if (item) {
    titleInput = item.title;
    descInput = item.description || '';
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!item) return;
    await onSave(item.id, titleInput.trim(), descInput.trim());
    onClose();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

{#if isOpen && item}
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
          <label class="block text-xs font-bold text-slate-700 mb-1">{t.labelTitle}</label>
          <input 
            type="text" 
            bind:value={titleInput} 
            required 
            class="w-full px-3 py-2 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:border-sky-500 focus:bg-white text-slate-800"
          />
        </div>

        <div>
          <label class="block text-xs font-bold text-slate-700 mb-1">{t.labelDesc}</label>
          <textarea 
            bind:value={descInput} 
            rows="2"
            class="w-full px-3 py-2 text-xs bg-slate-50 border border-slate-200 rounded-lg focus:outline-none focus:border-sky-500 focus:bg-white text-slate-800 resize-none"
            placeholder="คำอธิบายเพิ่มเติมเกี่ยวกับไฟล์นี้..."
          ></textarea>
        </div>

        <div>
          <label class="block text-xs font-bold text-slate-500 mb-1">{t.labelOriginalPath}</label>
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

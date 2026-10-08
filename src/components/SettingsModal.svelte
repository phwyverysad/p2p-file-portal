<script lang="ts">
  import { I18N, type Language } from '../lib/i18n';
  import type { AppSettings, ServerStatus } from '../lib/api';

  export let isOpen = false;
  export let settings: AppSettings;
  export let serverStatus: ServerStatus | null = null;
  export let onClose: () => void;
  export let onSaveSettings: (settings: AppSettings) => Promise<void>;

  $: t = I18N[settings.language];

  function setLanguage(l: Language) {
    settings.language = l;
    onSaveSettings(settings);
  }

  function handleToggleTray(e: Event) {
    settings.minimizeToTray = (e.target as HTMLInputElement).checked;
    onSaveSettings(settings);
  }

  function handleToggleAutoStart(e: Event) {
    settings.autoStart = (e.target as HTMLInputElement).checked;
    onSaveSettings(settings);
  }

  function handleToggleAutoCopy(e: Event) {
    settings.autoCopy = (e.target as HTMLInputElement).checked;
    onSaveSettings(settings);
  }

  function handleToggleContextMenu(e: Event) {
    settings.contextMenu = (e.target as HTMLInputElement).checked;
    onSaveSettings(settings);
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

{#if isOpen}
  <div 
    class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4 animate-fade-in"
    on:click={handleBackdropClick}
    role="dialog"
    tabindex="-1"
  >
    <div class="bg-white rounded-2xl border border-slate-200/90 shadow-2xl max-w-md w-full overflow-hidden animate-scale-up">
      <!-- Modal Header -->
      <div class="px-5 py-4 border-b border-slate-100 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-sky-50 text-sky-600 flex items-center justify-center">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3"></circle>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
            </svg>
          </div>
          <span class="font-bold text-slate-900 text-sm">{t.settingsTitle}</span>
        </div>
        <button 
          type="button" 
          on:click={onClose} 
          class="text-slate-400 hover:text-slate-600 p-1.5 rounded-lg hover:bg-slate-100 transition-colors"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>

      <!-- Settings List -->
      <div class="p-5 space-y-4 max-h-[70vh] overflow-y-auto">
        <!-- Language -->
        <div class="flex items-center justify-between gap-4 py-1">
          <div>
            <div class="text-xs font-bold text-slate-800">{t.settingLanguage}</div>
            <div class="text-[11px] text-slate-500 mt-0.5">{t.settingLanguageDesc}</div>
          </div>
          <div class="flex bg-slate-100 p-0.5 rounded-lg border border-slate-200">
            <button 
              type="button" 
              class="px-2.5 py-1 text-xs font-semibold rounded-md transition-all {settings.language === 'th' ? 'bg-white shadow-sm text-sky-700' : 'text-slate-600'}"
              on:click={() => setLanguage('th')}
            >
              ไทย
            </button>
            <button 
              type="button" 
              class="px-2.5 py-1 text-xs font-semibold rounded-md transition-all {settings.language === 'en' ? 'bg-white shadow-sm text-sky-700' : 'text-slate-600'}"
              on:click={() => setLanguage('en')}
            >
              English
            </button>
          </div>
        </div>

        <div class="border-t border-slate-100"></div>

        <!-- Windows Context Menu -->
        <div class="flex items-center justify-between gap-4 py-1">
          <div>
            <div class="text-xs font-bold text-slate-800 flex items-center gap-1.5">
              <span>{t.settingContextMenu}</span>
              <span class="badge text-[10px] bg-sky-50 text-sky-700 border-sky-200">Windows Shell</span>
            </div>
            <div class="text-[11px] text-slate-500 mt-0.5">{t.settingContextMenuDesc}</div>
          </div>
          <label class="toggle-switch shrink-0">
            <input type="checkbox" checked={settings.contextMenu} on:change={handleToggleContextMenu}>
            <span class="toggle-slider"></span>
          </label>
        </div>

        <div class="border-t border-slate-100"></div>

        <!-- System Tray -->
        <div class="flex items-center justify-between gap-4 py-1">
          <div>
            <div class="text-xs font-bold text-slate-800">{t.settingTray}</div>
            <div class="text-[11px] text-slate-500 mt-0.5">{t.settingTrayDesc}</div>
          </div>
          <label class="toggle-switch shrink-0">
            <input type="checkbox" checked={settings.minimizeToTray} on:change={handleToggleTray}>
            <span class="toggle-slider"></span>
          </label>
        </div>

        <div class="border-t border-slate-100"></div>

        <!-- Auto Start -->
        <div class="flex items-center justify-between gap-4 py-1">
          <div>
            <div class="text-xs font-bold text-slate-800">{t.settingAutoStart}</div>
            <div class="text-[11px] text-slate-500 mt-0.5">{t.settingAutoStartDesc}</div>
          </div>
          <label class="toggle-switch shrink-0">
            <input type="checkbox" checked={settings.autoStart} on:change={handleToggleAutoStart}>
            <span class="toggle-slider"></span>
          </label>
        </div>

        <div class="border-t border-slate-100"></div>

        <!-- Auto Copy -->
        <div class="flex items-center justify-between gap-4 py-1">
          <div>
            <div class="text-xs font-bold text-slate-800">{t.settingAutoCopy}</div>
            <div class="text-[11px] text-slate-500 mt-0.5">{t.settingAutoCopyDesc}</div>
          </div>
          <label class="toggle-switch shrink-0">
            <input type="checkbox" checked={settings.autoCopy} on:change={handleToggleAutoCopy}>
            <span class="toggle-slider"></span>
          </label>
        </div>

        <!-- App Info -->
        <div class="p-3 bg-slate-50 border border-slate-200/80 rounded-xl text-xs space-y-1">
          <div class="flex items-center justify-between font-bold text-slate-800">
            <span>File Sharer</span>
            <span class="text-slate-400 font-normal font-mono">v1.0.0</span>
          </div>
          <div class="text-[11px] text-slate-500 flex items-center gap-2">
            <span>{t.serverStatus}</span>
            <span>•</span>
            <span>{t.serverPort} {serverStatus?.port || 3030}</span>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3.5 bg-slate-50 border-t border-slate-100 flex justify-end">
        <button type="button" class="btn-primary !text-xs !py-1.5 !px-4" on:click={onClose}>
          {t.btnDone}
        </button>
      </div>
    </div>
  </div>
{/if}

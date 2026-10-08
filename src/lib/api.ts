// Tauri IPC and REST API client layer
import { invoke } from '@tauri-apps/api/core';

export interface ShareItem {
  id: string;
  filePath: string;
  originalName: string;
  title: string;
  description: string;
  size: number;
  mimeType: string;
  createdAt: string;
  downloads: number;
  views: number;
  shortUrl: string | null;
  publicUrl: string;
  localUrl: string;
  directDownload?: boolean;
}

export interface TunnelStatus {
  status: 'offline' | 'starting' | 'online' | 'error';
  publicUrl: string | null;
  errorMessage: string | null;
}

export interface ServerStatus {
  server: string;
  port: number;
  tunnel: TunnelStatus;
  lanIp: string;
}

export interface AppSettings {
  language: 'th' | 'en';
  minimizeToTray: boolean;
  autoStart: boolean;
  autoCopy: boolean;
  contextMenu: boolean;
  confirmDelete: boolean;
}

export function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function getFileCategoryIcon(fileName: string): string {
  const ext = fileName.split('.').pop()?.toLowerCase() || '';
  if (['mp4', 'mkv', 'mov', 'webm', 'avi', 'wmv'].includes(ext)) return 'video';
  if (['mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a'].includes(ext)) return 'audio';
  if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2'].includes(ext)) return 'archive';
  if (['pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt', 'csv'].includes(ext)) return 'document';
  if (['exe', 'msi', 'bat', 'cmd', 'ps1', 'sh'].includes(ext)) return 'executable';
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'].includes(ext)) return 'image';
  return 'file';
}

// Fallback to fetch if Tauri IPC not ready (e.g. dev browser mode)
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export async function listShares(): Promise<ShareItem[]> {
  if (isTauri) {
    return await invoke<ShareItem[]>('list_shares');
  }
  const res = await fetch('http://localhost:3030/api/shares');
  const data = await res.json();
  return data.shares || [];
}

export async function addLocalShare(filePath: string, title?: string): Promise<ShareItem> {
  if (isTauri) {
    return await invoke<ShareItem>('add_local_share', { filePath, title });
  }
  const res = await fetch('http://localhost:3030/api/shares/local', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ filePath, title })
  });
  const data = await res.json();
  if (!res.ok) throw new Error(data.error || 'Failed to add share');
  return data.share;
}

export async function updateShare(
  id: string, 
  title?: string, 
  description?: string, 
  directDownload?: boolean
): Promise<ShareItem> {
  if (isTauri) {
    return await invoke<ShareItem>('update_share', { id, title, description, directDownload });
  }
  const res = await fetch(`http://localhost:3030/api/shares/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ title, description, directDownload })
  });
  return await res.json();
}

export async function revokeShare(id: string): Promise<boolean> {
  if (isTauri) {
    return await invoke<boolean>('revoke_share', { id });
  }
  const res = await fetch(`http://localhost:3030/api/shares/${id}`, { method: 'DELETE' });
  return res.ok;
}

export async function shortenShareUrl(id: string): Promise<string> {
  if (isTauri) {
    return await invoke<string>('shorten_share_url', { id });
  }
  const res = await fetch(`http://localhost:3030/api/shares/${id}/shorten`, { method: 'POST' });
  const data = await res.json();
  return data.shortUrl;
}

export async function getStatus(): Promise<ServerStatus> {
  if (isTauri) {
    return await invoke<ServerStatus>('get_status');
  }
  const res = await fetch('http://localhost:3030/api/status');
  return await res.json();
}

export async function getAppSettings(): Promise<AppSettings> {
  if (isTauri) {
    return await invoke<AppSettings>('get_app_settings');
  }
  return {
    language: 'th',
    minimizeToTray: true,
    autoStart: false,
    autoCopy: true,
    contextMenu: true,
  };
}

export async function saveAppSettings(settings: AppSettings): Promise<AppSettings> {
  if (isTauri) {
    return await invoke<AppSettings>('save_app_settings', { settings });
  }
  return settings;
}

export async function pickFileDialog(): Promise<string | null> {
  if (isTauri) {
    return await invoke<string | null>('pick_file_dialog');
  }
  return null;
}

export async function showInFolder(path: string): Promise<void> {
  if (isTauri) {
    await invoke('show_in_folder', { path });
  }
}

export async function retryTunnel(): Promise<void> {
  if (isTauri) {
    await invoke('retry_tunnel');
  } else {
    await fetch('http://localhost:3030/api/tunnel/retry', { method: 'POST' });
  }
}

export async function copyToClipboard(text: string): Promise<boolean> {
  if (isTauri) {
    try {
      await invoke('copy_to_clipboard', { text });
      return true;
    } catch (e) {
      console.warn('Native clipboard error, fallback to navigator:', e);
    }
  }
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (err) {
    console.warn('Clipboard write error:', err);
    return false;
  }
}

export async function getPendingShares(): Promise<string[]> {
  if (isTauri) {
    return await invoke<string[]>('get_pending_shares');
  }
  return [];
}

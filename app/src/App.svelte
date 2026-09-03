<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import PopupView from './lib/popup/PopupView.svelte';
  import { hiddenPopup, type PopupViewModel } from './lib/popup/model';
  import SettingsView from './lib/settings/SettingsView.svelte';

  type ModelPackStatus = {
    id: string;
    installed: boolean;
    installSupported: boolean;
  };

  const windowLabel = getCurrentWindow().label;
  const isSettings = windowLabel === 'settings';

  let popup: PopupViewModel = $state(hiddenPopup);
  let modelPack: ModelPackStatus | null = $state(null);
  let modelAction: 'idle' | 'installing' | 'removing' | 'error' = $state('idle');

  onMount(() => {
    let disposed = false;
    const listeners = [
      listen<ModelPackStatus>('model-pack-state', (event) => {
        if (!disposed) {
          modelPack = event.payload;
          modelAction = 'idle';
        }
      }),
    ];

    if (!isSettings) {
      listeners.push(
        listen<PopupViewModel>('popup-state', (event) => {
          if (!disposed) popup = event.payload;
        }),
      );
      void invoke<PopupViewModel>('get_popup_state').then((state) => {
        if (!disposed) popup = state;
      });
    }

    void invoke<ModelPackStatus>('get_model_pack_status').then((state) => {
      if (!disposed) modelPack = state;
    });

    return () => {
      disposed = true;
      for (const listener of listeners) {
        void listener.then((stop) => stop());
      }
    };
  });

  function dismiss(): void {
    void invoke('dismiss_popup');
  }

  function installModel(): void {
    if (modelAction === 'installing') return;
    modelAction = 'installing';
    void invoke<ModelPackStatus>('install_model_pack')
      .then((state) => {
        modelPack = state;
        modelAction = 'idle';
      })
      .catch(() => {
        modelAction = 'error';
      });
  }

  function removeModel(): void {
    if (modelAction === 'removing') return;
    modelAction = 'removing';
    void invoke<ModelPackStatus>('remove_model_pack')
      .then((state) => {
        modelPack = state;
        modelAction = 'idle';
      })
      .catch(() => {
        modelAction = 'error';
      });
  }

  function quit(): void {
    void invoke('quit_app');
  }
</script>

{#if isSettings}
  <SettingsView
    {modelPack}
    {modelAction}
    onInstallModel={installModel}
    onRemoveModel={removeModel}
    onQuit={quit}
  />
{:else}
  <main class="popup-root">
    <PopupView
      model={popup}
      onDismiss={dismiss}
      onInstallModel={modelPack?.installSupported ? installModel : undefined}
      modelInstallState={modelAction === 'installing'
        ? 'installing'
        : modelAction === 'error'
          ? 'error'
          : modelPack?.installed
            ? 'installed'
            : 'idle'}
    />
  </main>
{/if}

<style>
  .popup-root {
    width: 100%;
    height: 100%;
  }
</style>

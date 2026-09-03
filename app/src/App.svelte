<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';
  import PopupView from './lib/popup/PopupView.svelte';
  import { hiddenPopup, type PopupViewModel } from './lib/popup/model';

  type ModelPackStatus = {
    id: string;
    installed: boolean;
    installSupported: boolean;
  };

  let popup: PopupViewModel = $state(hiddenPopup);
  let modelPack: ModelPackStatus | null = $state(null);
  let modelInstallState: 'idle' | 'installing' | 'installed' | 'error' = $state('idle');

  onMount(() => {
    let disposed = false;
    const unlisten = listen<PopupViewModel>('popup-state', (event) => {
      if (!disposed) popup = event.payload;
    });

    void invoke<PopupViewModel>('get_popup_state').then((state) => {
      if (!disposed) popup = state;
    });
    void invoke<ModelPackStatus>('get_model_pack_status').then((state) => {
      if (!disposed) modelPack = state;
    });

    return () => {
      disposed = true;
      void unlisten.then((stop) => stop());
    };
  });

  function dismiss(): void {
    void invoke('dismiss_popup');
  }

  function installModel(): void {
    if (modelInstallState === 'installing') return;
    modelInstallState = 'installing';
    void invoke<ModelPackStatus>('install_model_pack')
      .then((state) => {
        modelPack = state;
        modelInstallState = state.installed ? 'installed' : 'error';
      })
      .catch(() => {
        modelInstallState = 'error';
      });
  }
</script>

<main>
  <PopupView
    model={popup}
    onDismiss={dismiss}
    onInstallModel={modelPack?.installSupported ? installModel : undefined}
    {modelInstallState}
  />
</main>

<style>
  main {
    width: 100%;
    height: 100%;
  }
</style>

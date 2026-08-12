<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';
  import PopupView from './lib/popup/PopupView.svelte';
  import { hiddenPopup, type PopupViewModel } from './lib/popup/model';

  let popup: PopupViewModel = $state(hiddenPopup);

  onMount(() => {
    let disposed = false;
    const unlisten = listen<PopupViewModel>('popup-state', (event) => {
      if (!disposed) popup = event.payload;
    });

    void invoke<PopupViewModel>('get_popup_state').then((state) => {
      if (!disposed) popup = state;
    });

    return () => {
      disposed = true;
      void unlisten.then((stop) => stop());
    };
  });

  function dismiss(): void {
    void invoke('dismiss_popup');
  }
</script>

<main>
  <PopupView model={popup} onDismiss={dismiss} />
</main>

<style>
  main {
    width: 100%;
    height: 100%;
  }
</style>

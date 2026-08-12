<script lang="ts">
  import PopupView from './lib/popup/PopupView.svelte';
  import { hiddenPopup, type PopupViewModel } from './lib/popup/model';

  let popup: PopupViewModel = $state(hiddenPopup);

  function dismiss(): void {
    popup = hiddenPopup;
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

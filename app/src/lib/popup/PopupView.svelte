<script lang="ts">
  import type { PopupViewModel } from './model';

  let {
    model,
    onDismiss,
  }: {
    model: PopupViewModel;
    onDismiss?: () => void;
  } = $props();
</script>

{#if model.status !== 'hidden'}
  <section class="popup" aria-live="polite" aria-label="ClipLingo translation">
    <header class="header">
      <span class="brand">ClipLingo</span>
      {#if onDismiss}
        <button class="close" type="button" aria-label="Close translation" onclick={onDismiss}>×</button>
      {/if}
    </header>

    <div class="content">
      {#if model.status === 'capturing'}
        <p class="status">Capturing…</p>
      {:else if model.status === 'translating'}
        <p class="source">{model.sourceText}</p>
        <p class="status">Translating…</p>
      {:else if model.status === 'ready'}
        <p class="source">{model.sourceText}</p>
        <p class="translation">{model.translatedText}</p>
      {:else if model.status === 'error'}
        <p class="error" data-error-code={model.errorCode}>Unable to translate this selection.</p>
      {/if}
    </div>
  </section>
{/if}

<style>
  .popup {
    width: 100%;
    min-height: 100%;
    padding: 14px 16px 16px;
    border: 1px solid color-mix(in srgb, currentColor 14%, transparent);
    border-radius: 14px;
    background: color-mix(in srgb, Canvas 94%, transparent);
    box-shadow: 0 14px 38px rgb(0 0 0 / 18%);
    backdrop-filter: blur(18px);
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 24px;
    margin-bottom: 10px;
  }

  .brand {
    font-size: 12px;
    font-weight: 650;
    letter-spacing: 0.01em;
    opacity: 0.62;
  }

  .close {
    width: 26px;
    height: 26px;
    border: 0;
    border-radius: 7px;
    color: inherit;
    background: transparent;
    cursor: pointer;
  }

  .close:hover {
    background: color-mix(in srgb, currentColor 8%, transparent);
  }

  .content {
    display: grid;
    gap: 9px;
  }

  p {
    margin: 0;
  }

  .source {
    max-height: 44px;
    overflow: hidden;
    font-size: 12px;
    line-height: 1.45;
    opacity: 0.58;
  }

  .translation {
    max-height: 78px;
    overflow: auto;
    font-size: 15px;
    font-weight: 520;
    line-height: 1.55;
  }

  .status,
  .error {
    font-size: 14px;
    line-height: 1.5;
  }

  .status {
    opacity: 0.66;
  }

  .error {
    color: #b42318;
  }

  @media (prefers-color-scheme: dark) {
    .error {
      color: #ff8a80;
    }
  }
</style>

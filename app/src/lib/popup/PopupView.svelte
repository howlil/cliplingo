<script lang="ts">
  import type { PopupViewModel } from './model';

  let {
    model,
    onDismiss,
    onInstallModel,
    modelInstallState = 'idle',
  }: {
    model: PopupViewModel;
    onDismiss?: () => void;
    onInstallModel?: () => void;
    modelInstallState?: 'idle' | 'installing' | 'installed' | 'error';
  } = $props();
</script>

{#if model.status !== 'hidden'}
  <section class="popup" aria-live="polite" aria-label="ClipLingo translation">
    <header class="routebar" data-tauri-drag-region>
      <span class="route" data-tauri-drag-region>EN → ID</span>
      {#if onDismiss}
        <button class="close" type="button" aria-label="Close translation" onclick={onDismiss}>×</button>
      {/if}
    </header>

    <div class="content">
      {#if model.status === 'translating'}
        <p class="source">{model.sourceText}</p>
        <p class="status">Translating…</p>
      {:else if model.status === 'ready'}
        <p class="source">{model.sourceText}</p>
        <p class="translation">{model.translatedText}</p>
      {:else if model.status === 'error' && model.errorCode === 'model_unavailable'}
        {#if modelInstallState === 'installing'}
          <p class="status">Downloading and verifying the offline English → Indonesian model…</p>
        {:else if modelInstallState === 'installed'}
          <p class="status">Offline model installed. Select text and use the shortcut again.</p>
        {:else}
          <p class="error">
            {modelInstallState === 'error'
              ? 'The offline model could not be installed.'
              : 'The English → Indonesian offline model is not installed.'}
          </p>
          {#if onInstallModel}
            <button class="action" type="button" onclick={onInstallModel}>
              {modelInstallState === 'error' ? 'Retry model install' : 'Install offline model'}
            </button>
          {/if}
        {/if}
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
    border: 1px solid color-mix(in srgb, CanvasText 16%, transparent);
    border-radius: 8px;
    background: Canvas;
    color: CanvasText;
    box-shadow: 0 8px 24px rgb(0 0 0 / 14%);
    overflow: hidden;
  }

  .routebar {
    height: 34px;
    padding: 0 7px 0 12px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid color-mix(in srgb, CanvasText 12%, transparent);
    user-select: none;
    cursor: move;
  }

  *[data-tauri-drag-region] {
    app-region: drag;
  }

  .route {
    font-size: 11.5px;
    font-weight: 650;
    letter-spacing: 0.02em;
    color: color-mix(in srgb, CanvasText 68%, transparent);
  }

  .close {
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: 4px;
    color: inherit;
    background: transparent;
    cursor: pointer;
    line-height: 1;
  }

  .close:hover,
  .action:hover {
    background: color-mix(in srgb, CanvasText 7%, transparent);
  }

  .content {
    min-height: 144px;
    padding: 12px 14px 14px;
    display: grid;
    align-content: start;
    gap: 9px;
  }

  p {
    margin: 0;
  }

  .source {
    max-height: 38px;
    overflow: hidden;
    font-size: 11.5px;
    line-height: 1.45;
    color: color-mix(in srgb, CanvasText 54%, transparent);
  }

  .translation {
    max-height: 82px;
    overflow: auto;
    font-size: 15px;
    font-weight: 500;
    line-height: 1.52;
  }

  .status,
  .error {
    font-size: 13px;
    line-height: 1.5;
  }

  .status {
    color: color-mix(in srgb, CanvasText 62%, transparent);
  }

  .error {
    color: #b42318;
  }

  .action {
    justify-self: start;
    padding: 6px 9px;
    border: 1px solid color-mix(in srgb, CanvasText 18%, transparent);
    border-radius: 5px;
    color: inherit;
    background: transparent;
    font: inherit;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
  }

  @media (prefers-color-scheme: dark) {
    .popup {
      box-shadow: 0 8px 24px rgb(0 0 0 / 28%);
    }

    .error {
      color: #ff8a80;
    }
  }
</style>

<script lang="ts">
  let {
    modelPack,
    modelAction = 'idle',
    onInstallModel,
    onRemoveModel,
    onQuit,
  }: {
    modelPack: { id: string; installed: boolean; installSupported: boolean } | null;
    modelAction?: 'idle' | 'installing' | 'removing' | 'error';
    onInstallModel?: () => void;
    onRemoveModel?: () => void;
    onQuit?: () => void;
  } = $props();

  const route = 'English → Indonesian';
  const shortcut = 'Ctrl + Alt + T';
</script>

<section class="settings" aria-label="ClipLingo settings">
  <header class="intro">
    <div>
      <h1>ClipLingo</h1>
      <p>Running in the Windows notification area.</p>
    </div>
    <span class="running">Running</span>
  </header>

  <div class="section">
    <div class="row">
      <div>
        <div class="label">Translation</div>
        <div class="hint">Primary offline route</div>
      </div>
      <div class="value">{route}</div>
    </div>

    <div class="row">
      <div>
        <div class="label">Shortcut</div>
        <div class="hint">Translate the current text selection</div>
      </div>
      <kbd>{shortcut}</kbd>
    </div>
  </div>

  <div class="section model-section">
    <div class="row model-row">
      <div>
        <div class="label">Offline model</div>
        <div class="hint">
          {#if modelPack?.installed}
            EN → ID model installed
          {:else if modelPack}
            EN → ID model not installed
          {:else}
            Checking model status…
          {/if}
        </div>
      </div>

      {#if modelPack?.installed}
        <button class="secondary" type="button" disabled={modelAction === 'removing'} onclick={onRemoveModel}>
          {modelAction === 'removing' ? 'Removing…' : 'Remove'}
        </button>
      {:else}
        <button
          class="primary"
          type="button"
          disabled={!modelPack?.installSupported || modelAction === 'installing'}
          onclick={onInstallModel}
        >
          {modelAction === 'installing' ? 'Installing…' : 'Install model'}
        </button>
      {/if}
    </div>

    {#if modelAction === 'error'}
      <p class="error">The model action could not be completed.</p>
    {:else if modelPack && !modelPack.installSupported && !modelPack.installed}
      <p class="note">This build does not include a model download source.</p>
    {/if}
  </div>

  <footer>
    <p>Closing Settings keeps ClipLingo running in the system tray.</p>
    {#if onQuit}
      <button class="quit" type="button" onclick={onQuit}>Quit ClipLingo</button>
    {/if}
  </footer>
</section>

<style>
  .settings {
    width: 100%;
    height: 100%;
    padding: 22px 24px 18px;
    background: Canvas;
    color: CanvasText;
    display: flex;
    flex-direction: column;
  }

  .intro,
  .row,
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
  }

  .intro {
    padding-bottom: 18px;
  }

  h1,
  p {
    margin: 0;
  }

  h1 {
    font-size: 20px;
    line-height: 1.25;
    font-weight: 650;
  }

  .intro p,
  .hint,
  footer p,
  .note,
  .error {
    color: color-mix(in srgb, CanvasText 58%, transparent);
  }

  .intro p {
    margin-top: 3px;
    font-size: 12px;
  }

  .running {
    font-size: 12px;
    font-weight: 600;
    color: color-mix(in srgb, CanvasText 72%, transparent);
  }

  .section {
    border-top: 1px solid color-mix(in srgb, CanvasText 14%, transparent);
  }

  .row {
    min-height: 70px;
  }

  .label,
  .value,
  kbd,
  button {
    font-size: 13px;
  }

  .label {
    font-weight: 600;
  }

  .hint {
    margin-top: 3px;
    font-size: 11.5px;
  }

  .value {
    font-weight: 550;
  }

  kbd {
    padding: 4px 7px;
    border: 1px solid color-mix(in srgb, CanvasText 18%, transparent);
    border-radius: 4px;
    background: color-mix(in srgb, Canvas 96%, CanvasText 4%);
    font-family: inherit;
  }

  button {
    border-radius: 5px;
    padding: 6px 10px;
    cursor: pointer;
  }

  button:disabled {
    cursor: default;
    opacity: 0.48;
  }

  .primary {
    border: 1px solid CanvasText;
    background: CanvasText;
    color: Canvas;
  }

  .secondary,
  .quit {
    border: 1px solid color-mix(in srgb, CanvasText 20%, transparent);
    background: transparent;
    color: inherit;
  }

  .model-section {
    padding-bottom: 10px;
  }

  .model-row {
    min-height: 72px;
  }

  .note,
  .error {
    margin: -2px 0 8px;
    font-size: 11.5px;
  }

  .error {
    color: #b42318;
  }

  footer {
    margin-top: auto;
    padding-top: 14px;
    border-top: 1px solid color-mix(in srgb, CanvasText 14%, transparent);
  }

  footer p {
    max-width: 250px;
    font-size: 11.5px;
    line-height: 1.4;
  }

  .quit:hover,
  .secondary:hover:not(:disabled) {
    background: color-mix(in srgb, CanvasText 6%, transparent);
  }

  @media (prefers-color-scheme: dark) {
    .error {
      color: #ff8a80;
    }
  }
</style>

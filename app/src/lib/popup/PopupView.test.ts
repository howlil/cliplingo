import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import PopupView from './PopupView.svelte';

const base = {
  sourceText: null,
  translatedText: null,
  errorCode: null,
} as const;

describe('PopupView', () => {
  it('shows capture feedback immediately', () => {
    render(PopupView, { model: { ...base, status: 'capturing' } });
    expect(screen.getByText('Capturing…')).toBeInTheDocument();
  });

  it('shows a ready translation', () => {
    render(PopupView, {
      model: {
        status: 'ready',
        sourceText: 'こんにちは',
        translatedText: '[FAKE] こんにちは',
        errorCode: null,
      },
    });

    expect(screen.getByText('[FAKE] こんにちは')).toBeInTheDocument();
  });

  it('offers explicit model installation when the offline pack is missing', async () => {
    const onInstallModel = vi.fn();
    render(PopupView, {
      model: {
        ...base,
        status: 'error',
        errorCode: 'model_unavailable',
      },
      onInstallModel,
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Install offline model' }));
    expect(onInstallModel).toHaveBeenCalledOnce();
  });

  it('emits dismiss intent from the close action', async () => {
    const onDismiss = vi.fn();
    render(PopupView, { model: { ...base, status: 'capturing' }, onDismiss });

    await fireEvent.click(screen.getByRole('button', { name: 'Close translation' }));
    expect(onDismiss).toHaveBeenCalledOnce();
  });
});

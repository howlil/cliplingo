import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import PopupView from './PopupView.svelte';

const base = {
  sourceText: null,
  translatedText: null,
  errorCode: null,
} as const;

describe('PopupView', () => {
  it('shows the direct EN to ID route and ready translation', () => {
    render(PopupView, {
      model: {
        status: 'ready',
        sourceText: 'The deployment failed yesterday.',
        translatedText: 'Deployment gagal kemarin.',
        errorCode: null,
      },
    });

    expect(screen.getByText('EN → ID')).toBeInTheDocument();
    expect(screen.getByText('Deployment gagal kemarin.')).toBeInTheDocument();
  });

  it('offers explicit model installation when the EN to ID pack is missing', async () => {
    const onInstallModel = vi.fn();
    render(PopupView, {
      model: {
        ...base,
        status: 'error',
        errorCode: 'model_unavailable',
      },
      onInstallModel,
    });

    expect(screen.getByText('The English → Indonesian offline model is not installed.')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Install offline model' }));
    expect(onInstallModel).toHaveBeenCalledOnce();
  });

  it('emits dismiss intent from the close action', async () => {
    const onDismiss = vi.fn();
    render(PopupView, {
      model: {
        status: 'ready',
        sourceText: 'hello',
        translatedText: 'halo',
        errorCode: null,
      },
      onDismiss,
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Close translation' }));
    expect(onDismiss).toHaveBeenCalledOnce();
  });
});

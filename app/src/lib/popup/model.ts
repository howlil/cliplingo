export type PopupStatus =
  | 'hidden'
  | 'capturing'
  | 'translating'
  | 'ready'
  | 'error';

export interface PopupViewModel {
  status: PopupStatus;
  sourceText: string | null;
  translatedText: string | null;
  errorCode: string | null;
}

export const hiddenPopup: PopupViewModel = {
  status: 'hidden',
  sourceText: null,
  translatedText: null,
  errorCode: null,
};

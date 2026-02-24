import WebApp from '@twa-dev/sdk';

export const tg = WebApp;

// Initialize Telegram Web App
tg.ready();
tg.expand();

// Theme colors
export const theme = {
  bgColor: tg.themeParams.bg_color || '#ffffff',
  textColor: tg.themeParams.text_color || '#000000',
  hintColor: tg.themeParams.hint_color || '#999999',
  linkColor: tg.themeParams.link_color || '#2481cc',
  buttonColor: tg.themeParams.button_color || '#2481cc',
  buttonTextColor: tg.themeParams.button_text_color || '#ffffff',
};

// Haptic feedback helpers
export const haptic = {
  light: () => tg.HapticFeedback.impactOccurred('light'),
  medium: () => tg.HapticFeedback.impactOccurred('medium'),
  heavy: () => tg.HapticFeedback.impactOccurred('heavy'),
  success: () => tg.HapticFeedback.notificationOccurred('success'),
  warning: () => tg.HapticFeedback.notificationOccurred('warning'),
  error: () => tg.HapticFeedback.notificationOccurred('error'),
};

// Main button helpers
export const mainButton = {
  show: (text: string, onClick: () => void) => {
    tg.MainButton.setText(text);
    tg.MainButton.onClick(onClick);
    tg.MainButton.show();
  },
  hide: () => {
    tg.MainButton.hide();
  },
  showProgress: () => {
    tg.MainButton.showProgress();
  },
  hideProgress: () => {
    tg.MainButton.hideProgress();
  },
};

// Etkin derlenmiş temayı bileşenlere iletir (simge seti, kedi paleti, Kedi Evi görselleri).
import { createContext, useContext } from 'react';

export const ThemeContext = createContext(null);
/** Etkin tema (derlenmiş) ya da null. */
export const useActiveTheme = () => useContext(ThemeContext)?.compiled ?? null;
export const useThemeApi = () => useContext(ThemeContext);

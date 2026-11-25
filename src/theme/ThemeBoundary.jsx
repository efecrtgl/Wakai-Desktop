// Tema yalıtımı: bir tema arayüzün çizilmesini bozarsa yedek temaya dönülür ve kullanıcıya
// sade bir açıklama gösterilir. Uygulama verilerine dokunulmaz.
import { Component } from 'react';
import { applyTheme, builtinTheme, compileTheme, FALLBACK_THEME_ID } from './engine.js';

export class ThemeBoundary extends Component {
  constructor(props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error) {
    return { error };
  }

  componentDidCatch(error) {
    console.error('arayüz hatası', error);
    try {
      applyTheme(compileTheme(builtinTheme(FALLBACK_THEME_ID)));
    } catch {
      // tokens.css varsayılanları geçerli
    }
    this.props.onFallback?.();
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="cc-crash" role="alert">
        <h1>Bir şeyler ters gitti</h1>
        <p>Arayüz çizilirken bir hata oluştu. Güvenli görünüme (Modern) geçildi; verileriniz etkilenmedi.</p>
        <button type="button" className="wk-btn wk-btn--primary wk-btn--md" onClick={() => this.setState({ error: null })}>
          Yeniden dene
        </button>
      </div>
    );
  }
}

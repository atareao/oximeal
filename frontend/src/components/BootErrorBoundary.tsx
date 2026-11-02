import { Component, type ErrorInfo, type ReactNode } from 'react';
import './boot-error-boundary.css';

export interface BootErrorBoundaryProps {
  children: ReactNode;
}

interface BootErrorBoundaryState {
  hasError: boolean;
}

/**
 * Error boundary de arranque **sin antd**.
 *
 * Envuelve el árbol completo antes de que antd se haya cargado; por eso su
 * pantalla de error es HTML/CSS plano. Una vez cargado `AppRoot`, el
 * `ErrorBoundary` interno (con antd) captura los errores del resto de la app.
 *
 * Es un componente de clase porque React solo ofrece `getDerivedStateFromError`
 * y `componentDidCatch` en clases: no existe equivalente con hooks.
 */
export class BootErrorBoundary extends Component<BootErrorBoundaryProps, BootErrorBoundaryState> {
  state: BootErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): BootErrorBoundaryState {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error('Error no controlado en el arranque', error, info);
  }

  private handleReload = (): void => {
    window.location.reload();
  };

  render(): ReactNode {
    if (this.state.hasError) {
      return (
        <div className="boot-error" role="alert" data-testid="boot-error">
          <h1 className="boot-error__title">Algo ha ido mal</h1>
          <p className="boot-error__text">
            No se pudo cargar OxiMeal. Recarga la página para continuar.
          </p>
          <button type="button" className="boot-error__reload" onClick={this.handleReload}>
            Recargar
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

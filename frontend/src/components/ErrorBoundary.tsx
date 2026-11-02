import { Component, type ErrorInfo, type ReactNode } from 'react';
import { Button, Result } from 'antd';

export interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
}

/**
 * Captura errores de render y muestra una pantalla de recuperación.
 *
 * Es un componente de clase porque React solo ofrece `componentDidCatch` /
 * `getDerivedStateFromError` en clases: no existe equivalente con hooks.
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): ErrorBoundaryState {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error('Error no controlado en la UI', error, info);
  }

  private handleReload = (): void => {
    window.location.reload();
  };

  render(): ReactNode {
    if (this.state.hasError) {
      return (
        <Result
          status="error"
          title="Algo ha ido mal"
          subTitle="Se ha producido un error inesperado. Recarga la página para continuar."
          extra={
            <Button type="primary" onClick={this.handleReload}>
              Recargar
            </Button>
          }
        />
      );
    }
    return this.props.children;
  }
}

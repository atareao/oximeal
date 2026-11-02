import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BootFallback } from './BootFallback';

describe('BootFallback', () => {
  it('muestra el spinner de arranque con el texto por defecto', () => {
    render(<BootFallback />);

    expect(screen.getByTestId('boot-fallback')).toBeInTheDocument();
    expect(screen.getByRole('status')).toBeInTheDocument();
    expect(screen.getByText(/Cargando OxiMeal/)).toBeInTheDocument();
  });
});

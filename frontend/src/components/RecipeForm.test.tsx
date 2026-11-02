import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App as AntdApp } from 'antd';
import recipeFormCss from './RecipeForm.css?raw';
import type { RecipeInput } from '../api/types';
import { RecipeForm } from './RecipeForm';

function installMatchMedia(matches = true) {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }));
}

function renderForm(overrides: Partial<Parameters<typeof RecipeForm>[0]> = {}) {
  const onSubmit = vi.fn();
  const onClose = vi.fn();
  render(
    <AntdApp>
      <RecipeForm open initial={null} onClose={onClose} onSubmit={onSubmit} {...overrides} />
    </AntdApp>,
  );
  return { onSubmit, onClose };
}

describe('RecipeForm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    installMatchMedia();
  });

  it('no envía si el nombre está vacío y muestra el error de validación', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm();

    await user.click(screen.getByRole('button', { name: /guardar/i }));

    expect(await screen.findByText('Escribe el nombre de la receta')).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('exige al menos un ingrediente', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm();

    await user.click(await screen.findByRole('button', { name: /eliminar ingrediente 1/i }));
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    expect(await screen.findByText(/añade al menos un ingrediente/i)).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('envía los datos válidos normalizados', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm();

    await user.type(screen.getByLabelText('Nombre'), 'Tortilla');
    await user.type(screen.getByLabelText('Nombre del ingrediente 1'), 'Patata');
    await user.type(screen.getByLabelText('Cantidad del ingrediente 1'), '2');
    await user.type(screen.getByLabelText('Unidad del ingrediente 1'), 'ud');
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining<Partial<RecipeInput>>({
        name: 'Tortilla',
        category: 'main',
        ingredients: [expect.objectContaining({ name: 'Patata', amount: 2, unit: 'ud' })],
      }),
    );
  });

  it('el botón de guardar cumple el objetivo táctil de 48px', async () => {
    renderForm();

    const submit = await screen.findByRole('button', { name: /guardar/i });
    expect(submit).toHaveClass('recipe-form__submit');

    expect(recipeFormCss).toMatch(/\.recipe-form__submit\s*\{[^}]*min-height:\s*48px/);
  });
});

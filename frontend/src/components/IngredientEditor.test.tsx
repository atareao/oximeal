import { useState } from 'react';
import type { ComponentProps } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App as AntdApp } from 'antd';
import { IngredientEditor } from './IngredientEditor';
import { parseAmount } from '../lib/ingredients';

function renderEditor(overrides: Partial<ComponentProps<typeof IngredientEditor>> = {}) {
  const props: ComponentProps<typeof IngredientEditor> = {
    ingredients: [{ name: 'Patata', amount: 500, unit: 'g' }],
    factor: 1,
    showScaled: false,
    editable: true,
    saving: false,
    editing: false,
    onStartEdit: vi.fn(),
    onCancel: vi.fn(),
    onSave: vi.fn(),
    ...overrides,
  };
  render(
    <AntdApp>
      <IngredientEditor {...props} />
    </AntdApp>,
  );
  return props;
}

describe('parseAmount', () => {
  it('acepta enteros, punto y coma decimal', () => {
    expect(parseAmount('40')).toBe(40);
    expect(parseAmount('2.5')).toBe(2.5);
    expect(parseAmount('1,5')).toBe(1.5);
  });

  it('trata vacío, texto y no finitos como inválidos', () => {
    expect(parseAmount('')).toBeNull();
    expect(parseAmount('   ')).toBeNull();
    expect(parseAmount('abc')).toBeNull();
    expect(parseAmount('1/2')).toBeNull();
    expect(parseAmount('Infinity')).toBeNull();
  });

  it('conserva el signo para poder rechazar negativos aguas arriba', () => {
    expect(parseAmount('-3')).toBe(-3);
    expect(parseAmount('-3,5')).toBe(-3.5);
  });
});

describe('IngredientEditor', () => {
  it('en modo lectura muestra el nombre y la cantidad escalada y permite iniciar la edición', async () => {
    const user = userEvent.setup();
    const onStartEdit = vi.fn();
    renderEditor({
      ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }],
      factor: 0.5,
      showScaled: true,
      onStartEdit,
    });

    expect(screen.getByText('Patata')).toBeInTheDocument();
    expect(screen.getByText(/400\s*g/)).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /corregir ingredientes/i }));
    expect(onStartEdit).toHaveBeenCalledTimes(1);
  });

  it('sin conexión deshabilita las acciones de edición y avisa', async () => {
    const user = userEvent.setup();
    const props = renderEditor({ editing: true, editable: false });

    expect(screen.getByTestId('ingredient-editor-offline')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /añadir ingrediente/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /eliminar ingrediente 1/i })).toBeDisabled();
    const save = screen.getByRole('button', { name: /guardar ingredientes/i });
    expect(save).toBeDisabled();

    await user.click(save);
    expect(props.onSave).not.toHaveBeenCalled();
  });

  it('añade un ingrediente y persiste conservando la categoría del existente', async () => {
    const user = userEvent.setup();
    const props = renderEditor({
      ingredients: [{ name: 'Patata', amount: 500, unit: 'g', category: 'verduras' }],
      editing: true,
    });

    await user.click(screen.getByRole('button', { name: /añadir ingrediente/i }));
    await user.type(screen.getByLabelText('Nombre del ingrediente 2'), 'Cebolla');
    await user.type(screen.getByLabelText('Cantidad del ingrediente 2'), '1.5');
    await user.type(screen.getByLabelText('Unidad del ingrediente 2'), 'ud');
    await user.click(screen.getByRole('button', { name: /guardar ingredientes/i }));

    expect(props.onSave).toHaveBeenCalledWith([
      { name: 'Patata', amount: 500, unit: 'g', category: 'verduras' },
      { name: 'Cebolla', amount: 1.5, unit: 'ud' },
    ]);
  });

  it('quita una fila intermedia sin renumerar las etiquetas de las restantes', async () => {
    const user = userEvent.setup();
    const props = renderEditor({
      ingredients: [
        { name: 'Patata', amount: 500, unit: 'g' },
        { name: 'Sal', amount: 1, unit: 'ud' },
        { name: 'Pimienta', amount: 1, unit: 'ud' },
      ],
      editing: true,
    });

    await user.click(screen.getByRole('button', { name: /eliminar ingrediente 2/i }));

    // «Pimienta» conserva su ordinal 3 pese a quedar en la posición 2.
    expect(screen.getByLabelText('Nombre del ingrediente 3')).toHaveValue('Pimienta');

    await user.click(screen.getByRole('button', { name: /guardar ingredientes/i }));
    expect(props.onSave).toHaveBeenCalledWith([
      { name: 'Patata', amount: 500, unit: 'g' },
      { name: 'Pimienta', amount: 1, unit: 'ud' },
    ]);
  });

  it('no guarda si el nombre está vacío', async () => {
    const user = userEvent.setup();
    const props = renderEditor({ editing: true });

    await user.clear(screen.getByLabelText('Nombre del ingrediente 1'));
    await user.click(screen.getByRole('button', { name: /guardar ingredientes/i }));

    expect(
      await screen.findByText(/el nombre del ingrediente no puede estar vacío/i),
    ).toBeInTheDocument();
    expect(props.onSave).not.toHaveBeenCalled();
  });

  it('no guarda si la cantidad es negativa', async () => {
    const user = userEvent.setup();
    const props = renderEditor({ editing: true });

    fireEvent.change(screen.getByLabelText('Cantidad del ingrediente 1'), {
      target: { value: '-3' },
    });
    await user.click(screen.getByRole('button', { name: /guardar ingredientes/i }));

    expect(await screen.findByText(/la cantidad no puede ser negativa/i)).toBeInTheDocument();
    expect(props.onSave).not.toHaveBeenCalled();
  });

  it('gestiona el foco: al editar enfoca el primer campo y al cancelar vuelve al botón', async () => {
    const user = userEvent.setup();
    function Harness() {
      const [editing, setEditing] = useState(false);
      return (
        <IngredientEditor
          ingredients={[{ name: 'Patata', amount: 500, unit: 'g' }]}
          factor={1}
          showScaled={false}
          editable
          saving={false}
          editing={editing}
          onStartEdit={() => setEditing(true)}
          onCancel={() => setEditing(false)}
          onSave={() => setEditing(false)}
        />
      );
    }
    render(
      <AntdApp>
        <Harness />
      </AntdApp>,
    );

    await user.click(screen.getByRole('button', { name: /corregir ingredientes/i }));
    expect(screen.getByLabelText('Nombre del ingrediente 1')).toHaveFocus();

    await user.click(screen.getByRole('button', { name: /^cancelar$/i }));
    // Al volver a la vista de lectura el botón se remonta: se re-consulta el nodo.
    expect(screen.getByRole('button', { name: /corregir ingredientes/i })).toHaveFocus();
  });
});

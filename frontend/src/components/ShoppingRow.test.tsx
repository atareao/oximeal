import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import type { ShoppingItem } from '../api/types';
import { ShoppingRow } from './ShoppingRow';

function item(overrides: Partial<ShoppingItem> = {}): ShoppingItem {
  return {
    id: '1',
    name: 'Leche',
    amount: 1,
    unit: 'ud',
    category: 'Refrigerados',
    checked: false,
    updated_at: '2026-11-01T10:00:00Z',
    ...overrides,
  };
}

describe('ShoppingRow', () => {
  it('muestra la etiqueta "Pendiente" en un ítem sin sincronizar', () => {
    render(<ShoppingRow item={item({ pending: true })} onToggle={vi.fn()} />);

    const tag = screen.getByTestId('item-pending');
    expect(tag).toHaveTextContent('Pendiente');
    expect(tag).toHaveAccessibleName('Pendiente de sincronizar');
  });

  it('no muestra la etiqueta en un ítem sincronizado', () => {
    render(<ShoppingRow item={item()} onToggle={vi.fn()} />);

    expect(screen.queryByTestId('item-pending')).not.toBeInTheDocument();
  });
});

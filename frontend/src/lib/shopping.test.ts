import { describe, expect, it } from 'vitest';
import type { ShoppingItem } from '../api/types';
import { categoryEmoji, formatAmount, sortShoppingItems } from './shopping';

function item(overrides: Partial<ShoppingItem> & Pick<ShoppingItem, 'id'>): ShoppingItem {
  return {
    name: overrides.id,
    amount: 1,
    unit: 'ud',
    category: 'Otros',
    checked: false,
    updated_at: '2026-11-01T10:00:00Z',
    ...overrides,
  };
}

describe('sortShoppingItems', () => {
  it('coloca los comprados al final conservando el orden de pendientes', () => {
    const items = [
      item({ id: 'a', checked: true, updated_at: '2026-11-01T10:00:00Z' }),
      item({ id: 'b', checked: false, updated_at: '2026-11-01T11:00:00Z' }),
      item({ id: 'c', checked: false, updated_at: '2026-11-01T09:00:00Z' }),
    ];

    expect(sortShoppingItems(items).map((entry) => entry.id)).toEqual(['c', 'b', 'a']);
  });

  it('ordena los comprados por updated_at ascendente', () => {
    const items = [
      item({ id: 'a', checked: true, updated_at: '2026-11-01T12:00:00Z' }),
      item({ id: 'b', checked: true, updated_at: '2026-11-01T10:00:00Z' }),
    ];

    expect(sortShoppingItems(items).map((entry) => entry.id)).toEqual(['b', 'a']);
  });

  it('no muta el array original', () => {
    const items = [item({ id: 'a', checked: true }), item({ id: 'b' })];
    const copy = [...items];
    sortShoppingItems(items);
    expect(items).toEqual(copy);
  });
});

describe('formatAmount', () => {
  it('omite decimales innecesarios', () => {
    expect(formatAmount(2)).toBe('2');
    expect(formatAmount(1.5)).toBe('1,5');
    expect(formatAmount(0.25)).toBe('0,25');
  });
});

describe('categoryEmoji', () => {
  it('devuelve el emoji canónico de cada pasillo', () => {
    expect(categoryEmoji('Verdulería')).toBe('🥦');
    expect(categoryEmoji('Pescadería')).toBe('🐟');
    expect(categoryEmoji('Carnicería')).toBe('🥩');
    expect(categoryEmoji('Refrigerados')).toBe('❄️');
    expect(categoryEmoji('Panadería')).toBe('🍞');
    expect(categoryEmoji('Bebidas')).toBe('🥤');
    expect(categoryEmoji('Despensa')).toBe('🥫');
    expect(categoryEmoji('Otros')).toBe('🛒');
  });

  it('usa el fallback para categorías desconocidas', () => {
    expect(categoryEmoji('Congelados')).toBe('🛒');
  });
});

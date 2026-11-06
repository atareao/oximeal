import { Tag } from 'antd';
import type { ShoppingItem } from '../api/types';
import { formatAmount, SHOPPING_CATEGORIES } from '../lib/shopping';
import './ShoppingRow.css';

export interface ShoppingRowProps {
  item: ShoppingItem;
  onToggle: (item: ShoppingItem) => void;
  onCategoryChange?: (item: ShoppingItem, category: string) => void;
}

/** Fila táctil de un ítem de la compra con checkbox nativo y selector de pasillo. */
export function ShoppingRow({ item, onToggle, onCategoryChange }: ShoppingRowProps) {
  return (
    <li>
      <div className={`shopping-row${item.checked ? ' shopping-row--checked' : ''}`}>
        <label className="shopping-row__main">
          <input
            type="checkbox"
            className="shopping-row__check"
            checked={item.checked}
            onChange={() => onToggle(item)}
          />
          <span className="shopping-row__name">{item.name}</span>
          {item.pending && (
            <Tag
              color="orange"
              className="shopping-row__pending"
              data-testid="item-pending"
              aria-label="Pendiente de sincronizar"
            >
              Pendiente
            </Tag>
          )}
          <span className="shopping-row__amount">
            {formatAmount(item.amount)} {item.unit}
          </span>
        </label>
        <select
          className="shopping-row__category"
          aria-label={`Categoría de ${item.name}`}
          value={item.category}
          onChange={(event) => onCategoryChange?.(item, event.target.value)}
        >
          {SHOPPING_CATEGORIES.map((category) => (
            <option key={category} value={category}>
              {category}
            </option>
          ))}
        </select>
      </div>
    </li>
  );
}

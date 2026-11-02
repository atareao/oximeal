import type { ShoppingItem } from '../api/types';
import { formatAmount } from '../lib/shopping';
import './ShoppingRow.css';

export interface ShoppingRowProps {
  item: ShoppingItem;
  onToggle: (item: ShoppingItem) => void;
}

/** Fila táctil de un ítem de la compra con checkbox nativo. */
export function ShoppingRow({ item, onToggle }: ShoppingRowProps) {
  return (
    <li>
      <label className={`shopping-row${item.checked ? ' shopping-row--checked' : ''}`}>
        <input
          type="checkbox"
          className="shopping-row__check"
          checked={item.checked}
          onChange={() => onToggle(item)}
        />
        <span className="shopping-row__name">{item.name}</span>
        <span className="shopping-row__amount">
          {formatAmount(item.amount)} {item.unit}
        </span>
      </label>
    </li>
  );
}

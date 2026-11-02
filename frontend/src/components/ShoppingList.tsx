import { useId } from 'react';
import type { ShoppingItem, ShoppingListGroup } from '../api/types';
import { categoryEmoji, sortShoppingItems } from '../lib/shopping';
import { ShoppingRow } from './ShoppingRow';
import './ShoppingList.css';

export interface ShoppingListProps {
  groups: readonly ShoppingListGroup[];
  onToggle: (item: ShoppingItem) => void;
}

/** Lista de la compra agrupada por pasillo, respetando el orden del backend. */
export function ShoppingList({ groups, onToggle }: ShoppingListProps) {
  const baseId = useId();

  return (
    <div className="shopping-list">
      {groups.map((group, index) => {
        const headingId = `${baseId}-category-${index}`;
        return (
          <section
            key={group.category}
            className="shopping-list__group"
            aria-labelledby={headingId}
          >
            <h2 id={headingId} className="shopping-list__heading">
              <span className="shopping-list__emoji" aria-hidden="true">
                {categoryEmoji(group.category)}
              </span>
              <span>{group.category}</span>
            </h2>
            <ul className="shopping-list__items">
              {sortShoppingItems(group.items).map((item) => (
                <ShoppingRow key={item.id} item={item} onToggle={onToggle} />
              ))}
            </ul>
          </section>
        );
      })}
    </div>
  );
}

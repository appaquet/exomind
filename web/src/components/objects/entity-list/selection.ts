import { exocore } from 'exocore';
import * as _ from 'lodash';

export class Selection {
    items: SelectedItem[];

    constructor(items?: SelectedItem[]) {
        this.items = items ?? [];
    }

    contains(needle: SelectedItem): boolean {
        for (const item of this.items) {
            if (item.equals(needle)) {
                return true;
            }
        }

        return false;
    }

    copy(): Selection {
        return new Selection(_.clone(this.items));
    }

    cleared(): Selection {
        return new Selection();
    }

    isEmpty(): boolean {
        return this.length() == 0;
    }

    length(): number {
        return this.items.length;
    }

    withItem(item: SelectedItem): Selection {
        const newItems = [];
        newItems.push(...this.items);
        newItems.push(item);
        return new Selection(newItems);
    }

    withoutItem(item: SelectedItem): Selection {
        const newItems = this.items.flatMap((oneItem) => {
            if (oneItem.equals(item)) {
                return [];
            } else {
                return [oneItem];
            }
        });

        return new Selection(newItems);
    }
}

export class SelectedItem {
    private _entity?: exocore.store.IEntity;
    private _entityId?: string;
    private _trait?: exocore.store.ITrait;
    private _traitId?: string;

    static fromEntityId(entityId: string): SelectedItem {
        const item = new SelectedItem();
        item._entityId = entityId;
        return item;
    }

    static fromEntityTraitId(entityId: string, traitId: string): SelectedItem {
        const item = new SelectedItem();
        item._entityId = entityId;
        item._traitId = traitId;
        return item;
    }

    static fromEntity(entity: exocore.store.IEntity): SelectedItem {
        const item = new SelectedItem();
        item._entity = entity;
        return item;
    }

    static fromEntityTrait(entity: exocore.store.IEntity, trait: exocore.store.ITrait): SelectedItem {
        const item = new SelectedItem();
        item._entity = entity;
        item._trait = trait;
        return item;
    }

    equals(other: SelectedItem): boolean {
        return other.entityId == this.entityId && other.traitId == this.traitId;
    }

    get entityId(): string {
        if (this._entity) {
            return this._entity.id;
        } else {
            return this._entityId;
        }
    }

    get traitId(): string {
        if (this._trait) {
            return this._trait.id;
        } else {
            return this._traitId;
        }
    }

}
import _ from 'lodash';
import { EntityTrait, EntityTraits } from '../../../utils/entities';

export class Selection {
    items: SelectedItem[];

    private forceMulti: boolean;

    constructor(items?: SelectedItem[] | SelectedItem | undefined, forceMulti = false) {
        if (!items) {
            items = [];
        } else if (!Array.isArray(items)) {
            items = [items];
        }

        this.items = items;
        this.forceMulti = forceMulti;
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
        return new Selection(_.clone(this.items), this.forceMulti);
    }

    cleared(): Selection {
        return new Selection();
    }

    get isEmpty(): boolean {
        return this.length == 0;
    }

    get length(): number {
        return this.items.length;
    }

    get isMulti(): boolean {
        return this.forceMulti || this.length > 1;
    }

    withItem(item: SelectedItem): Selection {
        const newItems = [];
        newItems.push(...this.items);
        newItems.push(item);
        return new Selection(newItems, this.forceMulti);
    }

    withoutItem(item: SelectedItem): Selection {
        const newItems = this.items.flatMap((oneItem) => {
            if (oneItem.equals(item)) {
                return [];
            } else {
                return [oneItem];
            }
        });

        return new Selection(newItems, this.forceMulti);
    }

    withForceMulti(): Selection {
        this.forceMulti = true;
        return this;
    }

    filterSelectedEntities(entities: EntityTraits[] | null): EntityTraits[] {
        if (!entities) {
            return [];
        }

        const indexedEntities = entities.reduce((acc: { [key: string]: EntityTraits; }, entity) => {
            acc[entity.id] = entity;
            return acc;
        }, {});

        const selectedEntities = this.items.flatMap((sel) => {
            const entity = indexedEntities[sel.entityId];
            if (!entity) {
                return [];
            }

            return [entity];
        });

        return selectedEntities;
    }
}

export class SelectedItem {
    private _entity?: EntityTraits;
    private _entityId?: string;
    private _trait?: EntityTrait<unknown>;
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

    static fromEntity(entity: EntityTraits): SelectedItem {
        const item = new SelectedItem();
        item._entity = entity;
        return item;
    }

    static fromEntityTrait(entity: EntityTraits, trait: EntityTrait<unknown>): SelectedItem {
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
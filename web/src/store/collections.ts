import { Exocore, QueryBuilder } from "exocore";
import { exomind } from "../protos";
import { EntityTrait, EntityTraits, TraitIcon } from "./entities";


export class Collections {
    public static default: Collections = new Collections();

    private cache: Map<string, EntityTrait<exomind.base.ICollection>> = new Map();

    async getParents(entity: EntityTraits, lineage?: Set<string>): Promise<Parents> {
        const parents = new Parents();

        const colChildren = entity.traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild);
        for (const colChild of colChildren) {
            const parentId = colChild.message.collection.entityId;
            if (parentId == 'favorites') {
                continue
            }

            if ((lineage?.has(parentId) ?? false) || parents.isFetched(parentId)) {
                continue;
            }

            const collection = await this.getCollection(parentId);
            if (!collection) {
                continue;
            }

            const col: ICollection = {
                entityId: collection.et.id,
                icon: collection.icon,
                name: collection.displayName,
                collection: collection.message,
            };
            parents.add(col);

            const thisLineage = lineage || new Set();
            thisLineage.add(parentId);
            const grandParents = await this.getParents(collection.et, thisLineage);
            col.parents = grandParents.get();
        }

        return parents;
    }

    async getCollection(id: string): Promise<EntityTrait<exomind.base.ICollection> | null> {
        // TODO: Should batch and use watched query

        if (this.cache.has(id)) {
            return this.cache.get(id);
        }

        const result = await Exocore.store.query(QueryBuilder.withIds(id).build());
        for (const entity of result.entities) {
            const et = new EntityTraits(entity.entity);
            const col = et.traitOfType<exomind.base.ICollection>(exomind.base.Collection);
            this.cache.set(et.id, col);

            return col;
        }
    }
}

export interface ICollection {
    entityId: string,
    icon: TraitIcon,
    name: string,
    collection: exomind.base.ICollection,
    parents?: ICollection[],
}

export class Parents {
    parents: Map<string, ICollection> = new Map();

    add(col: ICollection): void {
        this.parents.set(col.entityId, col);
    }

    get(): ICollection[] {
        return Array.from(this.parents.values());
    }

    isFetched(id: string): boolean {
        return this.parents.has(id);
    }
}
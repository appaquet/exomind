import { Exocore, QueryBuilder } from "exocore";
import { exomind } from "../protos";
import { EntityTrait, EntityTraits } from "./entities";


export class Collections {
    public static default: Collections = new Collections();

    private cache: Map<string, EntityTrait<exomind.base.ICollection>> = new Map();

    async getParents(entity: EntityTraits, parents: Parents = null): Promise<Parents> {
        if (!parents) {
            parents = new Parents();
        }

        const colChildren = entity.traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild);
        for (const colChild of colChildren) {
            const parentId = colChild.message.collection.entityId;
            if (parents.contains(parentId)) {
                continue;
            }

            const collection = await this.getCollection(parentId);
            if (!collection) {
                continue;
            }

            // TODO: Should attach grand parents to child
            parents.add(collection);

            const grandParents = await this.getParents(collection.et, parents);
            parents.merge(grandParents);
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

export class Parents {
    parents: Map<string, EntityTrait<exomind.base.ICollection>> = new Map();

    merge(other: Parents): void {
        for (const id in other.parents) {
            this.parents.set(id, other.parents.get(id));
        }
    }

    add(col: EntityTrait<exomind.base.ICollection>): void {
        this.parents.set(col.et.id, col);
    }

    contains(id: string): boolean {
        return this.parents.has(id);
    }

    get(): EntityTrait<exomind.base.ICollection>[] {
        // TODO: Should be ordered by most important first
        return Array.from(this.parents.values());
    }
}
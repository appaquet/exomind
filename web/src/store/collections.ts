import { Exocore, QueryBuilder } from "exocore";
import { observable, ObservableMap } from "mobx";
import { cpuUsage } from "node:process";
import { exomind } from "../protos";
import { EntityTrait, EntityTraits, TraitIcon } from "./entities";


export class Collections {
    // TODO: Should be part of the stores
    public static default: Collections = new Collections();

    private cache: Map<string, EntityTrait<exomind.base.ICollection>> = new Map();
    private _cache: ObservableMap<string, EntityTrait<exomind.base.ICollection>> = observable.map();

    async getParents(entity: EntityTraits, lineage?: Set<string>): Promise<Parents> {
        // TODO: Should contain the lineage length so that we can choose which parents to show first
        const parents = new Parents();

        const colChildren = entity.traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild);
        for (const colChild of colChildren) {
            const parentId = colChild.message.collection.entityId;
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

            const thisLineage = new Set(lineage);
            thisLineage.add(parentId);
            const grandParents = await this.getParents(collection.et, thisLineage);

            col.parents = grandParents.get();
            sortCollections(col.parents);
            if (col.parents.length > 0) {
                col.minParent = col.parents[0];
            }
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
    entityId: string;
    icon: TraitIcon;
    name: string;
    collection: exomind.base.ICollection;
    parents?: ICollection[];

    minParent?: ICollection;
}

export class Parents {
    parents: Map<string, ICollection> = new Map();

    add(col: ICollection): void {
        this.parents.set(col.entityId, col);
    }

    get(): ICollection[] {
        const parents = Array.from(this.parents.values());
        sortCollections(parents);
        return parents;
    }

    isFetched(id: string): boolean {
        return this.parents.has(id);
    }
}

function minLineage(cols: ICollection[], init = 0): [number, ICollection | null] {
    if (cols.length == 0) {
        return [init, null];
    }

    let minLength = 0;
    let minCol: ICollection = null;
    for (const col of cols) {
        const [length, ] = minLineage(col.parents, init + 1);
        if (!minCol || length < minLength) {
            minLength = length;
            minCol = col;
        }
    }

    return [init + minLength, minCol];
}

function sortCollections(cols: ICollection[]): void {
    if (cols.length <= 1) {
        return;
    }

    cols.sort((a, b) => {
        const [aLin,] = minLineage(a.parents);
        const [bLin,] = minLineage(b.parents);
        if (aLin == bLin) {
            return a.name.localeCompare(b.name);
        } else {
            return aLin - bLin;
        }
    });
}
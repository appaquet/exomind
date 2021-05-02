import { Exocore, QueryBuilder } from "exocore";
import { makeAutoObservable, observable, ObservableMap, ObservableSet, runInAction } from "mobx";
import { exomind } from "../protos";
import { EntityTrait, EntityTraits, TraitIcon } from "./entities";

export class CollectionStore {
    private colResults: Map<string, Promise<EntityTrait<exomind.base.ICollection>>> = new Map();
    private parentCache: ObservableMap<string, Parents> = observable.map();
    private parentProcessing: ObservableSet<string> = observable.set();

    constructor() {
        makeAutoObservable(this);
    }

    getParents(entity: EntityTraits): Parents | null {
        const cacheKey = `${entity.id}${entity.entity.lastOperationId}`;

        if (this.parentCache.has(cacheKey)) {
            return this.parentCache.get(cacheKey);
        }

        if (this.parentProcessing.has(cacheKey) || this.parentProcessing.size > 2) {
            return null;
        }

        runInAction(() => {
            this.parentProcessing.add(cacheKey);
        });
        this.getParentsAsync(entity).then((parents) => {
            runInAction(() => {
                this.parentCache.set(cacheKey, parents);
                this.parentProcessing.delete(cacheKey);
            });
        })

        return null;
    }

    async getParentsAsync(entity: EntityTraits, lineage?: Set<string>): Promise<Parents> {
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
            const grandParents = await this.getParentsAsync(collection.et, thisLineage);

            col.parents = grandParents.get();
            sortCollections(col.parents);
            if (col.parents.length > 0) {
                col.minParent = col.parents[0];
            }
        }

        return parents;
    }

    async getCollection(id: string): Promise<EntityTrait<exomind.base.ICollection> | null> {
        // TODO: Should batch queries

        if (this.colResults.has(id)) {
            return this.colResults.get(id);
        }
        console.log('getting for collection', id);

        let firstResult = true;
        const colPromise = new Promise<EntityTrait<exomind.base.ICollection>>((resolve) => {
            const query = Exocore.store.watchedQuery(QueryBuilder.withIds(id).build());
            query.onChange((results) => {
                for (const entity of results.entities) {
                    const et = new EntityTraits(entity.entity);
                    const col = et.traitOfType<exomind.base.ICollection>(exomind.base.Collection);

                    if (firstResult) {
                        resolve(col);
                        firstResult = false;
                        return;
                    } else {
                        runInAction(() => {
                            this.colResults.set(id, Promise.resolve(col));
                            // this.parentCache.clear();
                        });
                    }
                }

                resolve(null);
            });
        });
        this.colResults.set(id, colPromise);

        return await colPromise;
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
        const [length,] = minLineage(col.parents, init + 1);
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

export function flattenHierarchy(collection: ICollection): ICollection[] {
    const out = [];

    while (collection != null) {
        if (collection.entityId == 'favorites') {
            break;
        }

        out.push(collection);

        if (!collection.minParent) {
            break;
        }
        collection = collection.minParent;
    }

    return out.reverse();
}
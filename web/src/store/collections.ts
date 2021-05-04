import { Exocore, QueryBuilder } from "exocore";
import { memoize } from "lodash";
import { observable, ObservableMap, ObservableSet, runInAction } from "mobx";
import { exomind } from "../protos";
import { EntityTrait, EntityTraits, TraitIcon } from "./entities";

const MaxSyncParallelism = 10;

export class CollectionStore {
    private colResults: Map<string, Promise<EntityTrait<exomind.base.ICollection>>> = new Map();
    private entityParents: ObservableMap<string, Parents> = observable.map();
    private entityProcessing: ObservableSet<string> = observable.set();

    getEntityParents(entity: EntityTraits): Parents | null {
        const cacheKey = `${entity.id}${entity.entity.lastOperationId}`;

        const parents = this.entityParents.get(cacheKey)
        if (parents) {
            return parents;
        }

        // make sure we aren't already fetching for this parent, and that we aren't fetching too many at same time
        if (this.entityProcessing.has(cacheKey) || this.entityProcessing.size > MaxSyncParallelism) {
            return null;
        }

        // prevent notifying components that call `getParents` in their render
        setTimeout(() => {
            runInAction(() => {
                this.entityProcessing.add(cacheKey);
            });

            this.getEntityParentsAsync(entity).then((parents) => {
                runInAction(() => {
                    this.entityParents.set(cacheKey, parents);
                    this.entityProcessing.delete(cacheKey);
                });
            })
        });

        return null;
    }

    async getEntityParentsAsync(entity: EntityTraits, lineage?: Set<string>): Promise<Parents> {
        const parents = new Parents();

        const colChildren = entity.traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild);
        for (const colChild of colChildren) {
            const parentId = colChild.message.collection.entityId;
            if (parents.isFetched(parentId) || (lineage?.has(parentId) ?? false)) {
                // the collection got already fetched, either because we have it twice in our parents, or because it's part of the lineage already
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
            const grandParents = await this.getEntityParentsAsync(collection.et, thisLineage);

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

                            // invalidate cache for all entities for which we fetched parents in which we are
                            for (const parentId of this.entityParents.keys()) {
                                const parent = this.entityParents.get(parentId);
                                const ids = parent.allIds();
                                if (ids.has(id)) {
                                    this.entityParents.delete(parentId);
                                }
                            }
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

    allIds= memoize((): Set<string> => {
        const ids = new Set<string>();

        const getParents = (col: ICollection) => {
            ids.add(col.entityId);
            for (const parent of col.parents) {
                if (!ids.has(parent.entityId)) {
                    ids.add(parent.entityId);
                    getParents(parent);
                }
            }
        };
        for (const parent of this.parents.values()) {
            getParents(parent);
        }

        return ids;
    });

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
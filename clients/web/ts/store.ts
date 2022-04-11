import { Exocore, exocore } from ".";
import { ExocoreClient, getModule, WatchedQuery } from "./wasm";

export class Store {
    wasmClient: ExocoreClient;
    statusChangeCallback: () => void;

    constructor(wasmClient: ExocoreClient) {
        this.wasmClient = wasmClient;
    }

    async mutate(mutation: exocore.store.IMutationRequest): Promise<exocore.store.MutationResult> {
        const encoded = exocore.store.MutationRequest.encode(mutation).finish();

        let resultsData: Uint8Array = await this.wasmClient.store_mutate(encoded);
        return exocore.store.MutationResult.decode(resultsData);
    }

    async query(query: exocore.store.IEntityQuery): Promise<exocore.store.EntityResults> {
        const encoded = exocore.store.EntityQuery.encode(query).finish();

        const resultsData: Uint8Array = await this.wasmClient.store_query(encoded);
        return exocore.store.EntityResults.decode(resultsData);
    }

    watchedQuery(query: exocore.store.IEntityQuery): WatchedQueryWrapper {
        const encoded = exocore.store.EntityQuery.encode(query).finish();

        return new WatchedQueryWrapper(this.wasmClient.store_watched_query(encoded));
    }

    generateId(prefix?: string): string {
        return getModule().generate_id(prefix);
    }

    httpEndpoints(): Array<string> {
        return this.wasmClient.store_http_endpoints();
    }
}

export class WatchedQueryWrapper {
    inner: WatchedQuery;

    constructor(inner: WatchedQuery) {
        this.inner = inner;
    }

    onChange(cb: (results: exocore.store.EntityResults) => void): WatchedQueryWrapper {
        this.inner.on_change(() => {
            try {
                const resultsData: Uint8Array = this.inner.get();
                const res = exocore.store.EntityResults.decode(resultsData);
                cb(res);
            } catch (e) {
                console.log(`Failed to update watched query result: ${e}`);
            }
        })
        return this;
    }

    free(): void {
        this.inner.free();
    }
}

export class MutationBuilder {
    entityId: string;
    request: exocore.store.MutationRequest;

    constructor(entityId: string) {
        this.entityId = entityId;
        this.request = new exocore.store.MutationRequest();
    }

    static createEntity(entityId?: string): MutationBuilder {
        if (!entityId) {
            entityId = getModule().generate_id('et')
        }

        return new MutationBuilder(entityId);
    }

    static updateEntity(entityId: string): MutationBuilder {
        return new MutationBuilder(entityId);
    }

    static deleteEntity(entityId: string): exocore.store.MutationRequest {
        return new exocore.store.MutationRequest({
            mutations: [
                new exocore.store.EntityMutation({
                    entityId: entityId,
                    deleteEntity: new exocore.store.DeleteEntityMutation({ entityId: entityId }),
                }),
            ],
        });
    }

    andUpdateEntity(entityId: string): MutationBuilder {
        this.entityId = entityId;

        return this;
    }

    andCreateEntity(entityId: string = null): MutationBuilder {
        if (!entityId) {
            entityId = getModule().generate_id('et')
        }

        this.entityId = entityId;

        return this;
    }

    putTrait(message: any, traitId: string = null): MutationBuilder {
        if (!traitId) {
            traitId = getModule().generate_id('trt');
        }

        this.request.mutations.push(new exocore.store.EntityMutation({
            entityId: this.entityId,
            putTrait: new exocore.store.PutTraitMutation({
                trait: new exocore.store.Trait({
                    id: traitId,
                    message: Exocore.registry.packToAny(message),
                })
            })
        }));

        return this;
    }

    deleteTrait(traitId: string): MutationBuilder {
        this.request.mutations.push(new exocore.store.EntityMutation({
            entityId: this.entityId,
            deleteTrait: new exocore.store.DeleteTraitMutation({
                traitId: traitId,
            })
        }));

        return this;
    }

    returnEntities(): MutationBuilder {
        this.request.returnEntities = true;

        return this;
    }

    build(): exocore.store.MutationRequest {
        return this.request;
    }
}

export class QueryBuilder {
    query: exocore.store.EntityQuery;

    constructor() {
        this.query = new exocore.store.EntityQuery();
    }

    static withTrait(message: any, traitQuery?: exocore.store.ITraitQuery): QueryBuilder {
        let builder = new QueryBuilder();
        builder.query.trait = new exocore.store.TraitPredicate({
            traitName: Exocore.registry.messageFullName(message),
            query: traitQuery,
        });
        return builder;
    }

    static matches(query: string): QueryBuilder {
        let builder = new QueryBuilder();
        builder.query.match = new exocore.store.MatchPredicate({
            query: query
        });
        return builder;
    }

    static withIds(ids: string | string[]): QueryBuilder {
        if (!Array.isArray(ids)) {
            ids = [ids];
        }

        let builder = new QueryBuilder();
        builder.query.ids = new exocore.store.IdsPredicate({
            ids: ids,
        });
        return builder;
    }

    static fromQueryString(query: string): QueryBuilder {
        let builder = new QueryBuilder();
        builder.query.queryString = new exocore.store.QueryStringPredicate({
            query: query,
        });
        return builder;
    }

    static all(): QueryBuilder {
        let builder = new QueryBuilder();
        builder.query.all = new exocore.store.AllPredicate();
        return builder;
    }

    count(count: number): QueryBuilder {
        this.query.paging = new exocore.store.Paging({
            count: count,
        });
        return this;
    }

    project(...projection: exocore.store.IProjection[]): QueryBuilder {
        this.query.projections = this.query.projections.concat(projection);
        return this;
    }

    orderByField(field: string, ascending: boolean): QueryBuilder {
        this.query.ordering = new exocore.store.Ordering({
            ascending: ascending === true,
            field: field,
        });
        return this;
    }

    orderByOperationIds(ascending: boolean): QueryBuilder {
        this.query.ordering = new exocore.store.Ordering({
            ascending: ascending === true,
            operationId: true,
        });
        return this;
    }

    includeDeleted(): QueryBuilder {
        this.query.includeDeleted = true;
        return this;
    }

    build(): exocore.store.IEntityQuery {
        return this.query;
    }
}

export class TraitQueryBuilder {
    query: exocore.store.TraitQuery

    constructor() {
        this.query = new exocore.store.TraitQuery();
    }

    static refersTo(field: string, entityId: string, traitId?: string): TraitQueryBuilder {
        let builder = new TraitQueryBuilder();
        builder.query.reference = new exocore.store.TraitFieldReferencePredicate({
            field: field,
            reference: new exocore.store.Reference({
                entityId: entityId,
                traitId: traitId,
            })
        });

        return builder;
    }

    static matches(query: string): TraitQueryBuilder {
        let builder = new TraitQueryBuilder();
        builder.query.match = new exocore.store.MatchPredicate({
            query: query,
        });

        return builder;
    }

    static fromQueryString(query: string): TraitQueryBuilder {
        let builder = new TraitQueryBuilder();
        builder.query.queryString = new exocore.store.QueryStringPredicate({
            query: query,
        });
        return builder;
    }

    build(): exocore.store.ITraitQuery {
        return this.query;
    }
}
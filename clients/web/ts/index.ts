import * as protos from '../protos';
import { exocore } from '../protos';
export {
    protos,
    exocore,
}

var _exocore_wasm: any = null;

export class Exocore {
    static defaultInstance: ExocoreInstance = null;

    static get initialized(): boolean {
        return Exocore.defaultInstance != null;
    }

    static async ensureLoaded(): Promise<void> {
        if (_exocore_wasm == null) {
            _exocore_wasm = await import('../wasm/exocore_client_web');
        }

        return _exocore_wasm;
    }

    static async initialize(config: object): Promise<ExocoreInstance> {
        const configJson = JSON.stringify(config);``
        const configBytes = new TextEncoder().encode(configJson);

        await Exocore.ensureLoaded();

        let instance: ExocoreInstance;
        const onStatusChange = (status: string) => {
            instance._triggerStatusChange(status)
        }

        const innerClient = new _exocore_wasm.ExocoreClient(configBytes, 'json', onStatusChange);
        instance = new ExocoreInstance(innerClient);

        if (!Exocore.defaultInstance) {
            Exocore.defaultInstance = instance;
        }

        return instance;
    }

    static get cell(): Cell {
        return Exocore.defaultInstance.cell;
    }

    static get store(): Store {
        return Exocore.defaultInstance.store;
    }

    static get registry(): Registry {
        return Exocore.defaultInstance.registry;
    }
}

export class ExocoreInstance {
    wasmClient: any;
    cell: Cell;
    store: Store;
    status: string;
    registry: Registry;
    onChange?: () => void;

    constructor(wasmClient: any) {
        this.wasmClient = wasmClient;
        this.cell = new Cell(wasmClient);
        this.store = new Store(wasmClient);
        this.registry = new Registry();
    }

    _triggerStatusChange(status: string): void {
        this.status = status;
        if (this.onChange) {
            this.onChange();
        }
    }
}

export class Cell {
    wasmClient: any;
    statusChangeCallback: () => void;

    constructor(inner: any) {
        this.wasmClient = inner;
    }

    generateAuthToken(expirationDays?: number): Array<string> {
        return this.wasmClient.cell_generate_auth_token(expirationDays ?? 0);
    }
}

export class Store {
    wasmClient: any;
    statusChangeCallback: () => void;

    constructor(inner: any) {
        this.wasmClient = inner;
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

    watchedQuery(query: exocore.store.IEntityQuery): WatchedQuery {
        const encoded = exocore.store.EntityQuery.encode(query).finish();

        return new WatchedQuery(this.wasmClient.store_watched_query(encoded));
    }

    generateId(prefix?: string): string {
        return _exocore_wasm.generate_id(prefix);
    }

    httpEndpoints(): Array<string> {
        return this.wasmClient.store_http_endpoints();
    }
}

export class WatchedQuery {
    inner: any;

    constructor(inner: any) {
        this.inner = inner;
    }

    onChange(cb: (results: exocore.store.EntityResults) => void): WatchedQuery {
        this.inner.on_change(() => {
            const resultsData: Uint8Array = this.inner.get();
            const res = exocore.store.EntityResults.decode(resultsData);
            cb(res);
        })
        return this;
    }

    free(): void {
        this.inner.free();
    }
}

export class Registry {
    private _registeredMessages: { [name: string]: any } = {};

    registerMessage(message: any, fullName: string): any {
        message.prototype._proto_full_name = fullName;

        this._registeredMessages[fullName] = {
            fullName: fullName,
            message: message,
        }
    }

    messageFullName(message: any): string {
        let fullName = message._proto_full_name;
        if (!fullName && message.prototype) {
            fullName = message.prototype._proto_full_name;
        }

        const info = this._registeredMessages[fullName];
        if (!info) {
            console.error('Tried to get full name for an unregistered message', message);
            throw 'Tried to pack an unregistered message';
        }

        return info.fullName;
    }

    packToAny(message: any): protos.google.protobuf.IAny {
        const info = this._registeredMessages[message._proto_full_name];
        if (!info) {
            console.log('Tried to pack an unregistered message', message);
            throw 'Tried to pack an unregistered message';
        }

        return new protos.google.protobuf.Any({
            type_url: 'type.googleapis.com/' + info.fullName,
            value: info.message.encode(message).finish(),
        })
    }

    unpackAny(any: protos.google.protobuf.IAny): any {
        const fullName = this.canonicalFullName(any.type_url);
        const info = this._registeredMessages[fullName];
        if (!info) {
            console.error('Tried to unpack any any with unregistered type', fullName);
            throw 'Tried to pack an unregistered message';
        }

        return info.message.decode(any.value);
    }

    canonicalFullName(name: string) {
        return name.replace('type.googleapis.com/', '');
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
            entityId = _exocore_wasm.generate_id('et')
        }

        return new MutationBuilder(entityId);
    }

    static updateEntity(entityId: string): MutationBuilder {
        return new MutationBuilder(entityId);
    }

    andUpdateEntity(entityId: string): MutationBuilder {
        this.entityId = entityId;

        return this;
    }

    andCreateEntity(entityId: string = null): MutationBuilder {
        if (!entityId) {
            entityId = _exocore_wasm.generate_id('et')
        }

        this.entityId = entityId;

        return this;
    }

    putTrait(message: any, traitId: string = null): MutationBuilder {
        if (!traitId) {
            traitId = _exocore_wasm.generate_id('trt');
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

    build(): exocore.store.ITraitQuery {
        return this.query;
    }
}

export function toProtoTimestamp(date: Date): protos.google.protobuf.ITimestamp {
    const epoch = date.getTime();
    const seconds = Math.floor(epoch / 1000);

    return new protos.google.protobuf.Timestamp({
        seconds: seconds,
        nanos: (epoch - (seconds * 1000)) * 1000000,
    });
}

export function fromProtoTimestamp(ts: protos.google.protobuf.ITimestamp): Date {
    return new Date((ts.seconds as number) * 1000 + ts.nanos / 1000000);
}

export function matchTrait<T>(trait: exocore.store.ITrait, matchMap: { [fullName: string]: (trait: exocore.store.ITrait, message: any)=>T}): T|null {
    const fullName = Exocore.registry.canonicalFullName(trait.message.type_url);

    if (fullName in matchMap) {
        const message = Exocore.registry.unpackAny(trait.message);
        return matchMap[fullName](trait, message);
    } else {
        return null;
    }
}
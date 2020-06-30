import * as protos from '../js/protos';
import { exocore } from '../js/protos';
export { protos, exocore, };
var _exocore_wasm = null;
export class Exocore {
    static get initialized() {
        return Exocore.defaultInstance != null;
    }
    static async initialize(config) {
        const configJson = JSON.stringify(config);
        ``;
        const configBytes = new TextEncoder().encode(configJson);
        let instance;
        const onStatusChange = (status) => {
            instance._triggerStatusChange(status);
        };
        if (_exocore_wasm != null) {
            const innerClient = new _exocore_wasm.ExocoreClient(configBytes, 'json', onStatusChange);
            instance = new ExocoreInstance(innerClient);
        }
        else {
            _exocore_wasm = await import('../pkg/exocore_client_web');
            console.log("Exocore WASM client loaded");
            const innerClient = new _exocore_wasm.ExocoreClient(configBytes, 'json', onStatusChange);
            instance = new ExocoreInstance(innerClient);
        }
        if (!Exocore.defaultInstance) {
            Exocore.defaultInstance = instance;
        }
        return instance;
    }
    static get store() {
        return Exocore.defaultInstance.store;
    }
    static get registry() {
        return Exocore.defaultInstance.registry;
    }
}
Exocore.defaultInstance = null;
export class ExocoreInstance {
    constructor(wasmClient) {
        this.wasmClient = wasmClient;
        this.store = new Store(wasmClient);
        this.registry = new Registry();
    }
    _triggerStatusChange(status) {
        this.status = status;
        if (this.onChange) {
            this.onChange();
        }
    }
}
export class Store {
    constructor(inner) {
        this.wasmClient = inner;
    }
    async mutate(mutation) {
        const encoded = exocore.index.MutationRequest.encode(mutation).finish();
        let resultsData = await this.wasmClient.mutate(encoded);
        return exocore.index.MutationResult.decode(resultsData);
    }
    async query(query) {
        const encoded = exocore.index.EntityQuery.encode(query).finish();
        const resultsData = await this.wasmClient.query(encoded);
        return exocore.index.EntityResults.decode(resultsData);
    }
    watchedQuery(query) {
        const encoded = exocore.index.EntityQuery.encode(query).finish();
        return new WatchedQuery(this.wasmClient.watched_query(encoded));
    }
    generateId(prefix) {
        return _exocore_wasm.generate_id(prefix);
    }
}
export class WatchedQuery {
    constructor(inner) {
        this.inner = inner;
    }
    onChange(cb) {
        this.inner.on_change(() => {
            const resultsData = this.inner.get();
            const res = exocore.index.EntityResults.decode(resultsData);
            cb(res);
        });
        return this;
    }
    free() {
        this.inner.free();
    }
}
export class Registry {
    constructor() {
        this._registeredMessages = {};
    }
    registerMessage(message, fullName) {
        message.prototype._proto_full_name = fullName;
        this._registeredMessages[fullName] = {
            fullName: fullName,
            message: message,
        };
    }
    messageFullName(message) {
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
    packToAny(message) {
        const info = this._registeredMessages[message._proto_full_name];
        if (!info) {
            console.log('Tried to pack an unregistered message', message);
            throw 'Tried to pack an unregistered message';
        }
        return new protos.google.protobuf.Any({
            type_url: 'type.googleapis.com/' + info.fullName,
            value: info.message.encode(message).finish(),
        });
    }
    unpackAny(any) {
        const fullName = this.canonicalFullName(any.type_url);
        const info = this._registeredMessages[fullName];
        if (!info) {
            console.error('Tried to unpack any any with unregistered type', fullName);
            throw 'Tried to pack an unregistered message';
        }
        return info.message.decode(any.value);
    }
    canonicalFullName(name) {
        return name.replace('type.googleapis.com/', '');
    }
}
export class MutationBuilder {
    constructor(entityId) {
        this.entityId = entityId;
        this.request = new exocore.index.MutationRequest();
    }
    static createEntity(entityId) {
        if (!entityId) {
            entityId = _exocore_wasm.generate_id('et');
        }
        return new MutationBuilder(entityId);
    }
    static updateEntity(entityId) {
        return new MutationBuilder(entityId);
    }
    andUpdateEntity(entityId) {
        this.entityId = entityId;
        return this;
    }
    andCreateEntity(entityId = null) {
        if (!entityId) {
            entityId = _exocore_wasm.generate_id('et');
        }
        this.entityId = entityId;
        return this;
    }
    putTrait(message, traitId = null) {
        if (!traitId) {
            traitId = _exocore_wasm.generate_id('trt');
        }
        this.request.mutations.push(new exocore.index.EntityMutation({
            entityId: this.entityId,
            putTrait: new exocore.index.PutTraitMutation({
                trait: new exocore.index.Trait({
                    id: traitId,
                    creationDate: toProtoTimestamp(new Date()),
                    message: Exocore.registry.packToAny(message),
                })
            })
        }));
        return this;
    }
    deleteTrait(traitId) {
        this.request.mutations.push(new exocore.index.EntityMutation({
            entityId: this.entityId,
            deleteTrait: new exocore.index.DeleteTraitMutation({
                traitId: traitId,
            })
        }));
        return this;
    }
    returnEntities() {
        this.request.returnEntities = true;
        return this;
    }
    build() {
        return this.request;
    }
}
export class QueryBuilder {
    constructor() {
        this.query = new exocore.index.EntityQuery();
    }
    static withTrait(message, traitQuery) {
        let builder = new QueryBuilder();
        builder.query.trait = new exocore.index.TraitPredicate({
            traitName: Exocore.registry.messageFullName(message),
            query: traitQuery,
        });
        return builder;
    }
    static matching(query) {
        let builder = new QueryBuilder();
        builder.query.match = new exocore.index.MatchPredicate({
            query: query
        });
        return builder;
    }
    static withIds(ids) {
        if (!Array.isArray(ids)) {
            ids = [ids];
        }
        let builder = new QueryBuilder();
        builder.query.ids = new exocore.index.IdsPredicate({
            ids: ids,
        });
        return builder;
    }
    static all() {
        let builder = new QueryBuilder();
        builder.query.all = new exocore.index.AllPredicate();
        return builder;
    }
    count(count) {
        this.query.paging = new exocore.index.Paging({
            count: count,
        });
        return this;
    }
    orderByField(field, ascending) {
        this.query.ordering = new exocore.index.Ordering({
            ascending: ascending === true,
            field: field,
        });
        return this;
    }
    orderByOperationIds(ascending) {
        this.query.ordering = new exocore.index.Ordering({
            ascending: ascending === true,
            operationId: true,
        });
        return this;
    }
    includeDeleted() {
        this.query.includeDeleted = true;
        return this;
    }
    build() {
        return this.query;
    }
}
export class TraitQueryBuilder {
    constructor() {
        this.query = new exocore.index.TraitQuery();
    }
    static refersTo(field, entityId, traitId) {
        let builder = new TraitQueryBuilder();
        builder.query.reference = new exocore.index.TraitFieldReferencePredicate({
            field: field,
            reference: new exocore.index.Reference({
                entityId: entityId,
                traitId: traitId,
            })
        });
        return builder;
    }
    static matching(query) {
        let builder = new TraitQueryBuilder();
        builder.query.match = new exocore.index.MatchPredicate({
            query: query,
        });
        return builder;
    }
    build() {
        return this.query;
    }
}
export function toProtoTimestamp(date) {
    const epoch = date.getTime();
    const seconds = Math.floor(epoch / 1000);
    return new protos.google.protobuf.Timestamp({
        seconds: seconds,
        nanos: (epoch - (seconds * 1000)) * 1000,
    });
}
export function fromProtoTimestamp(ts) {
    return new Date(ts.seconds * 1000 + ts.nanos / 1000);
}
export function matchTrait(trait, matchMap) {
    const fullName = Exocore.registry.canonicalFullName(trait.message.type_url);
    if (fullName in matchMap) {
        const message = Exocore.registry.unpackAny(trait.message);
        return matchMap[fullName](trait, message);
    }
    else {
        return null;
    }
}
//# sourceMappingURL=index.js.map
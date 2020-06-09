import * as proto from './proto';

export {
    proto
}

var _exocore_wasm = null;

export class Client {
    constructor(innerClient) {
        this.innerClient = innerClient;
    }

    static create(config, statusChangeCallback) {
        const configJson = JSON.stringify(config);
        const configBytes = new TextEncoder('utf-8').encode(configJson);

        if (_exocore_wasm != null) {
            const innerClient = new _exocore_wasm.ExocoreClient(configBytes, 'json', statusChangeCallback);
            return Promise.resolve(new Client(innerClient));

        } else {
            return import("exocore-client-web").then((module) => {
                _exocore_wasm = module;

                console.log("Exocore WASM client loaded");
                const innerClient = new _exocore_wasm.ExocoreClient(configBytes, 'json', statusChangeCallback);
                return new Client(innerClient);
            });
        }
    }

    mutate(mutation) {
        const encoded = proto.exocore.index.MutationRequest.encode(mutation).finish();

        return this.innerClient
            .mutate(encoded)
            .then((resultsData) => {
                const result = proto.exocore.index.MutationResult.decode(resultsData);
                return result;
            });
    }

    query(query) {
        const encoded = proto.exocore.index.EntityQuery.encode(query).finish();

        return this.innerClient
            .query(encoded).then((resultsData) => {
                return proto.exocore.index.EntityResults.decode(resultsData);
            });
    }

    watchedQuery(query) {
        const encoded = proto.exocore.index.EntityQuery.encode(query).finish();
        return new WatchedQuery(this.innerClient.watched_query(encoded));
    }

    generateId(prefix = '') {
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
            const res = proto.exocore.index.EntityResults.decode(resultsData);
            cb(res);
        })
    }

    free() {
        this.inner.free();
    }
}

var _registeredMessages = {};

export class Registry {
    static registerMessage(message, fullName) {
        message.prototype._exocore_proto_full_name = fullName;

        _registeredMessages[fullName] = {
            fullName: fullName,
            message: message,
        }
    }

    static messageFullName(message) {
        let fullName = message._exocore_proto_full_name;
        if (!fullName && message.prototype) {
            fullName = message.prototype._exocore_proto_full_name;
        }

        const info = _registeredMessages[fullName];
        if (!info) {
            console.error('Tried to get full name for an unregistered message', message);
            throw 'Tried to pack an unregistered message';
        }

        return info.fullName;
    }

    static packToAny(message) {
        const info = _registeredMessages[message._exocore_proto_full_name];
        if (!info) {
            console.log('Tried to pack an unregistered message', message);
            throw 'Tried to pack an unregistered message';
        }

        return new proto.google.protobuf.Any({
            type_url: 'type.googleapis.com/' + info.fullName,
            value: info.message.encode(message).finish(),
        })
    }

    static unpackAny(any) {
        const fullName = Registry.canonicalFullName(any.type_url);
        const info = _registeredMessages[fullName];
        if (!info) {
            console.error('Tried to unpack any any with unregistered type', fullName);
            throw 'Tried to pack an unregistered message';
        }

        return info.message.decode(any.value);
    }

    static canonicalFullName(name) {
        return name.replace('type.googleapis.com/', '');
    }
}

export class MutationBuilder {
    constructor(entityId) {
        this.entityId = entityId;
        this.request = new proto.exocore.index.MutationRequest();
    }

    static createEntity(entityId = null) {
        if (!entityId) {
            entityId = _exocore_wasm.generate_id('et')
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
            entityId = _exocore_wasm.generate_id('et')
        }

        this.entityId = entityId;

        return this;
    }

    putTrait(message, traitId = null) {
        if (!traitId) {
            traitId = _exocore_wasm.generate_id('trt');
        }

        this.request.mutations.push(new proto.exocore.index.EntityMutation({
            entityId: this.entityId,
            putTrait: new proto.exocore.index.PutTraitMutation({
                trait: new proto.exocore.index.Trait({
                    id: traitId,
                    creationDate: toProtoTimestamp(new Date()),
                    message: Registry.packToAny(message),
                })
            })
        }));

        return this;
    }

    deleteTrait(traitId) {
        this.request.mutations.push(new proto.exocore.index.EntityMutation({
            entityId: this.entityId,
            deleteTrait: new proto.exocore.index.DeleteTraitMutation({
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
        this.query = new proto.exocore.index.EntityQuery();
    }

    static withTrait(message, traitQuery = null) {
        let builder = new QueryBuilder();
        builder.query.trait = new proto.exocore.index.TraitPredicate({
            traitName: Registry.messageFullName(message),
            query: traitQuery,
        });
        return builder;
    }

    static matching(query) {
        let builder = new QueryBuilder();
        builder.query.match = new proto.exocore.index.MatchPredicate({
            query: query
        });
        return builder;
    }

    static withId(id) {
        let builder = new QueryBuilder();
        builder.query.id = new proto.exocore.index.IdPredicate({
            id: id,
        });
        return builder;
    }

    count(count) {
        this.query.paging = new proto.exocore.index.Paging({
            count: count,
        });
        return this;
    }

    orderByField(field, ascending) {
        this.query.sorting = new proto.exocore.index.Sorting({
            ascending: ascending === true,
            field: field,
        });
        return this;
    }

    build() {
        return this.query;
    }
}

export class TraitQueryBuilder {
    constructor() {
        this.query = new proto.exocore.index.TraitQuery();
    }

    static refersTo(field, entityId, traitId = null) {
        let builder = new TraitQueryBuilder();
        builder.query.reference = new proto.exocore.index.TraitFieldReferencePredicate({
            field: field,
            reference: new proto.exocore.index.Reference({
                entityId: entityId,
                traitId: traitId,
            })
        });

        return builder;
    }

    static matching(query) {
        let builder = new TraitQueryBuilder();
        builder.query.match = new proto.exocore.index.MatchPredicate({
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

    return new proto.google.protobuf.Timestamp({
        seconds: seconds,
        nanos: (epoch - (seconds * 1000)) * 1000,
    });
}

export function fromProtoTimestamp(ts) {
    return new Date(ts.seconds * 1000 + ts.nanos / 1000);
}

export function matchTrait(trait, matchMap) {
    const fullName = Registry.canonicalFullName(trait.message.type_url);

    if (fullName in matchMap) {
        const message = Registry.unpackAny(trait.message);
        return matchMap[fullName](message);
    } else {
        return [];
    }
}
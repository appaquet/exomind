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
        const encoded = proto.exocore.index.EntityMutation.encode(mutation).finish();

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

    watched_query(query) {
        const encoded = proto.exocore.index.EntityQuery.encode(query).finish();

        return this.innerClient.watched_query(encoded);
    }

    generate_id(prefix = '') {
        return _exocore_wasm.generate_id(prefix);
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
        if (!fullName) {
            fullName = message.prototype._exocore_proto_full_name;
        }

        const info = _registeredMessages[fullName];
        if (!info) {
            console.log('Tried to get full name for an unregistered message', message);
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
        const fullName = any.type_url.replace('type.googleapis.com/', '');
        const info = _registeredMessages[fullName];
        if (!info) {
            console.log('Tried to unpack any any with unregistered type', fullName);
            throw 'Tried to pack an unregistered message';
        }

        return info.message.decode(any.value);
    }
}

export class MutationBuilder {
    constructor(entityId) {
        this.mutation = new proto.exocore.index.EntityMutation({
            entityId: entityId,
        });
    }

    static createEntity(entityId = null) {
        if (!entityId) {
            entityId = _exocore_wasm.generate_id("entity")
        }

        return new MutationBuilder(entityId);
    }

    static updateEntity(entityId) {
        return new MutationBuilder(entityId);
    }

    putTrait(message, traitId = null) {
        this.mutation.putTrait = new proto.exocore.index.PutTraitMutation({
            trait: new proto.exocore.index.Trait({
                id: traitId,
                creationDate: toProtoTimestamp(new Date()),
                message: Registry.packToAny(message),
            })
        });
        return this;
    }

    deleteTrait(traitId) {
        this.mutation.deleteTrait = new proto.exocore.index.DeleteTraitMutation({
            traitId: traitId,
        });
        return this;
    }

    build() {
        return this.mutation;
    }
}

export class QueryBuilder {
    constructor() {
        this.query = new proto.exocore.index.EntityQuery({});
    }

    static withTrait(message) {
        let builder = new QueryBuilder();
        builder.query.trait = new proto.exocore.index.TraitPredicate({
            traitName: Registry.messageFullName(message),
        });
        return builder;
    }

    static matching(query) {
        let builder = new QueryBuilder();
        builder.query.trait = new proto.exocore.index.MatchPredicate({
            query: query
        });
        return builder;
    }

    count(count) {
        this.query.paging = new proto.exocore.index.Paging({
            count: count,
        });
        return this;
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
    const fullName = trait.message.type_url.replace('type.googleapis.com/', '');

    if (matchMap[fullName]) {
        const message = Registry.unpackAny(trait.message);

        return matchMap[fullName](message);
    } else {
        return [];
    }
}
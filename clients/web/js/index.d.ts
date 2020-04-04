import {WatchedQuery} from "exocore-client-wasm";
import * as proto from './proto';

export {
    proto
}

export class Client {
    static create(config: object, status_change_callback?: any): Promise<Client>;

    mutate(mutation: proto.exocore.index.EntityMutation): Promise<proto.exocore.index.MutationResult>;

    query(query: proto.exocore.index.EntityQuery): Promise<proto.exocore.index.EntityResults>;

    watched_query(query: proto.exocore.index.EntityQuery): WatchedQuery;

    generate_id(prefix?: string): string;
}

export class Registry {
    static registerMessage(message: any, fullName: string);

    static messageFullName(message: any): string;

    static packToAny(message: any): proto.google.protobuf.Any;

    static unpackAny(any: proto.google.protobuf.Any): any;
}

export class MutationBuilder {
    static createEntity(entityId?: string | null): MutationBuilder;

    static updateEntity(entityId: string): MutationBuilder;

    putTrait(message: any, traitId?: string): MutationBuilder;

    deleteTrait(traitId: string): MutationBuilder;

    build(): proto.exocore.index.EntityMutation;
}

export class QueryBuilder {
    static withTrait(message: any): QueryBuilder;

    static matching(query: string): QueryBuilder;

    count(count: number): QueryBuilder;

    build(): proto.exocore.index.EntityQuery;
}

export function toProtoTimestamp(date: Date): proto.google.protobuf.Timestamp;

export function fromProtoTimestamp(ts: proto.google.protobuf.Timestamp): Date;

export function matchTrait(trait: proto.exocore.index.Trait, matchMap): proto.google.protobuf.Any;

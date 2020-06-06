import * as proto from './proto';

export {proto}

export class Client {
    static create(config: object, status_change_callback?: any): Promise<Client>;

    mutate(mutation: proto.exocore.index.MutationRequest): Promise<proto.exocore.index.MutationResult>;

    query(query: proto.exocore.index.EntityQuery): Promise<proto.exocore.index.EntityResults>;

    watchedQuery(query: proto.exocore.index.EntityQuery): WatchedQuery;

    generateId(prefix?: string): string;
}

export class WatchedQuery {
    onChange(cb: (results: proto.exocore.index.EntityResults)=>void): void;

    free(): void;
}

export class Registry {
    static registerMessage(message: any, fullName: string): any;

    static messageFullName(message: any): string;

    static packToAny(message: any): proto.google.protobuf.IAny;

    static unpackAny(any: proto.google.protobuf.IAny): any;

    static canonicalFullName(name: string): string;
}

export class MutationBuilder {
    static createEntity(entityId?: string | null): MutationBuilder;

    static updateEntity(entityId: string): MutationBuilder;

    andUpdateEntity(entityId: string): MutationBuilder;

    andCreateEntity(entityId?: string | null): MutationBuilder;

    putTrait(message: any, traitId?: string): MutationBuilder;

    deleteTrait(traitId: string): MutationBuilder;

    returnEntities(): MutationBuilder;

    build(): proto.exocore.index.MutationRequest;
}

export class QueryBuilder {
    static withTrait(message: any, traitQuery?: proto.exocore.index.ITraitQuery): QueryBuilder;

    static matching(query: string): QueryBuilder;

    static withId(id: string): QueryBuilder;

    count(count: number): QueryBuilder;

    orderByField(field: string, ascending?: boolean): QueryBuilder;

    build(): proto.exocore.index.EntityQuery;
}

export class TraitQueryBuilder {
    static refersTo(entityId: string, traitId?: string): TraitQueryBuilder;

    build(): proto.exocore.index.TraitQuery;
}

export function toProtoTimestamp(date: Date): proto.google.protobuf.ITimestamp;

export function fromProtoTimestamp(ts: proto.google.protobuf.ITimestamp): Date;

export function matchTrait(trait: proto.exocore.index.ITrait, matchMap: any): proto.google.protobuf.IAny;

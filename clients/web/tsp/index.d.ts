import * as protos from '../js/protos';
import { exocore } from '../js/protos';
export { protos, exocore, };
export declare class Exocore {
    static defaultInstance: ExocoreInstance;
    static get initialized(): boolean;
    static initialize(config: object): Promise<ExocoreInstance>;
    static get store(): Store;
    static get registry(): Registry;
}
export declare class ExocoreInstance {
    wasmClient: any;
    store: Store;
    status: string;
    registry: Registry;
    onChange?: () => void;
    constructor(wasmClient: any);
    _triggerStatusChange(status: string): void;
}
export declare class Store {
    wasmClient: any;
    statusChangeCallback: () => void;
    constructor(inner: any);
    mutate(mutation: exocore.index.IMutationRequest): Promise<exocore.index.MutationResult>;
    query(query: exocore.index.IEntityQuery): Promise<exocore.index.EntityResults>;
    watchedQuery(query: exocore.index.IEntityQuery): WatchedQuery;
    generateId(prefix?: string): string;
}
export declare class WatchedQuery {
    inner: any;
    constructor(inner: any);
    onChange(cb: (results: exocore.index.EntityResults) => void): WatchedQuery;
    free(): void;
}
export declare class Registry {
    private _registeredMessages;
    registerMessage(message: any, fullName: string): any;
    messageFullName(message: any): string;
    packToAny(message: any): protos.google.protobuf.IAny;
    unpackAny(any: protos.google.protobuf.IAny): any;
    canonicalFullName(name: string): string;
}
export declare class MutationBuilder {
    entityId: string;
    request: exocore.index.MutationRequest;
    constructor(entityId: string);
    static createEntity(entityId?: string): MutationBuilder;
    static updateEntity(entityId: string): MutationBuilder;
    andUpdateEntity(entityId: string): MutationBuilder;
    andCreateEntity(entityId?: string): MutationBuilder;
    putTrait(message: any, traitId?: string): MutationBuilder;
    deleteTrait(traitId: string): MutationBuilder;
    returnEntities(): MutationBuilder;
    build(): exocore.index.MutationRequest;
}
export declare class QueryBuilder {
    query: exocore.index.EntityQuery;
    constructor();
    static withTrait(message: any, traitQuery?: exocore.index.ITraitQuery): QueryBuilder;
    static matching(query: string): QueryBuilder;
    static withIds(ids: string | string[]): QueryBuilder;
    static all(): QueryBuilder;
    count(count: number): QueryBuilder;
    orderByField(field: string, ascending: boolean): QueryBuilder;
    orderByOperationIds(ascending: boolean): QueryBuilder;
    includeDeleted(): QueryBuilder;
    build(): exocore.index.IEntityQuery;
}
export declare class TraitQueryBuilder {
    query: exocore.index.TraitQuery;
    constructor();
    static refersTo(field: string, entityId: string, traitId?: string): TraitQueryBuilder;
    static matching(query: string): TraitQueryBuilder;
    build(): exocore.index.TraitQuery;
}
export declare function toProtoTimestamp(date: Date): protos.google.protobuf.ITimestamp;
export declare function fromProtoTimestamp(ts: protos.google.protobuf.ITimestamp): Date;
export declare function matchTrait<T>(trait: exocore.index.ITrait, matchMap: {
    [fullName: string]: (trait: exocore.index.ITrait, message: any) => T;
}): T | null;

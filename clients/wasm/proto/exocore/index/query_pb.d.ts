// package: exocore.index
// file: exocore/index/query.proto

import * as jspb from "google-protobuf";
import * as exocore_index_entity_pb from "../../exocore/index/entity_pb";

export class EntityQuery extends jspb.Message {
  hasMatch(): boolean;
  clearMatch(): void;
  getMatch(): MatchPredicate | undefined;
  setMatch(value?: MatchPredicate): void;

  hasTrait(): boolean;
  clearTrait(): void;
  getTrait(): TraitPredicate | undefined;
  setTrait(value?: TraitPredicate): void;

  hasId(): boolean;
  clearId(): void;
  getId(): IdPredicate | undefined;
  setId(value?: IdPredicate): void;

  hasTest(): boolean;
  clearTest(): void;
  getTest(): TestPredicate | undefined;
  setTest(value?: TestPredicate): void;

  hasPaging(): boolean;
  clearPaging(): void;
  getPaging(): Paging | undefined;
  setPaging(value?: Paging): void;

  getSummary(): boolean;
  setSummary(value: boolean): void;

  getWatchToken(): number;
  setWatchToken(value: number): void;

  getResultHash(): number;
  setResultHash(value: number): void;

  getPredicateCase(): EntityQuery.PredicateCase;
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): EntityQuery.AsObject;
  static toObject(includeInstance: boolean, msg: EntityQuery): EntityQuery.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: EntityQuery, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): EntityQuery;
  static deserializeBinaryFromReader(message: EntityQuery, reader: jspb.BinaryReader): EntityQuery;
}

export namespace EntityQuery {
  export type AsObject = {
    match?: MatchPredicate.AsObject,
    trait?: TraitPredicate.AsObject,
    id?: IdPredicate.AsObject,
    test?: TestPredicate.AsObject,
    paging?: Paging.AsObject,
    summary: boolean,
    watchToken: number,
    resultHash: number,
  }

  export enum PredicateCase {
    PREDICATE_NOT_SET = 0,
    MATCH = 1,
    TRAIT = 2,
    ID = 3,
    TEST = 99,
  }
}

export class MatchPredicate extends jspb.Message {
  getQuery(): string;
  setQuery(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): MatchPredicate.AsObject;
  static toObject(includeInstance: boolean, msg: MatchPredicate): MatchPredicate.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: MatchPredicate, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): MatchPredicate;
  static deserializeBinaryFromReader(message: MatchPredicate, reader: jspb.BinaryReader): MatchPredicate;
}

export namespace MatchPredicate {
  export type AsObject = {
    query: string,
  }
}

export class IdPredicate extends jspb.Message {
  getId(): string;
  setId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): IdPredicate.AsObject;
  static toObject(includeInstance: boolean, msg: IdPredicate): IdPredicate.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: IdPredicate, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): IdPredicate;
  static deserializeBinaryFromReader(message: IdPredicate, reader: jspb.BinaryReader): IdPredicate;
}

export namespace IdPredicate {
  export type AsObject = {
    id: string,
  }
}

export class TestPredicate extends jspb.Message {
  getSuccess(): boolean;
  setSuccess(value: boolean): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TestPredicate.AsObject;
  static toObject(includeInstance: boolean, msg: TestPredicate): TestPredicate.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: TestPredicate, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TestPredicate;
  static deserializeBinaryFromReader(message: TestPredicate, reader: jspb.BinaryReader): TestPredicate;
}

export namespace TestPredicate {
  export type AsObject = {
    success: boolean,
  }
}

export class TraitPredicate extends jspb.Message {
  getTraitName(): string;
  setTraitName(value: string): void;

  hasQuery(): boolean;
  clearQuery(): void;
  getQuery(): TraitQuery | undefined;
  setQuery(value?: TraitQuery): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TraitPredicate.AsObject;
  static toObject(includeInstance: boolean, msg: TraitPredicate): TraitPredicate.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: TraitPredicate, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TraitPredicate;
  static deserializeBinaryFromReader(message: TraitPredicate, reader: jspb.BinaryReader): TraitPredicate;
}

export namespace TraitPredicate {
  export type AsObject = {
    traitName: string,
    query?: TraitQuery.AsObject,
  }
}

export class TraitQuery extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TraitQuery.AsObject;
  static toObject(includeInstance: boolean, msg: TraitQuery): TraitQuery.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: TraitQuery, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TraitQuery;
  static deserializeBinaryFromReader(message: TraitQuery, reader: jspb.BinaryReader): TraitQuery;
}

export namespace TraitQuery {
  export type AsObject = {
  }
}

export class Paging extends jspb.Message {
  getAfterToken(): string;
  setAfterToken(value: string): void;

  getBeforeToken(): string;
  setBeforeToken(value: string): void;

  getCount(): number;
  setCount(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Paging.AsObject;
  static toObject(includeInstance: boolean, msg: Paging): Paging.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Paging, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Paging;
  static deserializeBinaryFromReader(message: Paging, reader: jspb.BinaryReader): Paging;
}

export namespace Paging {
  export type AsObject = {
    afterToken: string,
    beforeToken: string,
    count: number,
  }
}

export class EntityResults extends jspb.Message {
  clearEntitiesList(): void;
  getEntitiesList(): Array<EntityResult>;
  setEntitiesList(value: Array<EntityResult>): void;
  addEntities(value?: EntityResult, index?: number): EntityResult;

  getSummary(): boolean;
  setSummary(value: boolean): void;

  getEstimatedCount(): number;
  setEstimatedCount(value: number): void;

  hasCurrentPage(): boolean;
  clearCurrentPage(): void;
  getCurrentPage(): Paging | undefined;
  setCurrentPage(value?: Paging): void;

  hasNextPage(): boolean;
  clearNextPage(): void;
  getNextPage(): Paging | undefined;
  setNextPage(value?: Paging): void;

  getHash(): number;
  setHash(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): EntityResults.AsObject;
  static toObject(includeInstance: boolean, msg: EntityResults): EntityResults.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: EntityResults, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): EntityResults;
  static deserializeBinaryFromReader(message: EntityResults, reader: jspb.BinaryReader): EntityResults;
}

export namespace EntityResults {
  export type AsObject = {
    entitiesList: Array<EntityResult.AsObject>,
    summary: boolean,
    estimatedCount: number,
    currentPage?: Paging.AsObject,
    nextPage?: Paging.AsObject,
    hash: number,
  }
}

export class EntityResult extends jspb.Message {
  hasEntity(): boolean;
  clearEntity(): void;
  getEntity(): exocore_index_entity_pb.Entity | undefined;
  setEntity(value?: exocore_index_entity_pb.Entity): void;

  getSource(): EntityResultSourceMap[keyof EntityResultSourceMap];
  setSource(value: EntityResultSourceMap[keyof EntityResultSourceMap]): void;

  getSortToken(): string;
  setSortToken(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): EntityResult.AsObject;
  static toObject(includeInstance: boolean, msg: EntityResult): EntityResult.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: EntityResult, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): EntityResult;
  static deserializeBinaryFromReader(message: EntityResult, reader: jspb.BinaryReader): EntityResult;
}

export namespace EntityResult {
  export type AsObject = {
    entity?: exocore_index_entity_pb.Entity.AsObject,
    source: EntityResultSourceMap[keyof EntityResultSourceMap],
    sortToken: string,
  }
}

export interface EntityResultSourceMap {
  UNKNOWN: 0;
  PENDING: 1;
  CHAIN: 2;
}

export const EntityResultSource: EntityResultSourceMap;


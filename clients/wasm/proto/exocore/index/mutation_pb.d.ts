// package: exocore.index
// file: exocore/index/mutation.proto

import * as jspb from "google-protobuf";
import * as exocore_index_entity_pb from "../../exocore/index/entity_pb";

export class EntityMutation extends jspb.Message {
  getEntityId(): string;
  setEntityId(value: string): void;

  hasPutTrait(): boolean;
  clearPutTrait(): void;
  getPutTrait(): PutTraitMutation | undefined;
  setPutTrait(value?: PutTraitMutation): void;

  hasDeleteTrait(): boolean;
  clearDeleteTrait(): void;
  getDeleteTrait(): DeleteTraitMutation | undefined;
  setDeleteTrait(value?: DeleteTraitMutation): void;

  hasTest(): boolean;
  clearTest(): void;
  getTest(): TestMutation | undefined;
  setTest(value?: TestMutation): void;

  getMutationCase(): EntityMutation.MutationCase;
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): EntityMutation.AsObject;
  static toObject(includeInstance: boolean, msg: EntityMutation): EntityMutation.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: EntityMutation, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): EntityMutation;
  static deserializeBinaryFromReader(message: EntityMutation, reader: jspb.BinaryReader): EntityMutation;
}

export namespace EntityMutation {
  export type AsObject = {
    entityId: string,
    putTrait?: PutTraitMutation.AsObject,
    deleteTrait?: DeleteTraitMutation.AsObject,
    test?: TestMutation.AsObject,
  }

  export enum MutationCase {
    MUTATION_NOT_SET = 0,
    PUT_TRAIT = 2,
    DELETE_TRAIT = 3,
    TEST = 99,
  }
}

export class PutTraitMutation extends jspb.Message {
  hasTrait(): boolean;
  clearTrait(): void;
  getTrait(): exocore_index_entity_pb.Trait | undefined;
  setTrait(value?: exocore_index_entity_pb.Trait): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): PutTraitMutation.AsObject;
  static toObject(includeInstance: boolean, msg: PutTraitMutation): PutTraitMutation.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: PutTraitMutation, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): PutTraitMutation;
  static deserializeBinaryFromReader(message: PutTraitMutation, reader: jspb.BinaryReader): PutTraitMutation;
}

export namespace PutTraitMutation {
  export type AsObject = {
    trait?: exocore_index_entity_pb.Trait.AsObject,
  }
}

export class DeleteTraitMutation extends jspb.Message {
  getTraitId(): string;
  setTraitId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DeleteTraitMutation.AsObject;
  static toObject(includeInstance: boolean, msg: DeleteTraitMutation): DeleteTraitMutation.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: DeleteTraitMutation, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DeleteTraitMutation;
  static deserializeBinaryFromReader(message: DeleteTraitMutation, reader: jspb.BinaryReader): DeleteTraitMutation;
}

export namespace DeleteTraitMutation {
  export type AsObject = {
    traitId: string,
  }
}

export class TestMutation extends jspb.Message {
  getSuccess(): boolean;
  setSuccess(value: boolean): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TestMutation.AsObject;
  static toObject(includeInstance: boolean, msg: TestMutation): TestMutation.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: TestMutation, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TestMutation;
  static deserializeBinaryFromReader(message: TestMutation, reader: jspb.BinaryReader): TestMutation;
}

export namespace TestMutation {
  export type AsObject = {
    success: boolean,
  }
}

export class MutationResult extends jspb.Message {
  getOperationId(): number;
  setOperationId(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): MutationResult.AsObject;
  static toObject(includeInstance: boolean, msg: MutationResult): MutationResult.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: MutationResult, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): MutationResult;
  static deserializeBinaryFromReader(message: MutationResult, reader: jspb.BinaryReader): MutationResult;
}

export namespace MutationResult {
  export type AsObject = {
    operationId: number,
  }
}


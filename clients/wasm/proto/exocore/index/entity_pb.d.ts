// package: exocore.index
// file: exocore/index/entity.proto

import * as jspb from "google-protobuf";
import * as google_protobuf_timestamp_pb from "google-protobuf/google/protobuf/timestamp_pb";
import * as google_protobuf_any_pb from "google-protobuf/google/protobuf/any_pb";

export class Entity extends jspb.Message {
  getId(): string;
  setId(value: string): void;

  clearTraitsList(): void;
  getTraitsList(): Array<Trait>;
  setTraitsList(value: Array<Trait>): void;
  addTraits(value?: Trait, index?: number): Trait;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Entity.AsObject;
  static toObject(includeInstance: boolean, msg: Entity): Entity.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Entity, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Entity;
  static deserializeBinaryFromReader(message: Entity, reader: jspb.BinaryReader): Entity;
}

export namespace Entity {
  export type AsObject = {
    id: string,
    traitsList: Array<Trait.AsObject>,
  }
}

export class Trait extends jspb.Message {
  getId(): string;
  setId(value: string): void;

  hasMessage(): boolean;
  clearMessage(): void;
  getMessage(): google_protobuf_any_pb.Any | undefined;
  setMessage(value?: google_protobuf_any_pb.Any): void;

  hasCreationDate(): boolean;
  clearCreationDate(): void;
  getCreationDate(): google_protobuf_timestamp_pb.Timestamp | undefined;
  setCreationDate(value?: google_protobuf_timestamp_pb.Timestamp): void;

  hasModificationDate(): boolean;
  clearModificationDate(): void;
  getModificationDate(): google_protobuf_timestamp_pb.Timestamp | undefined;
  setModificationDate(value?: google_protobuf_timestamp_pb.Timestamp): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Trait.AsObject;
  static toObject(includeInstance: boolean, msg: Trait): Trait.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Trait, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Trait;
  static deserializeBinaryFromReader(message: Trait, reader: jspb.BinaryReader): Trait;
}

export namespace Trait {
  export type AsObject = {
    id: string,
    message?: google_protobuf_any_pb.Any.AsObject,
    creationDate?: google_protobuf_timestamp_pb.Timestamp.AsObject,
    modificationDate?: google_protobuf_timestamp_pb.Timestamp.AsObject,
  }
}


// package: exocore.test
// file: exocore/test/test.proto

import * as jspb from "google-protobuf";
import * as exocore_index_options_pb from "../../exocore/index/options_pb";
import * as google_protobuf_timestamp_pb from "google-protobuf/google/protobuf/timestamp_pb";

export class TestMessage extends jspb.Message {
  getString1(): string;
  setString1(value: string): void;

  getString2(): string;
  setString2(value: string): void;

  hasStruct1(): boolean;
  clearStruct1(): void;
  getStruct1(): TestStruct | undefined;
  setStruct1(value?: TestStruct): void;

  hasOneofString1(): boolean;
  clearOneofString1(): void;
  getOneofString1(): string;
  setOneofString1(value: string): void;

  hasOneofInt1(): boolean;
  clearOneofInt1(): void;
  getOneofInt1(): number;
  setOneofInt1(value: number): void;

  hasDate1(): boolean;
  clearDate1(): void;
  getDate1(): google_protobuf_timestamp_pb.Timestamp | undefined;
  setDate1(value?: google_protobuf_timestamp_pb.Timestamp): void;

  hasDate2(): boolean;
  clearDate2(): void;
  getDate2(): google_protobuf_timestamp_pb.Timestamp | undefined;
  setDate2(value?: google_protobuf_timestamp_pb.Timestamp): void;

  getInt1(): number;
  setInt1(value: number): void;

  getInt2(): number;
  setInt2(value: number): void;

  getFieldsCase(): TestMessage.FieldsCase;
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TestMessage.AsObject;
  static toObject(includeInstance: boolean, msg: TestMessage): TestMessage.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: TestMessage, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TestMessage;
  static deserializeBinaryFromReader(message: TestMessage, reader: jspb.BinaryReader): TestMessage;
}

export namespace TestMessage {
  export type AsObject = {
    string1: string,
    string2: string,
    struct1?: TestStruct.AsObject,
    oneofString1: string,
    oneofInt1: number,
    date1?: google_protobuf_timestamp_pb.Timestamp.AsObject,
    date2?: google_protobuf_timestamp_pb.Timestamp.AsObject,
    int1: number,
    int2: number,
  }

  export enum FieldsCase {
    FIELDS_NOT_SET = 0,
    ONEOF_STRING1 = 4,
    ONEOF_INT1 = 5,
  }
}

export class TestStruct extends jspb.Message {
  getString1(): string;
  setString1(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TestStruct.AsObject;
  static toObject(includeInstance: boolean, msg: TestStruct): TestStruct.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: TestStruct, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TestStruct;
  static deserializeBinaryFromReader(message: TestStruct, reader: jspb.BinaryReader): TestStruct;
}

export namespace TestStruct {
  export type AsObject = {
    string1: string,
  }
}

export class TestMessage2 extends jspb.Message {
  getString1(): string;
  setString1(value: string): void;

  getString2(): string;
  setString2(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TestMessage2.AsObject;
  static toObject(includeInstance: boolean, msg: TestMessage2): TestMessage2.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: TestMessage2, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TestMessage2;
  static deserializeBinaryFromReader(message: TestMessage2, reader: jspb.BinaryReader): TestMessage2;
}

export namespace TestMessage2 {
  export type AsObject = {
    string1: string,
    string2: string,
  }
}


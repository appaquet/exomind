// source: exocore/index/options.proto
/**
 * @fileoverview
 * @enhanceable
 * @suppress {messageConventions} JS Compiler reports an error if a variable or
 *     field starts with 'MSG_' and isn't a translatable message.
 * @public
 */
// GENERATED CODE -- DO NOT EDIT!

var jspb = require('google-protobuf');
var goog = jspb;
var global = Function('return this')();

var google_protobuf_descriptor_pb = require('google-protobuf/google/protobuf/descriptor_pb.js');
goog.object.extend(proto, google_protobuf_descriptor_pb);
goog.exportSymbol('proto.exocore.indexed', null, global);

/**
 * A tuple of {field number, class constructor} for the extension
 * field named `indexed`.
 * @type {!jspb.ExtensionFieldInfo<boolean>}
 */
proto.exocore.indexed = new jspb.ExtensionFieldInfo(
    1000,
    {indexed: 0},
    null,
     /** @type {?function((boolean|undefined),!jspb.Message=): !Object} */ (
         null),
    0);

google_protobuf_descriptor_pb.FieldOptions.extensionsBinary[1000] = new jspb.ExtensionFieldBinaryInfo(
    proto.exocore.indexed,
    jspb.BinaryReader.prototype.readBool,
    jspb.BinaryWriter.prototype.writeBool,
    undefined,
    undefined,
    false);
// This registers the extension field with the extended class, so that
// toObject() will function correctly.
google_protobuf_descriptor_pb.FieldOptions.extensions[1000] = proto.exocore.indexed;

goog.object.extend(exports, proto.exocore);

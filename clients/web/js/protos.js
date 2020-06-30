/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars*/
import * as $protobuf from "protobufjs/minimal";

// Common aliases
const $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;

// Exported root namespace
const $root = $protobuf.roots["exocore-root"] || ($protobuf.roots["exocore-root"] = {});

export const exocore = $root.exocore = (() => {

    /**
     * Namespace exocore.
     * @exports exocore
     * @namespace
     */
    const exocore = {};

    exocore.index = (function() {

        /**
         * Namespace index.
         * @memberof exocore
         * @namespace
         */
        const index = {};

        index.Entity = (function() {

            /**
             * Properties of an Entity.
             * @memberof exocore.index
             * @interface IEntity
             * @property {string|null} [id] Entity id
             * @property {Array.<exocore.index.ITrait>|null} [traits] Entity traits
             */

            /**
             * Constructs a new Entity.
             * @memberof exocore.index
             * @classdesc Represents an Entity.
             * @implements IEntity
             * @constructor
             * @param {exocore.index.IEntity=} [properties] Properties to set
             */
            function Entity(properties) {
                this.traits = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Entity id.
             * @member {string} id
             * @memberof exocore.index.Entity
             * @instance
             */
            Entity.prototype.id = "";

            /**
             * Entity traits.
             * @member {Array.<exocore.index.ITrait>} traits
             * @memberof exocore.index.Entity
             * @instance
             */
            Entity.prototype.traits = $util.emptyArray;

            /**
             * Creates a new Entity instance using the specified properties.
             * @function create
             * @memberof exocore.index.Entity
             * @static
             * @param {exocore.index.IEntity=} [properties] Properties to set
             * @returns {exocore.index.Entity} Entity instance
             */
            Entity.create = function create(properties) {
                return new Entity(properties);
            };

            /**
             * Encodes the specified Entity message. Does not implicitly {@link exocore.index.Entity.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.Entity
             * @static
             * @param {exocore.index.IEntity} message Entity message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Entity.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
                if (message.traits != null && message.traits.length)
                    for (let i = 0; i < message.traits.length; ++i)
                        $root.exocore.index.Trait.encode(message.traits[i], writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified Entity message, length delimited. Does not implicitly {@link exocore.index.Entity.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.Entity
             * @static
             * @param {exocore.index.IEntity} message Entity message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Entity.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an Entity message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.Entity
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.Entity} Entity
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Entity.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.Entity();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.id = reader.string();
                        break;
                    case 4:
                        if (!(message.traits && message.traits.length))
                            message.traits = [];
                        message.traits.push($root.exocore.index.Trait.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an Entity message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.Entity
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.Entity} Entity
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Entity.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an Entity message.
             * @function verify
             * @memberof exocore.index.Entity
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Entity.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.id != null && message.hasOwnProperty("id"))
                    if (!$util.isString(message.id))
                        return "id: string expected";
                if (message.traits != null && message.hasOwnProperty("traits")) {
                    if (!Array.isArray(message.traits))
                        return "traits: array expected";
                    for (let i = 0; i < message.traits.length; ++i) {
                        let error = $root.exocore.index.Trait.verify(message.traits[i]);
                        if (error)
                            return "traits." + error;
                    }
                }
                return null;
            };

            /**
             * Creates an Entity message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.Entity
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.Entity} Entity
             */
            Entity.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.Entity)
                    return object;
                let message = new $root.exocore.index.Entity();
                if (object.id != null)
                    message.id = String(object.id);
                if (object.traits) {
                    if (!Array.isArray(object.traits))
                        throw TypeError(".exocore.index.Entity.traits: array expected");
                    message.traits = [];
                    for (let i = 0; i < object.traits.length; ++i) {
                        if (typeof object.traits[i] !== "object")
                            throw TypeError(".exocore.index.Entity.traits: object expected");
                        message.traits[i] = $root.exocore.index.Trait.fromObject(object.traits[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from an Entity message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.Entity
             * @static
             * @param {exocore.index.Entity} message Entity
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Entity.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.traits = [];
                if (options.defaults)
                    object.id = "";
                if (message.id != null && message.hasOwnProperty("id"))
                    object.id = message.id;
                if (message.traits && message.traits.length) {
                    object.traits = [];
                    for (let j = 0; j < message.traits.length; ++j)
                        object.traits[j] = $root.exocore.index.Trait.toObject(message.traits[j], options);
                }
                return object;
            };

            /**
             * Converts this Entity to JSON.
             * @function toJSON
             * @memberof exocore.index.Entity
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Entity.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return Entity;
        })();

        index.Trait = (function() {

            /**
             * Properties of a Trait.
             * @memberof exocore.index
             * @interface ITrait
             * @property {string|null} [id] Trait id
             * @property {google.protobuf.IAny|null} [message] Trait message
             * @property {google.protobuf.ITimestamp|null} [creationDate] Trait creationDate
             * @property {google.protobuf.ITimestamp|null} [modificationDate] Trait modificationDate
             */

            /**
             * Constructs a new Trait.
             * @memberof exocore.index
             * @classdesc Represents a Trait.
             * @implements ITrait
             * @constructor
             * @param {exocore.index.ITrait=} [properties] Properties to set
             */
            function Trait(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Trait id.
             * @member {string} id
             * @memberof exocore.index.Trait
             * @instance
             */
            Trait.prototype.id = "";

            /**
             * Trait message.
             * @member {google.protobuf.IAny|null|undefined} message
             * @memberof exocore.index.Trait
             * @instance
             */
            Trait.prototype.message = null;

            /**
             * Trait creationDate.
             * @member {google.protobuf.ITimestamp|null|undefined} creationDate
             * @memberof exocore.index.Trait
             * @instance
             */
            Trait.prototype.creationDate = null;

            /**
             * Trait modificationDate.
             * @member {google.protobuf.ITimestamp|null|undefined} modificationDate
             * @memberof exocore.index.Trait
             * @instance
             */
            Trait.prototype.modificationDate = null;

            /**
             * Creates a new Trait instance using the specified properties.
             * @function create
             * @memberof exocore.index.Trait
             * @static
             * @param {exocore.index.ITrait=} [properties] Properties to set
             * @returns {exocore.index.Trait} Trait instance
             */
            Trait.create = function create(properties) {
                return new Trait(properties);
            };

            /**
             * Encodes the specified Trait message. Does not implicitly {@link exocore.index.Trait.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.Trait
             * @static
             * @param {exocore.index.ITrait} message Trait message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Trait.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.id);
                if (message.message != null && Object.hasOwnProperty.call(message, "message"))
                    $root.google.protobuf.Any.encode(message.message, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.creationDate != null && Object.hasOwnProperty.call(message, "creationDate"))
                    $root.google.protobuf.Timestamp.encode(message.creationDate, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.modificationDate != null && Object.hasOwnProperty.call(message, "modificationDate"))
                    $root.google.protobuf.Timestamp.encode(message.modificationDate, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified Trait message, length delimited. Does not implicitly {@link exocore.index.Trait.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.Trait
             * @static
             * @param {exocore.index.ITrait} message Trait message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Trait.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a Trait message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.Trait
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.Trait} Trait
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Trait.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.Trait();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.id = reader.string();
                        break;
                    case 2:
                        message.message = $root.google.protobuf.Any.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.creationDate = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                        break;
                    case 4:
                        message.modificationDate = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a Trait message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.Trait
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.Trait} Trait
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Trait.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a Trait message.
             * @function verify
             * @memberof exocore.index.Trait
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Trait.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.id != null && message.hasOwnProperty("id"))
                    if (!$util.isString(message.id))
                        return "id: string expected";
                if (message.message != null && message.hasOwnProperty("message")) {
                    let error = $root.google.protobuf.Any.verify(message.message);
                    if (error)
                        return "message." + error;
                }
                if (message.creationDate != null && message.hasOwnProperty("creationDate")) {
                    let error = $root.google.protobuf.Timestamp.verify(message.creationDate);
                    if (error)
                        return "creationDate." + error;
                }
                if (message.modificationDate != null && message.hasOwnProperty("modificationDate")) {
                    let error = $root.google.protobuf.Timestamp.verify(message.modificationDate);
                    if (error)
                        return "modificationDate." + error;
                }
                return null;
            };

            /**
             * Creates a Trait message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.Trait
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.Trait} Trait
             */
            Trait.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.Trait)
                    return object;
                let message = new $root.exocore.index.Trait();
                if (object.id != null)
                    message.id = String(object.id);
                if (object.message != null) {
                    if (typeof object.message !== "object")
                        throw TypeError(".exocore.index.Trait.message: object expected");
                    message.message = $root.google.protobuf.Any.fromObject(object.message);
                }
                if (object.creationDate != null) {
                    if (typeof object.creationDate !== "object")
                        throw TypeError(".exocore.index.Trait.creationDate: object expected");
                    message.creationDate = $root.google.protobuf.Timestamp.fromObject(object.creationDate);
                }
                if (object.modificationDate != null) {
                    if (typeof object.modificationDate !== "object")
                        throw TypeError(".exocore.index.Trait.modificationDate: object expected");
                    message.modificationDate = $root.google.protobuf.Timestamp.fromObject(object.modificationDate);
                }
                return message;
            };

            /**
             * Creates a plain object from a Trait message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.Trait
             * @static
             * @param {exocore.index.Trait} message Trait
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Trait.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.id = "";
                    object.message = null;
                    object.creationDate = null;
                    object.modificationDate = null;
                }
                if (message.id != null && message.hasOwnProperty("id"))
                    object.id = message.id;
                if (message.message != null && message.hasOwnProperty("message"))
                    object.message = $root.google.protobuf.Any.toObject(message.message, options);
                if (message.creationDate != null && message.hasOwnProperty("creationDate"))
                    object.creationDate = $root.google.protobuf.Timestamp.toObject(message.creationDate, options);
                if (message.modificationDate != null && message.hasOwnProperty("modificationDate"))
                    object.modificationDate = $root.google.protobuf.Timestamp.toObject(message.modificationDate, options);
                return object;
            };

            /**
             * Converts this Trait to JSON.
             * @function toJSON
             * @memberof exocore.index.Trait
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Trait.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return Trait;
        })();

        index.Reference = (function() {

            /**
             * Properties of a Reference.
             * @memberof exocore.index
             * @interface IReference
             * @property {string|null} [entityId] Reference entityId
             * @property {string|null} [traitId] Reference traitId
             */

            /**
             * Constructs a new Reference.
             * @memberof exocore.index
             * @classdesc Represents a Reference.
             * @implements IReference
             * @constructor
             * @param {exocore.index.IReference=} [properties] Properties to set
             */
            function Reference(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Reference entityId.
             * @member {string} entityId
             * @memberof exocore.index.Reference
             * @instance
             */
            Reference.prototype.entityId = "";

            /**
             * Reference traitId.
             * @member {string} traitId
             * @memberof exocore.index.Reference
             * @instance
             */
            Reference.prototype.traitId = "";

            /**
             * Creates a new Reference instance using the specified properties.
             * @function create
             * @memberof exocore.index.Reference
             * @static
             * @param {exocore.index.IReference=} [properties] Properties to set
             * @returns {exocore.index.Reference} Reference instance
             */
            Reference.create = function create(properties) {
                return new Reference(properties);
            };

            /**
             * Encodes the specified Reference message. Does not implicitly {@link exocore.index.Reference.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.Reference
             * @static
             * @param {exocore.index.IReference} message Reference message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Reference.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.entityId != null && Object.hasOwnProperty.call(message, "entityId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.entityId);
                if (message.traitId != null && Object.hasOwnProperty.call(message, "traitId"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.traitId);
                return writer;
            };

            /**
             * Encodes the specified Reference message, length delimited. Does not implicitly {@link exocore.index.Reference.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.Reference
             * @static
             * @param {exocore.index.IReference} message Reference message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Reference.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a Reference message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.Reference
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.Reference} Reference
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Reference.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.Reference();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.entityId = reader.string();
                        break;
                    case 2:
                        message.traitId = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a Reference message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.Reference
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.Reference} Reference
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Reference.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a Reference message.
             * @function verify
             * @memberof exocore.index.Reference
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Reference.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.entityId != null && message.hasOwnProperty("entityId"))
                    if (!$util.isString(message.entityId))
                        return "entityId: string expected";
                if (message.traitId != null && message.hasOwnProperty("traitId"))
                    if (!$util.isString(message.traitId))
                        return "traitId: string expected";
                return null;
            };

            /**
             * Creates a Reference message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.Reference
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.Reference} Reference
             */
            Reference.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.Reference)
                    return object;
                let message = new $root.exocore.index.Reference();
                if (object.entityId != null)
                    message.entityId = String(object.entityId);
                if (object.traitId != null)
                    message.traitId = String(object.traitId);
                return message;
            };

            /**
             * Creates a plain object from a Reference message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.Reference
             * @static
             * @param {exocore.index.Reference} message Reference
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Reference.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.entityId = "";
                    object.traitId = "";
                }
                if (message.entityId != null && message.hasOwnProperty("entityId"))
                    object.entityId = message.entityId;
                if (message.traitId != null && message.hasOwnProperty("traitId"))
                    object.traitId = message.traitId;
                return object;
            };

            /**
             * Converts this Reference to JSON.
             * @function toJSON
             * @memberof exocore.index.Reference
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Reference.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return Reference;
        })();

        index.MutationRequest = (function() {

            /**
             * Properties of a MutationRequest.
             * @memberof exocore.index
             * @interface IMutationRequest
             * @property {Array.<exocore.index.IEntityMutation>|null} [mutations] Mutations to apply.
             * @property {boolean|null} [waitIndexed] Waits for mutation to be indexed.
             * @property {boolean|null} [returnEntities] Waits for mutation to be indexed and returns the mutated entities.
             */

            /**
             * Constructs a new MutationRequest.
             * @memberof exocore.index
             * @classdesc Represents a MutationRequest.
             * @implements IMutationRequest
             * @constructor
             * @param {exocore.index.IMutationRequest=} [properties] Properties to set
             */
            function MutationRequest(properties) {
                this.mutations = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Mutations to apply.
             * @member {Array.<exocore.index.IEntityMutation>} mutations
             * @memberof exocore.index.MutationRequest
             * @instance
             */
            MutationRequest.prototype.mutations = $util.emptyArray;

            /**
             * Waits for mutation to be indexed.
             * @member {boolean} waitIndexed
             * @memberof exocore.index.MutationRequest
             * @instance
             */
            MutationRequest.prototype.waitIndexed = false;

            /**
             * Waits for mutation to be indexed and returns the mutated entities.
             * @member {boolean} returnEntities
             * @memberof exocore.index.MutationRequest
             * @instance
             */
            MutationRequest.prototype.returnEntities = false;

            /**
             * Creates a new MutationRequest instance using the specified properties.
             * @function create
             * @memberof exocore.index.MutationRequest
             * @static
             * @param {exocore.index.IMutationRequest=} [properties] Properties to set
             * @returns {exocore.index.MutationRequest} MutationRequest instance
             */
            MutationRequest.create = function create(properties) {
                return new MutationRequest(properties);
            };

            /**
             * Encodes the specified MutationRequest message. Does not implicitly {@link exocore.index.MutationRequest.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.MutationRequest
             * @static
             * @param {exocore.index.IMutationRequest} message MutationRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MutationRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.mutations != null && message.mutations.length)
                    for (let i = 0; i < message.mutations.length; ++i)
                        $root.exocore.index.EntityMutation.encode(message.mutations[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.waitIndexed != null && Object.hasOwnProperty.call(message, "waitIndexed"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.waitIndexed);
                if (message.returnEntities != null && Object.hasOwnProperty.call(message, "returnEntities"))
                    writer.uint32(/* id 3, wireType 0 =*/24).bool(message.returnEntities);
                return writer;
            };

            /**
             * Encodes the specified MutationRequest message, length delimited. Does not implicitly {@link exocore.index.MutationRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.MutationRequest
             * @static
             * @param {exocore.index.IMutationRequest} message MutationRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MutationRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a MutationRequest message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.MutationRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.MutationRequest} MutationRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MutationRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.MutationRequest();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.mutations && message.mutations.length))
                            message.mutations = [];
                        message.mutations.push($root.exocore.index.EntityMutation.decode(reader, reader.uint32()));
                        break;
                    case 2:
                        message.waitIndexed = reader.bool();
                        break;
                    case 3:
                        message.returnEntities = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a MutationRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.MutationRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.MutationRequest} MutationRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MutationRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a MutationRequest message.
             * @function verify
             * @memberof exocore.index.MutationRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            MutationRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.mutations != null && message.hasOwnProperty("mutations")) {
                    if (!Array.isArray(message.mutations))
                        return "mutations: array expected";
                    for (let i = 0; i < message.mutations.length; ++i) {
                        let error = $root.exocore.index.EntityMutation.verify(message.mutations[i]);
                        if (error)
                            return "mutations." + error;
                    }
                }
                if (message.waitIndexed != null && message.hasOwnProperty("waitIndexed"))
                    if (typeof message.waitIndexed !== "boolean")
                        return "waitIndexed: boolean expected";
                if (message.returnEntities != null && message.hasOwnProperty("returnEntities"))
                    if (typeof message.returnEntities !== "boolean")
                        return "returnEntities: boolean expected";
                return null;
            };

            /**
             * Creates a MutationRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.MutationRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.MutationRequest} MutationRequest
             */
            MutationRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.MutationRequest)
                    return object;
                let message = new $root.exocore.index.MutationRequest();
                if (object.mutations) {
                    if (!Array.isArray(object.mutations))
                        throw TypeError(".exocore.index.MutationRequest.mutations: array expected");
                    message.mutations = [];
                    for (let i = 0; i < object.mutations.length; ++i) {
                        if (typeof object.mutations[i] !== "object")
                            throw TypeError(".exocore.index.MutationRequest.mutations: object expected");
                        message.mutations[i] = $root.exocore.index.EntityMutation.fromObject(object.mutations[i]);
                    }
                }
                if (object.waitIndexed != null)
                    message.waitIndexed = Boolean(object.waitIndexed);
                if (object.returnEntities != null)
                    message.returnEntities = Boolean(object.returnEntities);
                return message;
            };

            /**
             * Creates a plain object from a MutationRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.MutationRequest
             * @static
             * @param {exocore.index.MutationRequest} message MutationRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            MutationRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.mutations = [];
                if (options.defaults) {
                    object.waitIndexed = false;
                    object.returnEntities = false;
                }
                if (message.mutations && message.mutations.length) {
                    object.mutations = [];
                    for (let j = 0; j < message.mutations.length; ++j)
                        object.mutations[j] = $root.exocore.index.EntityMutation.toObject(message.mutations[j], options);
                }
                if (message.waitIndexed != null && message.hasOwnProperty("waitIndexed"))
                    object.waitIndexed = message.waitIndexed;
                if (message.returnEntities != null && message.hasOwnProperty("returnEntities"))
                    object.returnEntities = message.returnEntities;
                return object;
            };

            /**
             * Converts this MutationRequest to JSON.
             * @function toJSON
             * @memberof exocore.index.MutationRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            MutationRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return MutationRequest;
        })();

        index.MutationResult = (function() {

            /**
             * Properties of a MutationResult.
             * @memberof exocore.index
             * @interface IMutationResult
             * @property {Array.<number|Long>|null} [operationIds] Unique operation ids for each mutations.
             * @property {Array.<exocore.index.IEntity>|null} [entities] Mutated entities if requested.
             */

            /**
             * Constructs a new MutationResult.
             * @memberof exocore.index
             * @classdesc Represents a MutationResult.
             * @implements IMutationResult
             * @constructor
             * @param {exocore.index.IMutationResult=} [properties] Properties to set
             */
            function MutationResult(properties) {
                this.operationIds = [];
                this.entities = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Unique operation ids for each mutations.
             * @member {Array.<number|Long>} operationIds
             * @memberof exocore.index.MutationResult
             * @instance
             */
            MutationResult.prototype.operationIds = $util.emptyArray;

            /**
             * Mutated entities if requested.
             * @member {Array.<exocore.index.IEntity>} entities
             * @memberof exocore.index.MutationResult
             * @instance
             */
            MutationResult.prototype.entities = $util.emptyArray;

            /**
             * Creates a new MutationResult instance using the specified properties.
             * @function create
             * @memberof exocore.index.MutationResult
             * @static
             * @param {exocore.index.IMutationResult=} [properties] Properties to set
             * @returns {exocore.index.MutationResult} MutationResult instance
             */
            MutationResult.create = function create(properties) {
                return new MutationResult(properties);
            };

            /**
             * Encodes the specified MutationResult message. Does not implicitly {@link exocore.index.MutationResult.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.MutationResult
             * @static
             * @param {exocore.index.IMutationResult} message MutationResult message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MutationResult.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.operationIds != null && message.operationIds.length) {
                    writer.uint32(/* id 1, wireType 2 =*/10).fork();
                    for (let i = 0; i < message.operationIds.length; ++i)
                        writer.uint64(message.operationIds[i]);
                    writer.ldelim();
                }
                if (message.entities != null && message.entities.length)
                    for (let i = 0; i < message.entities.length; ++i)
                        $root.exocore.index.Entity.encode(message.entities[i], writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified MutationResult message, length delimited. Does not implicitly {@link exocore.index.MutationResult.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.MutationResult
             * @static
             * @param {exocore.index.IMutationResult} message MutationResult message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MutationResult.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a MutationResult message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.MutationResult
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.MutationResult} MutationResult
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MutationResult.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.MutationResult();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.operationIds && message.operationIds.length))
                            message.operationIds = [];
                        if ((tag & 7) === 2) {
                            let end2 = reader.uint32() + reader.pos;
                            while (reader.pos < end2)
                                message.operationIds.push(reader.uint64());
                        } else
                            message.operationIds.push(reader.uint64());
                        break;
                    case 2:
                        if (!(message.entities && message.entities.length))
                            message.entities = [];
                        message.entities.push($root.exocore.index.Entity.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a MutationResult message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.MutationResult
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.MutationResult} MutationResult
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MutationResult.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a MutationResult message.
             * @function verify
             * @memberof exocore.index.MutationResult
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            MutationResult.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.operationIds != null && message.hasOwnProperty("operationIds")) {
                    if (!Array.isArray(message.operationIds))
                        return "operationIds: array expected";
                    for (let i = 0; i < message.operationIds.length; ++i)
                        if (!$util.isInteger(message.operationIds[i]) && !(message.operationIds[i] && $util.isInteger(message.operationIds[i].low) && $util.isInteger(message.operationIds[i].high)))
                            return "operationIds: integer|Long[] expected";
                }
                if (message.entities != null && message.hasOwnProperty("entities")) {
                    if (!Array.isArray(message.entities))
                        return "entities: array expected";
                    for (let i = 0; i < message.entities.length; ++i) {
                        let error = $root.exocore.index.Entity.verify(message.entities[i]);
                        if (error)
                            return "entities." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a MutationResult message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.MutationResult
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.MutationResult} MutationResult
             */
            MutationResult.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.MutationResult)
                    return object;
                let message = new $root.exocore.index.MutationResult();
                if (object.operationIds) {
                    if (!Array.isArray(object.operationIds))
                        throw TypeError(".exocore.index.MutationResult.operationIds: array expected");
                    message.operationIds = [];
                    for (let i = 0; i < object.operationIds.length; ++i)
                        if ($util.Long)
                            (message.operationIds[i] = $util.Long.fromValue(object.operationIds[i])).unsigned = true;
                        else if (typeof object.operationIds[i] === "string")
                            message.operationIds[i] = parseInt(object.operationIds[i], 10);
                        else if (typeof object.operationIds[i] === "number")
                            message.operationIds[i] = object.operationIds[i];
                        else if (typeof object.operationIds[i] === "object")
                            message.operationIds[i] = new $util.LongBits(object.operationIds[i].low >>> 0, object.operationIds[i].high >>> 0).toNumber(true);
                }
                if (object.entities) {
                    if (!Array.isArray(object.entities))
                        throw TypeError(".exocore.index.MutationResult.entities: array expected");
                    message.entities = [];
                    for (let i = 0; i < object.entities.length; ++i) {
                        if (typeof object.entities[i] !== "object")
                            throw TypeError(".exocore.index.MutationResult.entities: object expected");
                        message.entities[i] = $root.exocore.index.Entity.fromObject(object.entities[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a MutationResult message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.MutationResult
             * @static
             * @param {exocore.index.MutationResult} message MutationResult
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            MutationResult.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults) {
                    object.operationIds = [];
                    object.entities = [];
                }
                if (message.operationIds && message.operationIds.length) {
                    object.operationIds = [];
                    for (let j = 0; j < message.operationIds.length; ++j)
                        if (typeof message.operationIds[j] === "number")
                            object.operationIds[j] = options.longs === String ? String(message.operationIds[j]) : message.operationIds[j];
                        else
                            object.operationIds[j] = options.longs === String ? $util.Long.prototype.toString.call(message.operationIds[j]) : options.longs === Number ? new $util.LongBits(message.operationIds[j].low >>> 0, message.operationIds[j].high >>> 0).toNumber(true) : message.operationIds[j];
                }
                if (message.entities && message.entities.length) {
                    object.entities = [];
                    for (let j = 0; j < message.entities.length; ++j)
                        object.entities[j] = $root.exocore.index.Entity.toObject(message.entities[j], options);
                }
                return object;
            };

            /**
             * Converts this MutationResult to JSON.
             * @function toJSON
             * @memberof exocore.index.MutationResult
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            MutationResult.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return MutationResult;
        })();

        index.EntityMutation = (function() {

            /**
             * Properties of an EntityMutation.
             * @memberof exocore.index
             * @interface IEntityMutation
             * @property {string|null} [entityId] EntityMutation entityId
             * @property {exocore.index.IPutTraitMutation|null} [putTrait] EntityMutation putTrait
             * @property {exocore.index.IDeleteTraitMutation|null} [deleteTrait] EntityMutation deleteTrait
             * @property {exocore.index.IDeleteEntityMutation|null} [deleteEntity] EntityMutation deleteEntity
             * @property {exocore.index.IUpdateTraitMutation|null} [updateTrait] EntityMutation updateTrait
             * @property {exocore.index.ICompactTraitMutation|null} [compactTrait] EntityMutation compactTrait
             * @property {exocore.index.ITestMutation|null} [test] EntityMutation test
             */

            /**
             * Constructs a new EntityMutation.
             * @memberof exocore.index
             * @classdesc Represents an EntityMutation.
             * @implements IEntityMutation
             * @constructor
             * @param {exocore.index.IEntityMutation=} [properties] Properties to set
             */
            function EntityMutation(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * EntityMutation entityId.
             * @member {string} entityId
             * @memberof exocore.index.EntityMutation
             * @instance
             */
            EntityMutation.prototype.entityId = "";

            /**
             * EntityMutation putTrait.
             * @member {exocore.index.IPutTraitMutation|null|undefined} putTrait
             * @memberof exocore.index.EntityMutation
             * @instance
             */
            EntityMutation.prototype.putTrait = null;

            /**
             * EntityMutation deleteTrait.
             * @member {exocore.index.IDeleteTraitMutation|null|undefined} deleteTrait
             * @memberof exocore.index.EntityMutation
             * @instance
             */
            EntityMutation.prototype.deleteTrait = null;

            /**
             * EntityMutation deleteEntity.
             * @member {exocore.index.IDeleteEntityMutation|null|undefined} deleteEntity
             * @memberof exocore.index.EntityMutation
             * @instance
             */
            EntityMutation.prototype.deleteEntity = null;

            /**
             * EntityMutation updateTrait.
             * @member {exocore.index.IUpdateTraitMutation|null|undefined} updateTrait
             * @memberof exocore.index.EntityMutation
             * @instance
             */
            EntityMutation.prototype.updateTrait = null;

            /**
             * EntityMutation compactTrait.
             * @member {exocore.index.ICompactTraitMutation|null|undefined} compactTrait
             * @memberof exocore.index.EntityMutation
             * @instance
             */
            EntityMutation.prototype.compactTrait = null;

            /**
             * EntityMutation test.
             * @member {exocore.index.ITestMutation|null|undefined} test
             * @memberof exocore.index.EntityMutation
             * @instance
             */
            EntityMutation.prototype.test = null;

            // OneOf field names bound to virtual getters and setters
            let $oneOfFields;

            /**
             * EntityMutation mutation.
             * @member {"putTrait"|"deleteTrait"|"deleteEntity"|"updateTrait"|"compactTrait"|"test"|undefined} mutation
             * @memberof exocore.index.EntityMutation
             * @instance
             */
            Object.defineProperty(EntityMutation.prototype, "mutation", {
                get: $util.oneOfGetter($oneOfFields = ["putTrait", "deleteTrait", "deleteEntity", "updateTrait", "compactTrait", "test"]),
                set: $util.oneOfSetter($oneOfFields)
            });

            /**
             * Creates a new EntityMutation instance using the specified properties.
             * @function create
             * @memberof exocore.index.EntityMutation
             * @static
             * @param {exocore.index.IEntityMutation=} [properties] Properties to set
             * @returns {exocore.index.EntityMutation} EntityMutation instance
             */
            EntityMutation.create = function create(properties) {
                return new EntityMutation(properties);
            };

            /**
             * Encodes the specified EntityMutation message. Does not implicitly {@link exocore.index.EntityMutation.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.EntityMutation
             * @static
             * @param {exocore.index.IEntityMutation} message EntityMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EntityMutation.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.entityId != null && Object.hasOwnProperty.call(message, "entityId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.entityId);
                if (message.putTrait != null && Object.hasOwnProperty.call(message, "putTrait"))
                    $root.exocore.index.PutTraitMutation.encode(message.putTrait, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.deleteTrait != null && Object.hasOwnProperty.call(message, "deleteTrait"))
                    $root.exocore.index.DeleteTraitMutation.encode(message.deleteTrait, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.deleteEntity != null && Object.hasOwnProperty.call(message, "deleteEntity"))
                    $root.exocore.index.DeleteEntityMutation.encode(message.deleteEntity, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.updateTrait != null && Object.hasOwnProperty.call(message, "updateTrait"))
                    $root.exocore.index.UpdateTraitMutation.encode(message.updateTrait, writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                if (message.compactTrait != null && Object.hasOwnProperty.call(message, "compactTrait"))
                    $root.exocore.index.CompactTraitMutation.encode(message.compactTrait, writer.uint32(/* id 6, wireType 2 =*/50).fork()).ldelim();
                if (message.test != null && Object.hasOwnProperty.call(message, "test"))
                    $root.exocore.index.TestMutation.encode(message.test, writer.uint32(/* id 99, wireType 2 =*/794).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified EntityMutation message, length delimited. Does not implicitly {@link exocore.index.EntityMutation.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.EntityMutation
             * @static
             * @param {exocore.index.IEntityMutation} message EntityMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EntityMutation.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an EntityMutation message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.EntityMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.EntityMutation} EntityMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EntityMutation.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.EntityMutation();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.entityId = reader.string();
                        break;
                    case 2:
                        message.putTrait = $root.exocore.index.PutTraitMutation.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.deleteTrait = $root.exocore.index.DeleteTraitMutation.decode(reader, reader.uint32());
                        break;
                    case 4:
                        message.deleteEntity = $root.exocore.index.DeleteEntityMutation.decode(reader, reader.uint32());
                        break;
                    case 5:
                        message.updateTrait = $root.exocore.index.UpdateTraitMutation.decode(reader, reader.uint32());
                        break;
                    case 6:
                        message.compactTrait = $root.exocore.index.CompactTraitMutation.decode(reader, reader.uint32());
                        break;
                    case 99:
                        message.test = $root.exocore.index.TestMutation.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an EntityMutation message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.EntityMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.EntityMutation} EntityMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EntityMutation.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an EntityMutation message.
             * @function verify
             * @memberof exocore.index.EntityMutation
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            EntityMutation.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                let properties = {};
                if (message.entityId != null && message.hasOwnProperty("entityId"))
                    if (!$util.isString(message.entityId))
                        return "entityId: string expected";
                if (message.putTrait != null && message.hasOwnProperty("putTrait")) {
                    properties.mutation = 1;
                    {
                        let error = $root.exocore.index.PutTraitMutation.verify(message.putTrait);
                        if (error)
                            return "putTrait." + error;
                    }
                }
                if (message.deleteTrait != null && message.hasOwnProperty("deleteTrait")) {
                    if (properties.mutation === 1)
                        return "mutation: multiple values";
                    properties.mutation = 1;
                    {
                        let error = $root.exocore.index.DeleteTraitMutation.verify(message.deleteTrait);
                        if (error)
                            return "deleteTrait." + error;
                    }
                }
                if (message.deleteEntity != null && message.hasOwnProperty("deleteEntity")) {
                    if (properties.mutation === 1)
                        return "mutation: multiple values";
                    properties.mutation = 1;
                    {
                        let error = $root.exocore.index.DeleteEntityMutation.verify(message.deleteEntity);
                        if (error)
                            return "deleteEntity." + error;
                    }
                }
                if (message.updateTrait != null && message.hasOwnProperty("updateTrait")) {
                    if (properties.mutation === 1)
                        return "mutation: multiple values";
                    properties.mutation = 1;
                    {
                        let error = $root.exocore.index.UpdateTraitMutation.verify(message.updateTrait);
                        if (error)
                            return "updateTrait." + error;
                    }
                }
                if (message.compactTrait != null && message.hasOwnProperty("compactTrait")) {
                    if (properties.mutation === 1)
                        return "mutation: multiple values";
                    properties.mutation = 1;
                    {
                        let error = $root.exocore.index.CompactTraitMutation.verify(message.compactTrait);
                        if (error)
                            return "compactTrait." + error;
                    }
                }
                if (message.test != null && message.hasOwnProperty("test")) {
                    if (properties.mutation === 1)
                        return "mutation: multiple values";
                    properties.mutation = 1;
                    {
                        let error = $root.exocore.index.TestMutation.verify(message.test);
                        if (error)
                            return "test." + error;
                    }
                }
                return null;
            };

            /**
             * Creates an EntityMutation message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.EntityMutation
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.EntityMutation} EntityMutation
             */
            EntityMutation.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.EntityMutation)
                    return object;
                let message = new $root.exocore.index.EntityMutation();
                if (object.entityId != null)
                    message.entityId = String(object.entityId);
                if (object.putTrait != null) {
                    if (typeof object.putTrait !== "object")
                        throw TypeError(".exocore.index.EntityMutation.putTrait: object expected");
                    message.putTrait = $root.exocore.index.PutTraitMutation.fromObject(object.putTrait);
                }
                if (object.deleteTrait != null) {
                    if (typeof object.deleteTrait !== "object")
                        throw TypeError(".exocore.index.EntityMutation.deleteTrait: object expected");
                    message.deleteTrait = $root.exocore.index.DeleteTraitMutation.fromObject(object.deleteTrait);
                }
                if (object.deleteEntity != null) {
                    if (typeof object.deleteEntity !== "object")
                        throw TypeError(".exocore.index.EntityMutation.deleteEntity: object expected");
                    message.deleteEntity = $root.exocore.index.DeleteEntityMutation.fromObject(object.deleteEntity);
                }
                if (object.updateTrait != null) {
                    if (typeof object.updateTrait !== "object")
                        throw TypeError(".exocore.index.EntityMutation.updateTrait: object expected");
                    message.updateTrait = $root.exocore.index.UpdateTraitMutation.fromObject(object.updateTrait);
                }
                if (object.compactTrait != null) {
                    if (typeof object.compactTrait !== "object")
                        throw TypeError(".exocore.index.EntityMutation.compactTrait: object expected");
                    message.compactTrait = $root.exocore.index.CompactTraitMutation.fromObject(object.compactTrait);
                }
                if (object.test != null) {
                    if (typeof object.test !== "object")
                        throw TypeError(".exocore.index.EntityMutation.test: object expected");
                    message.test = $root.exocore.index.TestMutation.fromObject(object.test);
                }
                return message;
            };

            /**
             * Creates a plain object from an EntityMutation message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.EntityMutation
             * @static
             * @param {exocore.index.EntityMutation} message EntityMutation
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            EntityMutation.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults)
                    object.entityId = "";
                if (message.entityId != null && message.hasOwnProperty("entityId"))
                    object.entityId = message.entityId;
                if (message.putTrait != null && message.hasOwnProperty("putTrait")) {
                    object.putTrait = $root.exocore.index.PutTraitMutation.toObject(message.putTrait, options);
                    if (options.oneofs)
                        object.mutation = "putTrait";
                }
                if (message.deleteTrait != null && message.hasOwnProperty("deleteTrait")) {
                    object.deleteTrait = $root.exocore.index.DeleteTraitMutation.toObject(message.deleteTrait, options);
                    if (options.oneofs)
                        object.mutation = "deleteTrait";
                }
                if (message.deleteEntity != null && message.hasOwnProperty("deleteEntity")) {
                    object.deleteEntity = $root.exocore.index.DeleteEntityMutation.toObject(message.deleteEntity, options);
                    if (options.oneofs)
                        object.mutation = "deleteEntity";
                }
                if (message.updateTrait != null && message.hasOwnProperty("updateTrait")) {
                    object.updateTrait = $root.exocore.index.UpdateTraitMutation.toObject(message.updateTrait, options);
                    if (options.oneofs)
                        object.mutation = "updateTrait";
                }
                if (message.compactTrait != null && message.hasOwnProperty("compactTrait")) {
                    object.compactTrait = $root.exocore.index.CompactTraitMutation.toObject(message.compactTrait, options);
                    if (options.oneofs)
                        object.mutation = "compactTrait";
                }
                if (message.test != null && message.hasOwnProperty("test")) {
                    object.test = $root.exocore.index.TestMutation.toObject(message.test, options);
                    if (options.oneofs)
                        object.mutation = "test";
                }
                return object;
            };

            /**
             * Converts this EntityMutation to JSON.
             * @function toJSON
             * @memberof exocore.index.EntityMutation
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            EntityMutation.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return EntityMutation;
        })();

        index.PutTraitMutation = (function() {

            /**
             * Properties of a PutTraitMutation.
             * @memberof exocore.index
             * @interface IPutTraitMutation
             * @property {exocore.index.ITrait|null} [trait] PutTraitMutation trait
             */

            /**
             * Constructs a new PutTraitMutation.
             * @memberof exocore.index
             * @classdesc Represents a PutTraitMutation.
             * @implements IPutTraitMutation
             * @constructor
             * @param {exocore.index.IPutTraitMutation=} [properties] Properties to set
             */
            function PutTraitMutation(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * PutTraitMutation trait.
             * @member {exocore.index.ITrait|null|undefined} trait
             * @memberof exocore.index.PutTraitMutation
             * @instance
             */
            PutTraitMutation.prototype.trait = null;

            /**
             * Creates a new PutTraitMutation instance using the specified properties.
             * @function create
             * @memberof exocore.index.PutTraitMutation
             * @static
             * @param {exocore.index.IPutTraitMutation=} [properties] Properties to set
             * @returns {exocore.index.PutTraitMutation} PutTraitMutation instance
             */
            PutTraitMutation.create = function create(properties) {
                return new PutTraitMutation(properties);
            };

            /**
             * Encodes the specified PutTraitMutation message. Does not implicitly {@link exocore.index.PutTraitMutation.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.PutTraitMutation
             * @static
             * @param {exocore.index.IPutTraitMutation} message PutTraitMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PutTraitMutation.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.trait != null && Object.hasOwnProperty.call(message, "trait"))
                    $root.exocore.index.Trait.encode(message.trait, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified PutTraitMutation message, length delimited. Does not implicitly {@link exocore.index.PutTraitMutation.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.PutTraitMutation
             * @static
             * @param {exocore.index.IPutTraitMutation} message PutTraitMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PutTraitMutation.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a PutTraitMutation message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.PutTraitMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.PutTraitMutation} PutTraitMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PutTraitMutation.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.PutTraitMutation();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.trait = $root.exocore.index.Trait.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a PutTraitMutation message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.PutTraitMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.PutTraitMutation} PutTraitMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PutTraitMutation.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a PutTraitMutation message.
             * @function verify
             * @memberof exocore.index.PutTraitMutation
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            PutTraitMutation.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.trait != null && message.hasOwnProperty("trait")) {
                    let error = $root.exocore.index.Trait.verify(message.trait);
                    if (error)
                        return "trait." + error;
                }
                return null;
            };

            /**
             * Creates a PutTraitMutation message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.PutTraitMutation
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.PutTraitMutation} PutTraitMutation
             */
            PutTraitMutation.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.PutTraitMutation)
                    return object;
                let message = new $root.exocore.index.PutTraitMutation();
                if (object.trait != null) {
                    if (typeof object.trait !== "object")
                        throw TypeError(".exocore.index.PutTraitMutation.trait: object expected");
                    message.trait = $root.exocore.index.Trait.fromObject(object.trait);
                }
                return message;
            };

            /**
             * Creates a plain object from a PutTraitMutation message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.PutTraitMutation
             * @static
             * @param {exocore.index.PutTraitMutation} message PutTraitMutation
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            PutTraitMutation.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults)
                    object.trait = null;
                if (message.trait != null && message.hasOwnProperty("trait"))
                    object.trait = $root.exocore.index.Trait.toObject(message.trait, options);
                return object;
            };

            /**
             * Converts this PutTraitMutation to JSON.
             * @function toJSON
             * @memberof exocore.index.PutTraitMutation
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            PutTraitMutation.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return PutTraitMutation;
        })();

        index.DeleteTraitMutation = (function() {

            /**
             * Properties of a DeleteTraitMutation.
             * @memberof exocore.index
             * @interface IDeleteTraitMutation
             * @property {string|null} [traitId] DeleteTraitMutation traitId
             */

            /**
             * Constructs a new DeleteTraitMutation.
             * @memberof exocore.index
             * @classdesc Represents a DeleteTraitMutation.
             * @implements IDeleteTraitMutation
             * @constructor
             * @param {exocore.index.IDeleteTraitMutation=} [properties] Properties to set
             */
            function DeleteTraitMutation(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * DeleteTraitMutation traitId.
             * @member {string} traitId
             * @memberof exocore.index.DeleteTraitMutation
             * @instance
             */
            DeleteTraitMutation.prototype.traitId = "";

            /**
             * Creates a new DeleteTraitMutation instance using the specified properties.
             * @function create
             * @memberof exocore.index.DeleteTraitMutation
             * @static
             * @param {exocore.index.IDeleteTraitMutation=} [properties] Properties to set
             * @returns {exocore.index.DeleteTraitMutation} DeleteTraitMutation instance
             */
            DeleteTraitMutation.create = function create(properties) {
                return new DeleteTraitMutation(properties);
            };

            /**
             * Encodes the specified DeleteTraitMutation message. Does not implicitly {@link exocore.index.DeleteTraitMutation.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.DeleteTraitMutation
             * @static
             * @param {exocore.index.IDeleteTraitMutation} message DeleteTraitMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DeleteTraitMutation.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.traitId != null && Object.hasOwnProperty.call(message, "traitId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.traitId);
                return writer;
            };

            /**
             * Encodes the specified DeleteTraitMutation message, length delimited. Does not implicitly {@link exocore.index.DeleteTraitMutation.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.DeleteTraitMutation
             * @static
             * @param {exocore.index.IDeleteTraitMutation} message DeleteTraitMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DeleteTraitMutation.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a DeleteTraitMutation message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.DeleteTraitMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.DeleteTraitMutation} DeleteTraitMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DeleteTraitMutation.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.DeleteTraitMutation();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.traitId = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a DeleteTraitMutation message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.DeleteTraitMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.DeleteTraitMutation} DeleteTraitMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DeleteTraitMutation.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a DeleteTraitMutation message.
             * @function verify
             * @memberof exocore.index.DeleteTraitMutation
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            DeleteTraitMutation.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.traitId != null && message.hasOwnProperty("traitId"))
                    if (!$util.isString(message.traitId))
                        return "traitId: string expected";
                return null;
            };

            /**
             * Creates a DeleteTraitMutation message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.DeleteTraitMutation
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.DeleteTraitMutation} DeleteTraitMutation
             */
            DeleteTraitMutation.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.DeleteTraitMutation)
                    return object;
                let message = new $root.exocore.index.DeleteTraitMutation();
                if (object.traitId != null)
                    message.traitId = String(object.traitId);
                return message;
            };

            /**
             * Creates a plain object from a DeleteTraitMutation message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.DeleteTraitMutation
             * @static
             * @param {exocore.index.DeleteTraitMutation} message DeleteTraitMutation
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            DeleteTraitMutation.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults)
                    object.traitId = "";
                if (message.traitId != null && message.hasOwnProperty("traitId"))
                    object.traitId = message.traitId;
                return object;
            };

            /**
             * Converts this DeleteTraitMutation to JSON.
             * @function toJSON
             * @memberof exocore.index.DeleteTraitMutation
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            DeleteTraitMutation.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return DeleteTraitMutation;
        })();

        index.DeleteEntityMutation = (function() {

            /**
             * Properties of a DeleteEntityMutation.
             * @memberof exocore.index
             * @interface IDeleteEntityMutation
             */

            /**
             * Constructs a new DeleteEntityMutation.
             * @memberof exocore.index
             * @classdesc Represents a DeleteEntityMutation.
             * @implements IDeleteEntityMutation
             * @constructor
             * @param {exocore.index.IDeleteEntityMutation=} [properties] Properties to set
             */
            function DeleteEntityMutation(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Creates a new DeleteEntityMutation instance using the specified properties.
             * @function create
             * @memberof exocore.index.DeleteEntityMutation
             * @static
             * @param {exocore.index.IDeleteEntityMutation=} [properties] Properties to set
             * @returns {exocore.index.DeleteEntityMutation} DeleteEntityMutation instance
             */
            DeleteEntityMutation.create = function create(properties) {
                return new DeleteEntityMutation(properties);
            };

            /**
             * Encodes the specified DeleteEntityMutation message. Does not implicitly {@link exocore.index.DeleteEntityMutation.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.DeleteEntityMutation
             * @static
             * @param {exocore.index.IDeleteEntityMutation} message DeleteEntityMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DeleteEntityMutation.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };

            /**
             * Encodes the specified DeleteEntityMutation message, length delimited. Does not implicitly {@link exocore.index.DeleteEntityMutation.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.DeleteEntityMutation
             * @static
             * @param {exocore.index.IDeleteEntityMutation} message DeleteEntityMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DeleteEntityMutation.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a DeleteEntityMutation message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.DeleteEntityMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.DeleteEntityMutation} DeleteEntityMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DeleteEntityMutation.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.DeleteEntityMutation();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a DeleteEntityMutation message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.DeleteEntityMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.DeleteEntityMutation} DeleteEntityMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DeleteEntityMutation.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a DeleteEntityMutation message.
             * @function verify
             * @memberof exocore.index.DeleteEntityMutation
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            DeleteEntityMutation.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };

            /**
             * Creates a DeleteEntityMutation message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.DeleteEntityMutation
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.DeleteEntityMutation} DeleteEntityMutation
             */
            DeleteEntityMutation.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.DeleteEntityMutation)
                    return object;
                return new $root.exocore.index.DeleteEntityMutation();
            };

            /**
             * Creates a plain object from a DeleteEntityMutation message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.DeleteEntityMutation
             * @static
             * @param {exocore.index.DeleteEntityMutation} message DeleteEntityMutation
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            DeleteEntityMutation.toObject = function toObject() {
                return {};
            };

            /**
             * Converts this DeleteEntityMutation to JSON.
             * @function toJSON
             * @memberof exocore.index.DeleteEntityMutation
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            DeleteEntityMutation.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return DeleteEntityMutation;
        })();

        index.UpdateTraitMutation = (function() {

            /**
             * Properties of an UpdateTraitMutation.
             * @memberof exocore.index
             * @interface IUpdateTraitMutation
             * @property {string|null} [traitId] UpdateTraitMutation traitId
             * @property {exocore.index.ITrait|null} [trait] UpdateTraitMutation trait
             * @property {google.protobuf.IFieldMask|null} [fieldMask] UpdateTraitMutation fieldMask
             * @property {number|Long|null} [ifLastOperationId] UpdateTraitMutation ifLastOperationId
             */

            /**
             * Constructs a new UpdateTraitMutation.
             * @memberof exocore.index
             * @classdesc Represents an UpdateTraitMutation.
             * @implements IUpdateTraitMutation
             * @constructor
             * @param {exocore.index.IUpdateTraitMutation=} [properties] Properties to set
             */
            function UpdateTraitMutation(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * UpdateTraitMutation traitId.
             * @member {string} traitId
             * @memberof exocore.index.UpdateTraitMutation
             * @instance
             */
            UpdateTraitMutation.prototype.traitId = "";

            /**
             * UpdateTraitMutation trait.
             * @member {exocore.index.ITrait|null|undefined} trait
             * @memberof exocore.index.UpdateTraitMutation
             * @instance
             */
            UpdateTraitMutation.prototype.trait = null;

            /**
             * UpdateTraitMutation fieldMask.
             * @member {google.protobuf.IFieldMask|null|undefined} fieldMask
             * @memberof exocore.index.UpdateTraitMutation
             * @instance
             */
            UpdateTraitMutation.prototype.fieldMask = null;

            /**
             * UpdateTraitMutation ifLastOperationId.
             * @member {number|Long} ifLastOperationId
             * @memberof exocore.index.UpdateTraitMutation
             * @instance
             */
            UpdateTraitMutation.prototype.ifLastOperationId = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * Creates a new UpdateTraitMutation instance using the specified properties.
             * @function create
             * @memberof exocore.index.UpdateTraitMutation
             * @static
             * @param {exocore.index.IUpdateTraitMutation=} [properties] Properties to set
             * @returns {exocore.index.UpdateTraitMutation} UpdateTraitMutation instance
             */
            UpdateTraitMutation.create = function create(properties) {
                return new UpdateTraitMutation(properties);
            };

            /**
             * Encodes the specified UpdateTraitMutation message. Does not implicitly {@link exocore.index.UpdateTraitMutation.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.UpdateTraitMutation
             * @static
             * @param {exocore.index.IUpdateTraitMutation} message UpdateTraitMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            UpdateTraitMutation.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.traitId != null && Object.hasOwnProperty.call(message, "traitId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.traitId);
                if (message.trait != null && Object.hasOwnProperty.call(message, "trait"))
                    $root.exocore.index.Trait.encode(message.trait, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.fieldMask != null && Object.hasOwnProperty.call(message, "fieldMask"))
                    $root.google.protobuf.FieldMask.encode(message.fieldMask, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.ifLastOperationId != null && Object.hasOwnProperty.call(message, "ifLastOperationId"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.ifLastOperationId);
                return writer;
            };

            /**
             * Encodes the specified UpdateTraitMutation message, length delimited. Does not implicitly {@link exocore.index.UpdateTraitMutation.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.UpdateTraitMutation
             * @static
             * @param {exocore.index.IUpdateTraitMutation} message UpdateTraitMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            UpdateTraitMutation.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an UpdateTraitMutation message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.UpdateTraitMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.UpdateTraitMutation} UpdateTraitMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            UpdateTraitMutation.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.UpdateTraitMutation();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.traitId = reader.string();
                        break;
                    case 2:
                        message.trait = $root.exocore.index.Trait.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.fieldMask = $root.google.protobuf.FieldMask.decode(reader, reader.uint32());
                        break;
                    case 4:
                        message.ifLastOperationId = reader.uint64();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an UpdateTraitMutation message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.UpdateTraitMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.UpdateTraitMutation} UpdateTraitMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            UpdateTraitMutation.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an UpdateTraitMutation message.
             * @function verify
             * @memberof exocore.index.UpdateTraitMutation
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            UpdateTraitMutation.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.traitId != null && message.hasOwnProperty("traitId"))
                    if (!$util.isString(message.traitId))
                        return "traitId: string expected";
                if (message.trait != null && message.hasOwnProperty("trait")) {
                    let error = $root.exocore.index.Trait.verify(message.trait);
                    if (error)
                        return "trait." + error;
                }
                if (message.fieldMask != null && message.hasOwnProperty("fieldMask")) {
                    let error = $root.google.protobuf.FieldMask.verify(message.fieldMask);
                    if (error)
                        return "fieldMask." + error;
                }
                if (message.ifLastOperationId != null && message.hasOwnProperty("ifLastOperationId"))
                    if (!$util.isInteger(message.ifLastOperationId) && !(message.ifLastOperationId && $util.isInteger(message.ifLastOperationId.low) && $util.isInteger(message.ifLastOperationId.high)))
                        return "ifLastOperationId: integer|Long expected";
                return null;
            };

            /**
             * Creates an UpdateTraitMutation message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.UpdateTraitMutation
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.UpdateTraitMutation} UpdateTraitMutation
             */
            UpdateTraitMutation.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.UpdateTraitMutation)
                    return object;
                let message = new $root.exocore.index.UpdateTraitMutation();
                if (object.traitId != null)
                    message.traitId = String(object.traitId);
                if (object.trait != null) {
                    if (typeof object.trait !== "object")
                        throw TypeError(".exocore.index.UpdateTraitMutation.trait: object expected");
                    message.trait = $root.exocore.index.Trait.fromObject(object.trait);
                }
                if (object.fieldMask != null) {
                    if (typeof object.fieldMask !== "object")
                        throw TypeError(".exocore.index.UpdateTraitMutation.fieldMask: object expected");
                    message.fieldMask = $root.google.protobuf.FieldMask.fromObject(object.fieldMask);
                }
                if (object.ifLastOperationId != null)
                    if ($util.Long)
                        (message.ifLastOperationId = $util.Long.fromValue(object.ifLastOperationId)).unsigned = true;
                    else if (typeof object.ifLastOperationId === "string")
                        message.ifLastOperationId = parseInt(object.ifLastOperationId, 10);
                    else if (typeof object.ifLastOperationId === "number")
                        message.ifLastOperationId = object.ifLastOperationId;
                    else if (typeof object.ifLastOperationId === "object")
                        message.ifLastOperationId = new $util.LongBits(object.ifLastOperationId.low >>> 0, object.ifLastOperationId.high >>> 0).toNumber(true);
                return message;
            };

            /**
             * Creates a plain object from an UpdateTraitMutation message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.UpdateTraitMutation
             * @static
             * @param {exocore.index.UpdateTraitMutation} message UpdateTraitMutation
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            UpdateTraitMutation.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.traitId = "";
                    object.trait = null;
                    object.fieldMask = null;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.ifLastOperationId = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.ifLastOperationId = options.longs === String ? "0" : 0;
                }
                if (message.traitId != null && message.hasOwnProperty("traitId"))
                    object.traitId = message.traitId;
                if (message.trait != null && message.hasOwnProperty("trait"))
                    object.trait = $root.exocore.index.Trait.toObject(message.trait, options);
                if (message.fieldMask != null && message.hasOwnProperty("fieldMask"))
                    object.fieldMask = $root.google.protobuf.FieldMask.toObject(message.fieldMask, options);
                if (message.ifLastOperationId != null && message.hasOwnProperty("ifLastOperationId"))
                    if (typeof message.ifLastOperationId === "number")
                        object.ifLastOperationId = options.longs === String ? String(message.ifLastOperationId) : message.ifLastOperationId;
                    else
                        object.ifLastOperationId = options.longs === String ? $util.Long.prototype.toString.call(message.ifLastOperationId) : options.longs === Number ? new $util.LongBits(message.ifLastOperationId.low >>> 0, message.ifLastOperationId.high >>> 0).toNumber(true) : message.ifLastOperationId;
                return object;
            };

            /**
             * Converts this UpdateTraitMutation to JSON.
             * @function toJSON
             * @memberof exocore.index.UpdateTraitMutation
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            UpdateTraitMutation.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return UpdateTraitMutation;
        })();

        index.CompactTraitMutation = (function() {

            /**
             * Properties of a CompactTraitMutation.
             * @memberof exocore.index
             * @interface ICompactTraitMutation
             * @property {Array.<exocore.index.CompactTraitMutation.IOperation>|null} [compactedOperations] CompactTraitMutation compactedOperations
             * @property {exocore.index.ITrait|null} [trait] CompactTraitMutation trait
             */

            /**
             * Constructs a new CompactTraitMutation.
             * @memberof exocore.index
             * @classdesc Represents a CompactTraitMutation.
             * @implements ICompactTraitMutation
             * @constructor
             * @param {exocore.index.ICompactTraitMutation=} [properties] Properties to set
             */
            function CompactTraitMutation(properties) {
                this.compactedOperations = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * CompactTraitMutation compactedOperations.
             * @member {Array.<exocore.index.CompactTraitMutation.IOperation>} compactedOperations
             * @memberof exocore.index.CompactTraitMutation
             * @instance
             */
            CompactTraitMutation.prototype.compactedOperations = $util.emptyArray;

            /**
             * CompactTraitMutation trait.
             * @member {exocore.index.ITrait|null|undefined} trait
             * @memberof exocore.index.CompactTraitMutation
             * @instance
             */
            CompactTraitMutation.prototype.trait = null;

            /**
             * Creates a new CompactTraitMutation instance using the specified properties.
             * @function create
             * @memberof exocore.index.CompactTraitMutation
             * @static
             * @param {exocore.index.ICompactTraitMutation=} [properties] Properties to set
             * @returns {exocore.index.CompactTraitMutation} CompactTraitMutation instance
             */
            CompactTraitMutation.create = function create(properties) {
                return new CompactTraitMutation(properties);
            };

            /**
             * Encodes the specified CompactTraitMutation message. Does not implicitly {@link exocore.index.CompactTraitMutation.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.CompactTraitMutation
             * @static
             * @param {exocore.index.ICompactTraitMutation} message CompactTraitMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            CompactTraitMutation.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.compactedOperations != null && message.compactedOperations.length)
                    for (let i = 0; i < message.compactedOperations.length; ++i)
                        $root.exocore.index.CompactTraitMutation.Operation.encode(message.compactedOperations[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.trait != null && Object.hasOwnProperty.call(message, "trait"))
                    $root.exocore.index.Trait.encode(message.trait, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified CompactTraitMutation message, length delimited. Does not implicitly {@link exocore.index.CompactTraitMutation.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.CompactTraitMutation
             * @static
             * @param {exocore.index.ICompactTraitMutation} message CompactTraitMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            CompactTraitMutation.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a CompactTraitMutation message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.CompactTraitMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.CompactTraitMutation} CompactTraitMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            CompactTraitMutation.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.CompactTraitMutation();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.compactedOperations && message.compactedOperations.length))
                            message.compactedOperations = [];
                        message.compactedOperations.push($root.exocore.index.CompactTraitMutation.Operation.decode(reader, reader.uint32()));
                        break;
                    case 2:
                        message.trait = $root.exocore.index.Trait.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a CompactTraitMutation message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.CompactTraitMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.CompactTraitMutation} CompactTraitMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            CompactTraitMutation.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a CompactTraitMutation message.
             * @function verify
             * @memberof exocore.index.CompactTraitMutation
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            CompactTraitMutation.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.compactedOperations != null && message.hasOwnProperty("compactedOperations")) {
                    if (!Array.isArray(message.compactedOperations))
                        return "compactedOperations: array expected";
                    for (let i = 0; i < message.compactedOperations.length; ++i) {
                        let error = $root.exocore.index.CompactTraitMutation.Operation.verify(message.compactedOperations[i]);
                        if (error)
                            return "compactedOperations." + error;
                    }
                }
                if (message.trait != null && message.hasOwnProperty("trait")) {
                    let error = $root.exocore.index.Trait.verify(message.trait);
                    if (error)
                        return "trait." + error;
                }
                return null;
            };

            /**
             * Creates a CompactTraitMutation message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.CompactTraitMutation
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.CompactTraitMutation} CompactTraitMutation
             */
            CompactTraitMutation.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.CompactTraitMutation)
                    return object;
                let message = new $root.exocore.index.CompactTraitMutation();
                if (object.compactedOperations) {
                    if (!Array.isArray(object.compactedOperations))
                        throw TypeError(".exocore.index.CompactTraitMutation.compactedOperations: array expected");
                    message.compactedOperations = [];
                    for (let i = 0; i < object.compactedOperations.length; ++i) {
                        if (typeof object.compactedOperations[i] !== "object")
                            throw TypeError(".exocore.index.CompactTraitMutation.compactedOperations: object expected");
                        message.compactedOperations[i] = $root.exocore.index.CompactTraitMutation.Operation.fromObject(object.compactedOperations[i]);
                    }
                }
                if (object.trait != null) {
                    if (typeof object.trait !== "object")
                        throw TypeError(".exocore.index.CompactTraitMutation.trait: object expected");
                    message.trait = $root.exocore.index.Trait.fromObject(object.trait);
                }
                return message;
            };

            /**
             * Creates a plain object from a CompactTraitMutation message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.CompactTraitMutation
             * @static
             * @param {exocore.index.CompactTraitMutation} message CompactTraitMutation
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            CompactTraitMutation.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.compactedOperations = [];
                if (options.defaults)
                    object.trait = null;
                if (message.compactedOperations && message.compactedOperations.length) {
                    object.compactedOperations = [];
                    for (let j = 0; j < message.compactedOperations.length; ++j)
                        object.compactedOperations[j] = $root.exocore.index.CompactTraitMutation.Operation.toObject(message.compactedOperations[j], options);
                }
                if (message.trait != null && message.hasOwnProperty("trait"))
                    object.trait = $root.exocore.index.Trait.toObject(message.trait, options);
                return object;
            };

            /**
             * Converts this CompactTraitMutation to JSON.
             * @function toJSON
             * @memberof exocore.index.CompactTraitMutation
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            CompactTraitMutation.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            CompactTraitMutation.Operation = (function() {

                /**
                 * Properties of an Operation.
                 * @memberof exocore.index.CompactTraitMutation
                 * @interface IOperation
                 * @property {number|Long|null} [operationId] Operation operationId
                 */

                /**
                 * Constructs a new Operation.
                 * @memberof exocore.index.CompactTraitMutation
                 * @classdesc Represents an Operation.
                 * @implements IOperation
                 * @constructor
                 * @param {exocore.index.CompactTraitMutation.IOperation=} [properties] Properties to set
                 */
                function Operation(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Operation operationId.
                 * @member {number|Long} operationId
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @instance
                 */
                Operation.prototype.operationId = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

                /**
                 * Creates a new Operation instance using the specified properties.
                 * @function create
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @static
                 * @param {exocore.index.CompactTraitMutation.IOperation=} [properties] Properties to set
                 * @returns {exocore.index.CompactTraitMutation.Operation} Operation instance
                 */
                Operation.create = function create(properties) {
                    return new Operation(properties);
                };

                /**
                 * Encodes the specified Operation message. Does not implicitly {@link exocore.index.CompactTraitMutation.Operation.verify|verify} messages.
                 * @function encode
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @static
                 * @param {exocore.index.CompactTraitMutation.IOperation} message Operation message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Operation.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.operationId != null && Object.hasOwnProperty.call(message, "operationId"))
                        writer.uint32(/* id 1, wireType 0 =*/8).uint64(message.operationId);
                    return writer;
                };

                /**
                 * Encodes the specified Operation message, length delimited. Does not implicitly {@link exocore.index.CompactTraitMutation.Operation.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @static
                 * @param {exocore.index.CompactTraitMutation.IOperation} message Operation message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Operation.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes an Operation message from the specified reader or buffer.
                 * @function decode
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exocore.index.CompactTraitMutation.Operation} Operation
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Operation.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.CompactTraitMutation.Operation();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1:
                            message.operationId = reader.uint64();
                            break;
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes an Operation message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exocore.index.CompactTraitMutation.Operation} Operation
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Operation.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies an Operation message.
                 * @function verify
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Operation.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.operationId != null && message.hasOwnProperty("operationId"))
                        if (!$util.isInteger(message.operationId) && !(message.operationId && $util.isInteger(message.operationId.low) && $util.isInteger(message.operationId.high)))
                            return "operationId: integer|Long expected";
                    return null;
                };

                /**
                 * Creates an Operation message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exocore.index.CompactTraitMutation.Operation} Operation
                 */
                Operation.fromObject = function fromObject(object) {
                    if (object instanceof $root.exocore.index.CompactTraitMutation.Operation)
                        return object;
                    let message = new $root.exocore.index.CompactTraitMutation.Operation();
                    if (object.operationId != null)
                        if ($util.Long)
                            (message.operationId = $util.Long.fromValue(object.operationId)).unsigned = true;
                        else if (typeof object.operationId === "string")
                            message.operationId = parseInt(object.operationId, 10);
                        else if (typeof object.operationId === "number")
                            message.operationId = object.operationId;
                        else if (typeof object.operationId === "object")
                            message.operationId = new $util.LongBits(object.operationId.low >>> 0, object.operationId.high >>> 0).toNumber(true);
                    return message;
                };

                /**
                 * Creates a plain object from an Operation message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @static
                 * @param {exocore.index.CompactTraitMutation.Operation} message Operation
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Operation.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults)
                        if ($util.Long) {
                            let long = new $util.Long(0, 0, true);
                            object.operationId = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                        } else
                            object.operationId = options.longs === String ? "0" : 0;
                    if (message.operationId != null && message.hasOwnProperty("operationId"))
                        if (typeof message.operationId === "number")
                            object.operationId = options.longs === String ? String(message.operationId) : message.operationId;
                        else
                            object.operationId = options.longs === String ? $util.Long.prototype.toString.call(message.operationId) : options.longs === Number ? new $util.LongBits(message.operationId.low >>> 0, message.operationId.high >>> 0).toNumber(true) : message.operationId;
                    return object;
                };

                /**
                 * Converts this Operation to JSON.
                 * @function toJSON
                 * @memberof exocore.index.CompactTraitMutation.Operation
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Operation.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                return Operation;
            })();

            return CompactTraitMutation;
        })();

        index.TestMutation = (function() {

            /**
             * Properties of a TestMutation.
             * @memberof exocore.index
             * @interface ITestMutation
             * @property {boolean|null} [success] TestMutation success
             */

            /**
             * Constructs a new TestMutation.
             * @memberof exocore.index
             * @classdesc Represents a TestMutation.
             * @implements ITestMutation
             * @constructor
             * @param {exocore.index.ITestMutation=} [properties] Properties to set
             */
            function TestMutation(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TestMutation success.
             * @member {boolean} success
             * @memberof exocore.index.TestMutation
             * @instance
             */
            TestMutation.prototype.success = false;

            /**
             * Creates a new TestMutation instance using the specified properties.
             * @function create
             * @memberof exocore.index.TestMutation
             * @static
             * @param {exocore.index.ITestMutation=} [properties] Properties to set
             * @returns {exocore.index.TestMutation} TestMutation instance
             */
            TestMutation.create = function create(properties) {
                return new TestMutation(properties);
            };

            /**
             * Encodes the specified TestMutation message. Does not implicitly {@link exocore.index.TestMutation.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.TestMutation
             * @static
             * @param {exocore.index.ITestMutation} message TestMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestMutation.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                    writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
                return writer;
            };

            /**
             * Encodes the specified TestMutation message, length delimited. Does not implicitly {@link exocore.index.TestMutation.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.TestMutation
             * @static
             * @param {exocore.index.ITestMutation} message TestMutation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestMutation.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TestMutation message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.TestMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.TestMutation} TestMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestMutation.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.TestMutation();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.success = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a TestMutation message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.TestMutation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.TestMutation} TestMutation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestMutation.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TestMutation message.
             * @function verify
             * @memberof exocore.index.TestMutation
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TestMutation.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.success != null && message.hasOwnProperty("success"))
                    if (typeof message.success !== "boolean")
                        return "success: boolean expected";
                return null;
            };

            /**
             * Creates a TestMutation message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.TestMutation
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.TestMutation} TestMutation
             */
            TestMutation.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.TestMutation)
                    return object;
                let message = new $root.exocore.index.TestMutation();
                if (object.success != null)
                    message.success = Boolean(object.success);
                return message;
            };

            /**
             * Creates a plain object from a TestMutation message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.TestMutation
             * @static
             * @param {exocore.index.TestMutation} message TestMutation
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TestMutation.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults)
                    object.success = false;
                if (message.success != null && message.hasOwnProperty("success"))
                    object.success = message.success;
                return object;
            };

            /**
             * Converts this TestMutation to JSON.
             * @function toJSON
             * @memberof exocore.index.TestMutation
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TestMutation.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return TestMutation;
        })();

        index.EntityQuery = (function() {

            /**
             * Properties of an EntityQuery.
             * @memberof exocore.index
             * @interface IEntityQuery
             * @property {exocore.index.IMatchPredicate|null} [match] EntityQuery match
             * @property {exocore.index.ITraitPredicate|null} [trait] EntityQuery trait
             * @property {exocore.index.IIdsPredicate|null} [ids] EntityQuery ids
             * @property {exocore.index.IReferencePredicate|null} [reference] EntityQuery reference
             * @property {exocore.index.IOperationsPredicate|null} [operations] EntityQuery operations
             * @property {exocore.index.IAllPredicate|null} [all] EntityQuery all
             * @property {exocore.index.ITestPredicate|null} [test] EntityQuery test
             * @property {exocore.index.IPaging|null} [paging] Query paging requested
             * @property {exocore.index.IOrdering|null} [ordering] Query ordering
             * @property {boolean|null} [summary] If true, only return summary
             * @property {number|Long|null} [watchToken] Optional watch token if this query is to be used for watching.
             * @property {number|Long|null} [resultHash] If specified, if results from server matches this hash, only a summary will be returned.
             * @property {boolean|null} [includeDeleted] also include deletions.
             */

            /**
             * Constructs a new EntityQuery.
             * @memberof exocore.index
             * @classdesc Represents an EntityQuery.
             * @implements IEntityQuery
             * @constructor
             * @param {exocore.index.IEntityQuery=} [properties] Properties to set
             */
            function EntityQuery(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * EntityQuery match.
             * @member {exocore.index.IMatchPredicate|null|undefined} match
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.match = null;

            /**
             * EntityQuery trait.
             * @member {exocore.index.ITraitPredicate|null|undefined} trait
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.trait = null;

            /**
             * EntityQuery ids.
             * @member {exocore.index.IIdsPredicate|null|undefined} ids
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.ids = null;

            /**
             * EntityQuery reference.
             * @member {exocore.index.IReferencePredicate|null|undefined} reference
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.reference = null;

            /**
             * EntityQuery operations.
             * @member {exocore.index.IOperationsPredicate|null|undefined} operations
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.operations = null;

            /**
             * EntityQuery all.
             * @member {exocore.index.IAllPredicate|null|undefined} all
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.all = null;

            /**
             * EntityQuery test.
             * @member {exocore.index.ITestPredicate|null|undefined} test
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.test = null;

            /**
             * Query paging requested
             * @member {exocore.index.IPaging|null|undefined} paging
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.paging = null;

            /**
             * Query ordering
             * @member {exocore.index.IOrdering|null|undefined} ordering
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.ordering = null;

            /**
             * If true, only return summary
             * @member {boolean} summary
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.summary = false;

            /**
             * Optional watch token if this query is to be used for watching.
             * @member {number|Long} watchToken
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.watchToken = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * If specified, if results from server matches this hash, only a summary will be returned.
             * @member {number|Long} resultHash
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.resultHash = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * also include deletions.
             * @member {boolean} includeDeleted
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            EntityQuery.prototype.includeDeleted = false;

            // OneOf field names bound to virtual getters and setters
            let $oneOfFields;

            /**
             * EntityQuery predicate.
             * @member {"match"|"trait"|"ids"|"reference"|"operations"|"all"|"test"|undefined} predicate
             * @memberof exocore.index.EntityQuery
             * @instance
             */
            Object.defineProperty(EntityQuery.prototype, "predicate", {
                get: $util.oneOfGetter($oneOfFields = ["match", "trait", "ids", "reference", "operations", "all", "test"]),
                set: $util.oneOfSetter($oneOfFields)
            });

            /**
             * Creates a new EntityQuery instance using the specified properties.
             * @function create
             * @memberof exocore.index.EntityQuery
             * @static
             * @param {exocore.index.IEntityQuery=} [properties] Properties to set
             * @returns {exocore.index.EntityQuery} EntityQuery instance
             */
            EntityQuery.create = function create(properties) {
                return new EntityQuery(properties);
            };

            /**
             * Encodes the specified EntityQuery message. Does not implicitly {@link exocore.index.EntityQuery.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.EntityQuery
             * @static
             * @param {exocore.index.IEntityQuery} message EntityQuery message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EntityQuery.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.match != null && Object.hasOwnProperty.call(message, "match"))
                    $root.exocore.index.MatchPredicate.encode(message.match, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.trait != null && Object.hasOwnProperty.call(message, "trait"))
                    $root.exocore.index.TraitPredicate.encode(message.trait, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.ids != null && Object.hasOwnProperty.call(message, "ids"))
                    $root.exocore.index.IdsPredicate.encode(message.ids, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.reference != null && Object.hasOwnProperty.call(message, "reference"))
                    $root.exocore.index.ReferencePredicate.encode(message.reference, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.paging != null && Object.hasOwnProperty.call(message, "paging"))
                    $root.exocore.index.Paging.encode(message.paging, writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                if (message.ordering != null && Object.hasOwnProperty.call(message, "ordering"))
                    $root.exocore.index.Ordering.encode(message.ordering, writer.uint32(/* id 6, wireType 2 =*/50).fork()).ldelim();
                if (message.summary != null && Object.hasOwnProperty.call(message, "summary"))
                    writer.uint32(/* id 7, wireType 0 =*/56).bool(message.summary);
                if (message.watchToken != null && Object.hasOwnProperty.call(message, "watchToken"))
                    writer.uint32(/* id 8, wireType 0 =*/64).uint64(message.watchToken);
                if (message.resultHash != null && Object.hasOwnProperty.call(message, "resultHash"))
                    writer.uint32(/* id 9, wireType 0 =*/72).uint64(message.resultHash);
                if (message.operations != null && Object.hasOwnProperty.call(message, "operations"))
                    $root.exocore.index.OperationsPredicate.encode(message.operations, writer.uint32(/* id 10, wireType 2 =*/82).fork()).ldelim();
                if (message.all != null && Object.hasOwnProperty.call(message, "all"))
                    $root.exocore.index.AllPredicate.encode(message.all, writer.uint32(/* id 11, wireType 2 =*/90).fork()).ldelim();
                if (message.includeDeleted != null && Object.hasOwnProperty.call(message, "includeDeleted"))
                    writer.uint32(/* id 12, wireType 0 =*/96).bool(message.includeDeleted);
                if (message.test != null && Object.hasOwnProperty.call(message, "test"))
                    $root.exocore.index.TestPredicate.encode(message.test, writer.uint32(/* id 99, wireType 2 =*/794).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified EntityQuery message, length delimited. Does not implicitly {@link exocore.index.EntityQuery.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.EntityQuery
             * @static
             * @param {exocore.index.IEntityQuery} message EntityQuery message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EntityQuery.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an EntityQuery message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.EntityQuery
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.EntityQuery} EntityQuery
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EntityQuery.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.EntityQuery();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.match = $root.exocore.index.MatchPredicate.decode(reader, reader.uint32());
                        break;
                    case 2:
                        message.trait = $root.exocore.index.TraitPredicate.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.ids = $root.exocore.index.IdsPredicate.decode(reader, reader.uint32());
                        break;
                    case 4:
                        message.reference = $root.exocore.index.ReferencePredicate.decode(reader, reader.uint32());
                        break;
                    case 10:
                        message.operations = $root.exocore.index.OperationsPredicate.decode(reader, reader.uint32());
                        break;
                    case 11:
                        message.all = $root.exocore.index.AllPredicate.decode(reader, reader.uint32());
                        break;
                    case 99:
                        message.test = $root.exocore.index.TestPredicate.decode(reader, reader.uint32());
                        break;
                    case 5:
                        message.paging = $root.exocore.index.Paging.decode(reader, reader.uint32());
                        break;
                    case 6:
                        message.ordering = $root.exocore.index.Ordering.decode(reader, reader.uint32());
                        break;
                    case 7:
                        message.summary = reader.bool();
                        break;
                    case 8:
                        message.watchToken = reader.uint64();
                        break;
                    case 9:
                        message.resultHash = reader.uint64();
                        break;
                    case 12:
                        message.includeDeleted = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an EntityQuery message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.EntityQuery
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.EntityQuery} EntityQuery
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EntityQuery.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an EntityQuery message.
             * @function verify
             * @memberof exocore.index.EntityQuery
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            EntityQuery.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                let properties = {};
                if (message.match != null && message.hasOwnProperty("match")) {
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.MatchPredicate.verify(message.match);
                        if (error)
                            return "match." + error;
                    }
                }
                if (message.trait != null && message.hasOwnProperty("trait")) {
                    if (properties.predicate === 1)
                        return "predicate: multiple values";
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.TraitPredicate.verify(message.trait);
                        if (error)
                            return "trait." + error;
                    }
                }
                if (message.ids != null && message.hasOwnProperty("ids")) {
                    if (properties.predicate === 1)
                        return "predicate: multiple values";
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.IdsPredicate.verify(message.ids);
                        if (error)
                            return "ids." + error;
                    }
                }
                if (message.reference != null && message.hasOwnProperty("reference")) {
                    if (properties.predicate === 1)
                        return "predicate: multiple values";
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.ReferencePredicate.verify(message.reference);
                        if (error)
                            return "reference." + error;
                    }
                }
                if (message.operations != null && message.hasOwnProperty("operations")) {
                    if (properties.predicate === 1)
                        return "predicate: multiple values";
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.OperationsPredicate.verify(message.operations);
                        if (error)
                            return "operations." + error;
                    }
                }
                if (message.all != null && message.hasOwnProperty("all")) {
                    if (properties.predicate === 1)
                        return "predicate: multiple values";
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.AllPredicate.verify(message.all);
                        if (error)
                            return "all." + error;
                    }
                }
                if (message.test != null && message.hasOwnProperty("test")) {
                    if (properties.predicate === 1)
                        return "predicate: multiple values";
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.TestPredicate.verify(message.test);
                        if (error)
                            return "test." + error;
                    }
                }
                if (message.paging != null && message.hasOwnProperty("paging")) {
                    let error = $root.exocore.index.Paging.verify(message.paging);
                    if (error)
                        return "paging." + error;
                }
                if (message.ordering != null && message.hasOwnProperty("ordering")) {
                    let error = $root.exocore.index.Ordering.verify(message.ordering);
                    if (error)
                        return "ordering." + error;
                }
                if (message.summary != null && message.hasOwnProperty("summary"))
                    if (typeof message.summary !== "boolean")
                        return "summary: boolean expected";
                if (message.watchToken != null && message.hasOwnProperty("watchToken"))
                    if (!$util.isInteger(message.watchToken) && !(message.watchToken && $util.isInteger(message.watchToken.low) && $util.isInteger(message.watchToken.high)))
                        return "watchToken: integer|Long expected";
                if (message.resultHash != null && message.hasOwnProperty("resultHash"))
                    if (!$util.isInteger(message.resultHash) && !(message.resultHash && $util.isInteger(message.resultHash.low) && $util.isInteger(message.resultHash.high)))
                        return "resultHash: integer|Long expected";
                if (message.includeDeleted != null && message.hasOwnProperty("includeDeleted"))
                    if (typeof message.includeDeleted !== "boolean")
                        return "includeDeleted: boolean expected";
                return null;
            };

            /**
             * Creates an EntityQuery message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.EntityQuery
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.EntityQuery} EntityQuery
             */
            EntityQuery.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.EntityQuery)
                    return object;
                let message = new $root.exocore.index.EntityQuery();
                if (object.match != null) {
                    if (typeof object.match !== "object")
                        throw TypeError(".exocore.index.EntityQuery.match: object expected");
                    message.match = $root.exocore.index.MatchPredicate.fromObject(object.match);
                }
                if (object.trait != null) {
                    if (typeof object.trait !== "object")
                        throw TypeError(".exocore.index.EntityQuery.trait: object expected");
                    message.trait = $root.exocore.index.TraitPredicate.fromObject(object.trait);
                }
                if (object.ids != null) {
                    if (typeof object.ids !== "object")
                        throw TypeError(".exocore.index.EntityQuery.ids: object expected");
                    message.ids = $root.exocore.index.IdsPredicate.fromObject(object.ids);
                }
                if (object.reference != null) {
                    if (typeof object.reference !== "object")
                        throw TypeError(".exocore.index.EntityQuery.reference: object expected");
                    message.reference = $root.exocore.index.ReferencePredicate.fromObject(object.reference);
                }
                if (object.operations != null) {
                    if (typeof object.operations !== "object")
                        throw TypeError(".exocore.index.EntityQuery.operations: object expected");
                    message.operations = $root.exocore.index.OperationsPredicate.fromObject(object.operations);
                }
                if (object.all != null) {
                    if (typeof object.all !== "object")
                        throw TypeError(".exocore.index.EntityQuery.all: object expected");
                    message.all = $root.exocore.index.AllPredicate.fromObject(object.all);
                }
                if (object.test != null) {
                    if (typeof object.test !== "object")
                        throw TypeError(".exocore.index.EntityQuery.test: object expected");
                    message.test = $root.exocore.index.TestPredicate.fromObject(object.test);
                }
                if (object.paging != null) {
                    if (typeof object.paging !== "object")
                        throw TypeError(".exocore.index.EntityQuery.paging: object expected");
                    message.paging = $root.exocore.index.Paging.fromObject(object.paging);
                }
                if (object.ordering != null) {
                    if (typeof object.ordering !== "object")
                        throw TypeError(".exocore.index.EntityQuery.ordering: object expected");
                    message.ordering = $root.exocore.index.Ordering.fromObject(object.ordering);
                }
                if (object.summary != null)
                    message.summary = Boolean(object.summary);
                if (object.watchToken != null)
                    if ($util.Long)
                        (message.watchToken = $util.Long.fromValue(object.watchToken)).unsigned = true;
                    else if (typeof object.watchToken === "string")
                        message.watchToken = parseInt(object.watchToken, 10);
                    else if (typeof object.watchToken === "number")
                        message.watchToken = object.watchToken;
                    else if (typeof object.watchToken === "object")
                        message.watchToken = new $util.LongBits(object.watchToken.low >>> 0, object.watchToken.high >>> 0).toNumber(true);
                if (object.resultHash != null)
                    if ($util.Long)
                        (message.resultHash = $util.Long.fromValue(object.resultHash)).unsigned = true;
                    else if (typeof object.resultHash === "string")
                        message.resultHash = parseInt(object.resultHash, 10);
                    else if (typeof object.resultHash === "number")
                        message.resultHash = object.resultHash;
                    else if (typeof object.resultHash === "object")
                        message.resultHash = new $util.LongBits(object.resultHash.low >>> 0, object.resultHash.high >>> 0).toNumber(true);
                if (object.includeDeleted != null)
                    message.includeDeleted = Boolean(object.includeDeleted);
                return message;
            };

            /**
             * Creates a plain object from an EntityQuery message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.EntityQuery
             * @static
             * @param {exocore.index.EntityQuery} message EntityQuery
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            EntityQuery.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.paging = null;
                    object.ordering = null;
                    object.summary = false;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.watchToken = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.watchToken = options.longs === String ? "0" : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.resultHash = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.resultHash = options.longs === String ? "0" : 0;
                    object.includeDeleted = false;
                }
                if (message.match != null && message.hasOwnProperty("match")) {
                    object.match = $root.exocore.index.MatchPredicate.toObject(message.match, options);
                    if (options.oneofs)
                        object.predicate = "match";
                }
                if (message.trait != null && message.hasOwnProperty("trait")) {
                    object.trait = $root.exocore.index.TraitPredicate.toObject(message.trait, options);
                    if (options.oneofs)
                        object.predicate = "trait";
                }
                if (message.ids != null && message.hasOwnProperty("ids")) {
                    object.ids = $root.exocore.index.IdsPredicate.toObject(message.ids, options);
                    if (options.oneofs)
                        object.predicate = "ids";
                }
                if (message.reference != null && message.hasOwnProperty("reference")) {
                    object.reference = $root.exocore.index.ReferencePredicate.toObject(message.reference, options);
                    if (options.oneofs)
                        object.predicate = "reference";
                }
                if (message.paging != null && message.hasOwnProperty("paging"))
                    object.paging = $root.exocore.index.Paging.toObject(message.paging, options);
                if (message.ordering != null && message.hasOwnProperty("ordering"))
                    object.ordering = $root.exocore.index.Ordering.toObject(message.ordering, options);
                if (message.summary != null && message.hasOwnProperty("summary"))
                    object.summary = message.summary;
                if (message.watchToken != null && message.hasOwnProperty("watchToken"))
                    if (typeof message.watchToken === "number")
                        object.watchToken = options.longs === String ? String(message.watchToken) : message.watchToken;
                    else
                        object.watchToken = options.longs === String ? $util.Long.prototype.toString.call(message.watchToken) : options.longs === Number ? new $util.LongBits(message.watchToken.low >>> 0, message.watchToken.high >>> 0).toNumber(true) : message.watchToken;
                if (message.resultHash != null && message.hasOwnProperty("resultHash"))
                    if (typeof message.resultHash === "number")
                        object.resultHash = options.longs === String ? String(message.resultHash) : message.resultHash;
                    else
                        object.resultHash = options.longs === String ? $util.Long.prototype.toString.call(message.resultHash) : options.longs === Number ? new $util.LongBits(message.resultHash.low >>> 0, message.resultHash.high >>> 0).toNumber(true) : message.resultHash;
                if (message.operations != null && message.hasOwnProperty("operations")) {
                    object.operations = $root.exocore.index.OperationsPredicate.toObject(message.operations, options);
                    if (options.oneofs)
                        object.predicate = "operations";
                }
                if (message.all != null && message.hasOwnProperty("all")) {
                    object.all = $root.exocore.index.AllPredicate.toObject(message.all, options);
                    if (options.oneofs)
                        object.predicate = "all";
                }
                if (message.includeDeleted != null && message.hasOwnProperty("includeDeleted"))
                    object.includeDeleted = message.includeDeleted;
                if (message.test != null && message.hasOwnProperty("test")) {
                    object.test = $root.exocore.index.TestPredicate.toObject(message.test, options);
                    if (options.oneofs)
                        object.predicate = "test";
                }
                return object;
            };

            /**
             * Converts this EntityQuery to JSON.
             * @function toJSON
             * @memberof exocore.index.EntityQuery
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            EntityQuery.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return EntityQuery;
        })();

        index.MatchPredicate = (function() {

            /**
             * Properties of a MatchPredicate.
             * @memberof exocore.index
             * @interface IMatchPredicate
             * @property {string|null} [query] MatchPredicate query
             */

            /**
             * Constructs a new MatchPredicate.
             * @memberof exocore.index
             * @classdesc Query entities by text match on all indexed fields across all traits.
             * @implements IMatchPredicate
             * @constructor
             * @param {exocore.index.IMatchPredicate=} [properties] Properties to set
             */
            function MatchPredicate(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * MatchPredicate query.
             * @member {string} query
             * @memberof exocore.index.MatchPredicate
             * @instance
             */
            MatchPredicate.prototype.query = "";

            /**
             * Creates a new MatchPredicate instance using the specified properties.
             * @function create
             * @memberof exocore.index.MatchPredicate
             * @static
             * @param {exocore.index.IMatchPredicate=} [properties] Properties to set
             * @returns {exocore.index.MatchPredicate} MatchPredicate instance
             */
            MatchPredicate.create = function create(properties) {
                return new MatchPredicate(properties);
            };

            /**
             * Encodes the specified MatchPredicate message. Does not implicitly {@link exocore.index.MatchPredicate.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.MatchPredicate
             * @static
             * @param {exocore.index.IMatchPredicate} message MatchPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MatchPredicate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.query != null && Object.hasOwnProperty.call(message, "query"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.query);
                return writer;
            };

            /**
             * Encodes the specified MatchPredicate message, length delimited. Does not implicitly {@link exocore.index.MatchPredicate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.MatchPredicate
             * @static
             * @param {exocore.index.IMatchPredicate} message MatchPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MatchPredicate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a MatchPredicate message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.MatchPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.MatchPredicate} MatchPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MatchPredicate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.MatchPredicate();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.query = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a MatchPredicate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.MatchPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.MatchPredicate} MatchPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MatchPredicate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a MatchPredicate message.
             * @function verify
             * @memberof exocore.index.MatchPredicate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            MatchPredicate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.query != null && message.hasOwnProperty("query"))
                    if (!$util.isString(message.query))
                        return "query: string expected";
                return null;
            };

            /**
             * Creates a MatchPredicate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.MatchPredicate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.MatchPredicate} MatchPredicate
             */
            MatchPredicate.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.MatchPredicate)
                    return object;
                let message = new $root.exocore.index.MatchPredicate();
                if (object.query != null)
                    message.query = String(object.query);
                return message;
            };

            /**
             * Creates a plain object from a MatchPredicate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.MatchPredicate
             * @static
             * @param {exocore.index.MatchPredicate} message MatchPredicate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            MatchPredicate.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults)
                    object.query = "";
                if (message.query != null && message.hasOwnProperty("query"))
                    object.query = message.query;
                return object;
            };

            /**
             * Converts this MatchPredicate to JSON.
             * @function toJSON
             * @memberof exocore.index.MatchPredicate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            MatchPredicate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return MatchPredicate;
        })();

        index.IdsPredicate = (function() {

            /**
             * Properties of an IdsPredicate.
             * @memberof exocore.index
             * @interface IIdsPredicate
             * @property {Array.<string>|null} [ids] IdsPredicate ids
             */

            /**
             * Constructs a new IdsPredicate.
             * @memberof exocore.index
             * @classdesc Query entities by IDs.
             * @implements IIdsPredicate
             * @constructor
             * @param {exocore.index.IIdsPredicate=} [properties] Properties to set
             */
            function IdsPredicate(properties) {
                this.ids = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * IdsPredicate ids.
             * @member {Array.<string>} ids
             * @memberof exocore.index.IdsPredicate
             * @instance
             */
            IdsPredicate.prototype.ids = $util.emptyArray;

            /**
             * Creates a new IdsPredicate instance using the specified properties.
             * @function create
             * @memberof exocore.index.IdsPredicate
             * @static
             * @param {exocore.index.IIdsPredicate=} [properties] Properties to set
             * @returns {exocore.index.IdsPredicate} IdsPredicate instance
             */
            IdsPredicate.create = function create(properties) {
                return new IdsPredicate(properties);
            };

            /**
             * Encodes the specified IdsPredicate message. Does not implicitly {@link exocore.index.IdsPredicate.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.IdsPredicate
             * @static
             * @param {exocore.index.IIdsPredicate} message IdsPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            IdsPredicate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.ids != null && message.ids.length)
                    for (let i = 0; i < message.ids.length; ++i)
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.ids[i]);
                return writer;
            };

            /**
             * Encodes the specified IdsPredicate message, length delimited. Does not implicitly {@link exocore.index.IdsPredicate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.IdsPredicate
             * @static
             * @param {exocore.index.IIdsPredicate} message IdsPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            IdsPredicate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an IdsPredicate message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.IdsPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.IdsPredicate} IdsPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            IdsPredicate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.IdsPredicate();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.ids && message.ids.length))
                            message.ids = [];
                        message.ids.push(reader.string());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an IdsPredicate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.IdsPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.IdsPredicate} IdsPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            IdsPredicate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an IdsPredicate message.
             * @function verify
             * @memberof exocore.index.IdsPredicate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            IdsPredicate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.ids != null && message.hasOwnProperty("ids")) {
                    if (!Array.isArray(message.ids))
                        return "ids: array expected";
                    for (let i = 0; i < message.ids.length; ++i)
                        if (!$util.isString(message.ids[i]))
                            return "ids: string[] expected";
                }
                return null;
            };

            /**
             * Creates an IdsPredicate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.IdsPredicate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.IdsPredicate} IdsPredicate
             */
            IdsPredicate.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.IdsPredicate)
                    return object;
                let message = new $root.exocore.index.IdsPredicate();
                if (object.ids) {
                    if (!Array.isArray(object.ids))
                        throw TypeError(".exocore.index.IdsPredicate.ids: array expected");
                    message.ids = [];
                    for (let i = 0; i < object.ids.length; ++i)
                        message.ids[i] = String(object.ids[i]);
                }
                return message;
            };

            /**
             * Creates a plain object from an IdsPredicate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.IdsPredicate
             * @static
             * @param {exocore.index.IdsPredicate} message IdsPredicate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            IdsPredicate.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.ids = [];
                if (message.ids && message.ids.length) {
                    object.ids = [];
                    for (let j = 0; j < message.ids.length; ++j)
                        object.ids[j] = message.ids[j];
                }
                return object;
            };

            /**
             * Converts this IdsPredicate to JSON.
             * @function toJSON
             * @memberof exocore.index.IdsPredicate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            IdsPredicate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return IdsPredicate;
        })();

        index.OperationsPredicate = (function() {

            /**
             * Properties of an OperationsPredicate.
             * @memberof exocore.index
             * @interface IOperationsPredicate
             * @property {Array.<number|Long>|null} [operationIds] OperationsPredicate operationIds
             */

            /**
             * Constructs a new OperationsPredicate.
             * @memberof exocore.index
             * @classdesc Used to return entities on which mutations with these operation ids were applied and indexed.
             * @implements IOperationsPredicate
             * @constructor
             * @param {exocore.index.IOperationsPredicate=} [properties] Properties to set
             */
            function OperationsPredicate(properties) {
                this.operationIds = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * OperationsPredicate operationIds.
             * @member {Array.<number|Long>} operationIds
             * @memberof exocore.index.OperationsPredicate
             * @instance
             */
            OperationsPredicate.prototype.operationIds = $util.emptyArray;

            /**
             * Creates a new OperationsPredicate instance using the specified properties.
             * @function create
             * @memberof exocore.index.OperationsPredicate
             * @static
             * @param {exocore.index.IOperationsPredicate=} [properties] Properties to set
             * @returns {exocore.index.OperationsPredicate} OperationsPredicate instance
             */
            OperationsPredicate.create = function create(properties) {
                return new OperationsPredicate(properties);
            };

            /**
             * Encodes the specified OperationsPredicate message. Does not implicitly {@link exocore.index.OperationsPredicate.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.OperationsPredicate
             * @static
             * @param {exocore.index.IOperationsPredicate} message OperationsPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OperationsPredicate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.operationIds != null && message.operationIds.length) {
                    writer.uint32(/* id 1, wireType 2 =*/10).fork();
                    for (let i = 0; i < message.operationIds.length; ++i)
                        writer.uint64(message.operationIds[i]);
                    writer.ldelim();
                }
                return writer;
            };

            /**
             * Encodes the specified OperationsPredicate message, length delimited. Does not implicitly {@link exocore.index.OperationsPredicate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.OperationsPredicate
             * @static
             * @param {exocore.index.IOperationsPredicate} message OperationsPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OperationsPredicate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an OperationsPredicate message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.OperationsPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.OperationsPredicate} OperationsPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OperationsPredicate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.OperationsPredicate();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.operationIds && message.operationIds.length))
                            message.operationIds = [];
                        if ((tag & 7) === 2) {
                            let end2 = reader.uint32() + reader.pos;
                            while (reader.pos < end2)
                                message.operationIds.push(reader.uint64());
                        } else
                            message.operationIds.push(reader.uint64());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an OperationsPredicate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.OperationsPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.OperationsPredicate} OperationsPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OperationsPredicate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an OperationsPredicate message.
             * @function verify
             * @memberof exocore.index.OperationsPredicate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OperationsPredicate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.operationIds != null && message.hasOwnProperty("operationIds")) {
                    if (!Array.isArray(message.operationIds))
                        return "operationIds: array expected";
                    for (let i = 0; i < message.operationIds.length; ++i)
                        if (!$util.isInteger(message.operationIds[i]) && !(message.operationIds[i] && $util.isInteger(message.operationIds[i].low) && $util.isInteger(message.operationIds[i].high)))
                            return "operationIds: integer|Long[] expected";
                }
                return null;
            };

            /**
             * Creates an OperationsPredicate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.OperationsPredicate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.OperationsPredicate} OperationsPredicate
             */
            OperationsPredicate.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.OperationsPredicate)
                    return object;
                let message = new $root.exocore.index.OperationsPredicate();
                if (object.operationIds) {
                    if (!Array.isArray(object.operationIds))
                        throw TypeError(".exocore.index.OperationsPredicate.operationIds: array expected");
                    message.operationIds = [];
                    for (let i = 0; i < object.operationIds.length; ++i)
                        if ($util.Long)
                            (message.operationIds[i] = $util.Long.fromValue(object.operationIds[i])).unsigned = true;
                        else if (typeof object.operationIds[i] === "string")
                            message.operationIds[i] = parseInt(object.operationIds[i], 10);
                        else if (typeof object.operationIds[i] === "number")
                            message.operationIds[i] = object.operationIds[i];
                        else if (typeof object.operationIds[i] === "object")
                            message.operationIds[i] = new $util.LongBits(object.operationIds[i].low >>> 0, object.operationIds[i].high >>> 0).toNumber(true);
                }
                return message;
            };

            /**
             * Creates a plain object from an OperationsPredicate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.OperationsPredicate
             * @static
             * @param {exocore.index.OperationsPredicate} message OperationsPredicate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OperationsPredicate.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.operationIds = [];
                if (message.operationIds && message.operationIds.length) {
                    object.operationIds = [];
                    for (let j = 0; j < message.operationIds.length; ++j)
                        if (typeof message.operationIds[j] === "number")
                            object.operationIds[j] = options.longs === String ? String(message.operationIds[j]) : message.operationIds[j];
                        else
                            object.operationIds[j] = options.longs === String ? $util.Long.prototype.toString.call(message.operationIds[j]) : options.longs === Number ? new $util.LongBits(message.operationIds[j].low >>> 0, message.operationIds[j].high >>> 0).toNumber(true) : message.operationIds[j];
                }
                return object;
            };

            /**
             * Converts this OperationsPredicate to JSON.
             * @function toJSON
             * @memberof exocore.index.OperationsPredicate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OperationsPredicate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return OperationsPredicate;
        })();

        index.AllPredicate = (function() {

            /**
             * Properties of an AllPredicate.
             * @memberof exocore.index
             * @interface IAllPredicate
             */

            /**
             * Constructs a new AllPredicate.
             * @memberof exocore.index
             * @classdesc Query all entities.
             * @implements IAllPredicate
             * @constructor
             * @param {exocore.index.IAllPredicate=} [properties] Properties to set
             */
            function AllPredicate(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Creates a new AllPredicate instance using the specified properties.
             * @function create
             * @memberof exocore.index.AllPredicate
             * @static
             * @param {exocore.index.IAllPredicate=} [properties] Properties to set
             * @returns {exocore.index.AllPredicate} AllPredicate instance
             */
            AllPredicate.create = function create(properties) {
                return new AllPredicate(properties);
            };

            /**
             * Encodes the specified AllPredicate message. Does not implicitly {@link exocore.index.AllPredicate.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.AllPredicate
             * @static
             * @param {exocore.index.IAllPredicate} message AllPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AllPredicate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };

            /**
             * Encodes the specified AllPredicate message, length delimited. Does not implicitly {@link exocore.index.AllPredicate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.AllPredicate
             * @static
             * @param {exocore.index.IAllPredicate} message AllPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            AllPredicate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an AllPredicate message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.AllPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.AllPredicate} AllPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AllPredicate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.AllPredicate();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an AllPredicate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.AllPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.AllPredicate} AllPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            AllPredicate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an AllPredicate message.
             * @function verify
             * @memberof exocore.index.AllPredicate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            AllPredicate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };

            /**
             * Creates an AllPredicate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.AllPredicate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.AllPredicate} AllPredicate
             */
            AllPredicate.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.AllPredicate)
                    return object;
                return new $root.exocore.index.AllPredicate();
            };

            /**
             * Creates a plain object from an AllPredicate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.AllPredicate
             * @static
             * @param {exocore.index.AllPredicate} message AllPredicate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            AllPredicate.toObject = function toObject() {
                return {};
            };

            /**
             * Converts this AllPredicate to JSON.
             * @function toJSON
             * @memberof exocore.index.AllPredicate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            AllPredicate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return AllPredicate;
        })();

        index.TestPredicate = (function() {

            /**
             * Properties of a TestPredicate.
             * @memberof exocore.index
             * @interface ITestPredicate
             * @property {boolean|null} [success] TestPredicate success
             */

            /**
             * Constructs a new TestPredicate.
             * @memberof exocore.index
             * @classdesc Used for tests.
             * @implements ITestPredicate
             * @constructor
             * @param {exocore.index.ITestPredicate=} [properties] Properties to set
             */
            function TestPredicate(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TestPredicate success.
             * @member {boolean} success
             * @memberof exocore.index.TestPredicate
             * @instance
             */
            TestPredicate.prototype.success = false;

            /**
             * Creates a new TestPredicate instance using the specified properties.
             * @function create
             * @memberof exocore.index.TestPredicate
             * @static
             * @param {exocore.index.ITestPredicate=} [properties] Properties to set
             * @returns {exocore.index.TestPredicate} TestPredicate instance
             */
            TestPredicate.create = function create(properties) {
                return new TestPredicate(properties);
            };

            /**
             * Encodes the specified TestPredicate message. Does not implicitly {@link exocore.index.TestPredicate.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.TestPredicate
             * @static
             * @param {exocore.index.ITestPredicate} message TestPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestPredicate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.success != null && Object.hasOwnProperty.call(message, "success"))
                    writer.uint32(/* id 1, wireType 0 =*/8).bool(message.success);
                return writer;
            };

            /**
             * Encodes the specified TestPredicate message, length delimited. Does not implicitly {@link exocore.index.TestPredicate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.TestPredicate
             * @static
             * @param {exocore.index.ITestPredicate} message TestPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestPredicate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TestPredicate message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.TestPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.TestPredicate} TestPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestPredicate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.TestPredicate();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.success = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a TestPredicate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.TestPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.TestPredicate} TestPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestPredicate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TestPredicate message.
             * @function verify
             * @memberof exocore.index.TestPredicate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TestPredicate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.success != null && message.hasOwnProperty("success"))
                    if (typeof message.success !== "boolean")
                        return "success: boolean expected";
                return null;
            };

            /**
             * Creates a TestPredicate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.TestPredicate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.TestPredicate} TestPredicate
             */
            TestPredicate.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.TestPredicate)
                    return object;
                let message = new $root.exocore.index.TestPredicate();
                if (object.success != null)
                    message.success = Boolean(object.success);
                return message;
            };

            /**
             * Creates a plain object from a TestPredicate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.TestPredicate
             * @static
             * @param {exocore.index.TestPredicate} message TestPredicate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TestPredicate.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults)
                    object.success = false;
                if (message.success != null && message.hasOwnProperty("success"))
                    object.success = message.success;
                return object;
            };

            /**
             * Converts this TestPredicate to JSON.
             * @function toJSON
             * @memberof exocore.index.TestPredicate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TestPredicate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return TestPredicate;
        })();

        index.TraitPredicate = (function() {

            /**
             * Properties of a TraitPredicate.
             * @memberof exocore.index
             * @interface ITraitPredicate
             * @property {string|null} [traitName] TraitPredicate traitName
             * @property {exocore.index.ITraitQuery|null} [query] TraitPredicate query
             */

            /**
             * Constructs a new TraitPredicate.
             * @memberof exocore.index
             * @classdesc Query entities that have a specified trait and optionally matching a trait query.
             * @implements ITraitPredicate
             * @constructor
             * @param {exocore.index.ITraitPredicate=} [properties] Properties to set
             */
            function TraitPredicate(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TraitPredicate traitName.
             * @member {string} traitName
             * @memberof exocore.index.TraitPredicate
             * @instance
             */
            TraitPredicate.prototype.traitName = "";

            /**
             * TraitPredicate query.
             * @member {exocore.index.ITraitQuery|null|undefined} query
             * @memberof exocore.index.TraitPredicate
             * @instance
             */
            TraitPredicate.prototype.query = null;

            /**
             * Creates a new TraitPredicate instance using the specified properties.
             * @function create
             * @memberof exocore.index.TraitPredicate
             * @static
             * @param {exocore.index.ITraitPredicate=} [properties] Properties to set
             * @returns {exocore.index.TraitPredicate} TraitPredicate instance
             */
            TraitPredicate.create = function create(properties) {
                return new TraitPredicate(properties);
            };

            /**
             * Encodes the specified TraitPredicate message. Does not implicitly {@link exocore.index.TraitPredicate.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.TraitPredicate
             * @static
             * @param {exocore.index.ITraitPredicate} message TraitPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TraitPredicate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.traitName != null && Object.hasOwnProperty.call(message, "traitName"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.traitName);
                if (message.query != null && Object.hasOwnProperty.call(message, "query"))
                    $root.exocore.index.TraitQuery.encode(message.query, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified TraitPredicate message, length delimited. Does not implicitly {@link exocore.index.TraitPredicate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.TraitPredicate
             * @static
             * @param {exocore.index.ITraitPredicate} message TraitPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TraitPredicate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TraitPredicate message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.TraitPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.TraitPredicate} TraitPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TraitPredicate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.TraitPredicate();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.traitName = reader.string();
                        break;
                    case 2:
                        message.query = $root.exocore.index.TraitQuery.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a TraitPredicate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.TraitPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.TraitPredicate} TraitPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TraitPredicate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TraitPredicate message.
             * @function verify
             * @memberof exocore.index.TraitPredicate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TraitPredicate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.traitName != null && message.hasOwnProperty("traitName"))
                    if (!$util.isString(message.traitName))
                        return "traitName: string expected";
                if (message.query != null && message.hasOwnProperty("query")) {
                    let error = $root.exocore.index.TraitQuery.verify(message.query);
                    if (error)
                        return "query." + error;
                }
                return null;
            };

            /**
             * Creates a TraitPredicate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.TraitPredicate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.TraitPredicate} TraitPredicate
             */
            TraitPredicate.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.TraitPredicate)
                    return object;
                let message = new $root.exocore.index.TraitPredicate();
                if (object.traitName != null)
                    message.traitName = String(object.traitName);
                if (object.query != null) {
                    if (typeof object.query !== "object")
                        throw TypeError(".exocore.index.TraitPredicate.query: object expected");
                    message.query = $root.exocore.index.TraitQuery.fromObject(object.query);
                }
                return message;
            };

            /**
             * Creates a plain object from a TraitPredicate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.TraitPredicate
             * @static
             * @param {exocore.index.TraitPredicate} message TraitPredicate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TraitPredicate.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.traitName = "";
                    object.query = null;
                }
                if (message.traitName != null && message.hasOwnProperty("traitName"))
                    object.traitName = message.traitName;
                if (message.query != null && message.hasOwnProperty("query"))
                    object.query = $root.exocore.index.TraitQuery.toObject(message.query, options);
                return object;
            };

            /**
             * Converts this TraitPredicate to JSON.
             * @function toJSON
             * @memberof exocore.index.TraitPredicate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TraitPredicate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return TraitPredicate;
        })();

        index.TraitQuery = (function() {

            /**
             * Properties of a TraitQuery.
             * @memberof exocore.index
             * @interface ITraitQuery
             * @property {exocore.index.IMatchPredicate|null} [match] TraitQuery match
             * @property {exocore.index.ITraitFieldPredicate|null} [field] TraitQuery field
             * @property {exocore.index.ITraitFieldReferencePredicate|null} [reference] TraitQuery reference
             */

            /**
             * Constructs a new TraitQuery.
             * @memberof exocore.index
             * @classdesc Represents a TraitQuery.
             * @implements ITraitQuery
             * @constructor
             * @param {exocore.index.ITraitQuery=} [properties] Properties to set
             */
            function TraitQuery(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TraitQuery match.
             * @member {exocore.index.IMatchPredicate|null|undefined} match
             * @memberof exocore.index.TraitQuery
             * @instance
             */
            TraitQuery.prototype.match = null;

            /**
             * TraitQuery field.
             * @member {exocore.index.ITraitFieldPredicate|null|undefined} field
             * @memberof exocore.index.TraitQuery
             * @instance
             */
            TraitQuery.prototype.field = null;

            /**
             * TraitQuery reference.
             * @member {exocore.index.ITraitFieldReferencePredicate|null|undefined} reference
             * @memberof exocore.index.TraitQuery
             * @instance
             */
            TraitQuery.prototype.reference = null;

            // OneOf field names bound to virtual getters and setters
            let $oneOfFields;

            /**
             * TraitQuery predicate.
             * @member {"match"|"field"|"reference"|undefined} predicate
             * @memberof exocore.index.TraitQuery
             * @instance
             */
            Object.defineProperty(TraitQuery.prototype, "predicate", {
                get: $util.oneOfGetter($oneOfFields = ["match", "field", "reference"]),
                set: $util.oneOfSetter($oneOfFields)
            });

            /**
             * Creates a new TraitQuery instance using the specified properties.
             * @function create
             * @memberof exocore.index.TraitQuery
             * @static
             * @param {exocore.index.ITraitQuery=} [properties] Properties to set
             * @returns {exocore.index.TraitQuery} TraitQuery instance
             */
            TraitQuery.create = function create(properties) {
                return new TraitQuery(properties);
            };

            /**
             * Encodes the specified TraitQuery message. Does not implicitly {@link exocore.index.TraitQuery.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.TraitQuery
             * @static
             * @param {exocore.index.ITraitQuery} message TraitQuery message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TraitQuery.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.match != null && Object.hasOwnProperty.call(message, "match"))
                    $root.exocore.index.MatchPredicate.encode(message.match, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.field != null && Object.hasOwnProperty.call(message, "field"))
                    $root.exocore.index.TraitFieldPredicate.encode(message.field, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.reference != null && Object.hasOwnProperty.call(message, "reference"))
                    $root.exocore.index.TraitFieldReferencePredicate.encode(message.reference, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified TraitQuery message, length delimited. Does not implicitly {@link exocore.index.TraitQuery.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.TraitQuery
             * @static
             * @param {exocore.index.ITraitQuery} message TraitQuery message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TraitQuery.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TraitQuery message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.TraitQuery
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.TraitQuery} TraitQuery
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TraitQuery.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.TraitQuery();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.match = $root.exocore.index.MatchPredicate.decode(reader, reader.uint32());
                        break;
                    case 2:
                        message.field = $root.exocore.index.TraitFieldPredicate.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.reference = $root.exocore.index.TraitFieldReferencePredicate.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a TraitQuery message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.TraitQuery
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.TraitQuery} TraitQuery
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TraitQuery.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TraitQuery message.
             * @function verify
             * @memberof exocore.index.TraitQuery
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TraitQuery.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                let properties = {};
                if (message.match != null && message.hasOwnProperty("match")) {
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.MatchPredicate.verify(message.match);
                        if (error)
                            return "match." + error;
                    }
                }
                if (message.field != null && message.hasOwnProperty("field")) {
                    if (properties.predicate === 1)
                        return "predicate: multiple values";
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.TraitFieldPredicate.verify(message.field);
                        if (error)
                            return "field." + error;
                    }
                }
                if (message.reference != null && message.hasOwnProperty("reference")) {
                    if (properties.predicate === 1)
                        return "predicate: multiple values";
                    properties.predicate = 1;
                    {
                        let error = $root.exocore.index.TraitFieldReferencePredicate.verify(message.reference);
                        if (error)
                            return "reference." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a TraitQuery message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.TraitQuery
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.TraitQuery} TraitQuery
             */
            TraitQuery.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.TraitQuery)
                    return object;
                let message = new $root.exocore.index.TraitQuery();
                if (object.match != null) {
                    if (typeof object.match !== "object")
                        throw TypeError(".exocore.index.TraitQuery.match: object expected");
                    message.match = $root.exocore.index.MatchPredicate.fromObject(object.match);
                }
                if (object.field != null) {
                    if (typeof object.field !== "object")
                        throw TypeError(".exocore.index.TraitQuery.field: object expected");
                    message.field = $root.exocore.index.TraitFieldPredicate.fromObject(object.field);
                }
                if (object.reference != null) {
                    if (typeof object.reference !== "object")
                        throw TypeError(".exocore.index.TraitQuery.reference: object expected");
                    message.reference = $root.exocore.index.TraitFieldReferencePredicate.fromObject(object.reference);
                }
                return message;
            };

            /**
             * Creates a plain object from a TraitQuery message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.TraitQuery
             * @static
             * @param {exocore.index.TraitQuery} message TraitQuery
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TraitQuery.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (message.match != null && message.hasOwnProperty("match")) {
                    object.match = $root.exocore.index.MatchPredicate.toObject(message.match, options);
                    if (options.oneofs)
                        object.predicate = "match";
                }
                if (message.field != null && message.hasOwnProperty("field")) {
                    object.field = $root.exocore.index.TraitFieldPredicate.toObject(message.field, options);
                    if (options.oneofs)
                        object.predicate = "field";
                }
                if (message.reference != null && message.hasOwnProperty("reference")) {
                    object.reference = $root.exocore.index.TraitFieldReferencePredicate.toObject(message.reference, options);
                    if (options.oneofs)
                        object.predicate = "reference";
                }
                return object;
            };

            /**
             * Converts this TraitQuery to JSON.
             * @function toJSON
             * @memberof exocore.index.TraitQuery
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TraitQuery.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return TraitQuery;
        })();

        index.TraitFieldPredicate = (function() {

            /**
             * Properties of a TraitFieldPredicate.
             * @memberof exocore.index
             * @interface ITraitFieldPredicate
             * @property {string|null} [field] TraitFieldPredicate field
             * @property {string|null} [string] TraitFieldPredicate string
             * @property {number|Long|null} [int64] TraitFieldPredicate int64
             * @property {number|Long|null} [uint64] TraitFieldPredicate uint64
             * @property {google.protobuf.ITimestamp|null} [date] TraitFieldPredicate date
             * @property {exocore.index.TraitFieldPredicate.Operator|null} [operator] TraitFieldPredicate operator
             */

            /**
             * Constructs a new TraitFieldPredicate.
             * @memberof exocore.index
             * @classdesc Represents a TraitFieldPredicate.
             * @implements ITraitFieldPredicate
             * @constructor
             * @param {exocore.index.ITraitFieldPredicate=} [properties] Properties to set
             */
            function TraitFieldPredicate(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TraitFieldPredicate field.
             * @member {string} field
             * @memberof exocore.index.TraitFieldPredicate
             * @instance
             */
            TraitFieldPredicate.prototype.field = "";

            /**
             * TraitFieldPredicate string.
             * @member {string} string
             * @memberof exocore.index.TraitFieldPredicate
             * @instance
             */
            TraitFieldPredicate.prototype.string = "";

            /**
             * TraitFieldPredicate int64.
             * @member {number|Long} int64
             * @memberof exocore.index.TraitFieldPredicate
             * @instance
             */
            TraitFieldPredicate.prototype.int64 = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

            /**
             * TraitFieldPredicate uint64.
             * @member {number|Long} uint64
             * @memberof exocore.index.TraitFieldPredicate
             * @instance
             */
            TraitFieldPredicate.prototype.uint64 = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * TraitFieldPredicate date.
             * @member {google.protobuf.ITimestamp|null|undefined} date
             * @memberof exocore.index.TraitFieldPredicate
             * @instance
             */
            TraitFieldPredicate.prototype.date = null;

            /**
             * TraitFieldPredicate operator.
             * @member {exocore.index.TraitFieldPredicate.Operator} operator
             * @memberof exocore.index.TraitFieldPredicate
             * @instance
             */
            TraitFieldPredicate.prototype.operator = 0;

            // OneOf field names bound to virtual getters and setters
            let $oneOfFields;

            /**
             * TraitFieldPredicate value.
             * @member {"string"|"int64"|"uint64"|"date"|undefined} value
             * @memberof exocore.index.TraitFieldPredicate
             * @instance
             */
            Object.defineProperty(TraitFieldPredicate.prototype, "value", {
                get: $util.oneOfGetter($oneOfFields = ["string", "int64", "uint64", "date"]),
                set: $util.oneOfSetter($oneOfFields)
            });

            /**
             * Creates a new TraitFieldPredicate instance using the specified properties.
             * @function create
             * @memberof exocore.index.TraitFieldPredicate
             * @static
             * @param {exocore.index.ITraitFieldPredicate=} [properties] Properties to set
             * @returns {exocore.index.TraitFieldPredicate} TraitFieldPredicate instance
             */
            TraitFieldPredicate.create = function create(properties) {
                return new TraitFieldPredicate(properties);
            };

            /**
             * Encodes the specified TraitFieldPredicate message. Does not implicitly {@link exocore.index.TraitFieldPredicate.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.TraitFieldPredicate
             * @static
             * @param {exocore.index.ITraitFieldPredicate} message TraitFieldPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TraitFieldPredicate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.field != null && Object.hasOwnProperty.call(message, "field"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.field);
                if (message.string != null && Object.hasOwnProperty.call(message, "string"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.string);
                if (message.int64 != null && Object.hasOwnProperty.call(message, "int64"))
                    writer.uint32(/* id 3, wireType 0 =*/24).int64(message.int64);
                if (message.uint64 != null && Object.hasOwnProperty.call(message, "uint64"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.uint64);
                if (message.date != null && Object.hasOwnProperty.call(message, "date"))
                    $root.google.protobuf.Timestamp.encode(message.date, writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                if (message.operator != null && Object.hasOwnProperty.call(message, "operator"))
                    writer.uint32(/* id 6, wireType 0 =*/48).int32(message.operator);
                return writer;
            };

            /**
             * Encodes the specified TraitFieldPredicate message, length delimited. Does not implicitly {@link exocore.index.TraitFieldPredicate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.TraitFieldPredicate
             * @static
             * @param {exocore.index.ITraitFieldPredicate} message TraitFieldPredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TraitFieldPredicate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TraitFieldPredicate message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.TraitFieldPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.TraitFieldPredicate} TraitFieldPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TraitFieldPredicate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.TraitFieldPredicate();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.field = reader.string();
                        break;
                    case 2:
                        message.string = reader.string();
                        break;
                    case 3:
                        message.int64 = reader.int64();
                        break;
                    case 4:
                        message.uint64 = reader.uint64();
                        break;
                    case 5:
                        message.date = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                        break;
                    case 6:
                        message.operator = reader.int32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a TraitFieldPredicate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.TraitFieldPredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.TraitFieldPredicate} TraitFieldPredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TraitFieldPredicate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TraitFieldPredicate message.
             * @function verify
             * @memberof exocore.index.TraitFieldPredicate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TraitFieldPredicate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                let properties = {};
                if (message.field != null && message.hasOwnProperty("field"))
                    if (!$util.isString(message.field))
                        return "field: string expected";
                if (message.string != null && message.hasOwnProperty("string")) {
                    properties.value = 1;
                    if (!$util.isString(message.string))
                        return "string: string expected";
                }
                if (message.int64 != null && message.hasOwnProperty("int64")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    if (!$util.isInteger(message.int64) && !(message.int64 && $util.isInteger(message.int64.low) && $util.isInteger(message.int64.high)))
                        return "int64: integer|Long expected";
                }
                if (message.uint64 != null && message.hasOwnProperty("uint64")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    if (!$util.isInteger(message.uint64) && !(message.uint64 && $util.isInteger(message.uint64.low) && $util.isInteger(message.uint64.high)))
                        return "uint64: integer|Long expected";
                }
                if (message.date != null && message.hasOwnProperty("date")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    {
                        let error = $root.google.protobuf.Timestamp.verify(message.date);
                        if (error)
                            return "date." + error;
                    }
                }
                if (message.operator != null && message.hasOwnProperty("operator"))
                    switch (message.operator) {
                    default:
                        return "operator: enum value expected";
                    case 0:
                    case 1:
                    case 2:
                    case 3:
                    case 4:
                        break;
                    }
                return null;
            };

            /**
             * Creates a TraitFieldPredicate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.TraitFieldPredicate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.TraitFieldPredicate} TraitFieldPredicate
             */
            TraitFieldPredicate.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.TraitFieldPredicate)
                    return object;
                let message = new $root.exocore.index.TraitFieldPredicate();
                if (object.field != null)
                    message.field = String(object.field);
                if (object.string != null)
                    message.string = String(object.string);
                if (object.int64 != null)
                    if ($util.Long)
                        (message.int64 = $util.Long.fromValue(object.int64)).unsigned = false;
                    else if (typeof object.int64 === "string")
                        message.int64 = parseInt(object.int64, 10);
                    else if (typeof object.int64 === "number")
                        message.int64 = object.int64;
                    else if (typeof object.int64 === "object")
                        message.int64 = new $util.LongBits(object.int64.low >>> 0, object.int64.high >>> 0).toNumber();
                if (object.uint64 != null)
                    if ($util.Long)
                        (message.uint64 = $util.Long.fromValue(object.uint64)).unsigned = true;
                    else if (typeof object.uint64 === "string")
                        message.uint64 = parseInt(object.uint64, 10);
                    else if (typeof object.uint64 === "number")
                        message.uint64 = object.uint64;
                    else if (typeof object.uint64 === "object")
                        message.uint64 = new $util.LongBits(object.uint64.low >>> 0, object.uint64.high >>> 0).toNumber(true);
                if (object.date != null) {
                    if (typeof object.date !== "object")
                        throw TypeError(".exocore.index.TraitFieldPredicate.date: object expected");
                    message.date = $root.google.protobuf.Timestamp.fromObject(object.date);
                }
                switch (object.operator) {
                case "EQUAL":
                case 0:
                    message.operator = 0;
                    break;
                case "GT":
                case 1:
                    message.operator = 1;
                    break;
                case "GTE":
                case 2:
                    message.operator = 2;
                    break;
                case "LT":
                case 3:
                    message.operator = 3;
                    break;
                case "LTE":
                case 4:
                    message.operator = 4;
                    break;
                }
                return message;
            };

            /**
             * Creates a plain object from a TraitFieldPredicate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.TraitFieldPredicate
             * @static
             * @param {exocore.index.TraitFieldPredicate} message TraitFieldPredicate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TraitFieldPredicate.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.field = "";
                    object.operator = options.enums === String ? "EQUAL" : 0;
                }
                if (message.field != null && message.hasOwnProperty("field"))
                    object.field = message.field;
                if (message.string != null && message.hasOwnProperty("string")) {
                    object.string = message.string;
                    if (options.oneofs)
                        object.value = "string";
                }
                if (message.int64 != null && message.hasOwnProperty("int64")) {
                    if (typeof message.int64 === "number")
                        object.int64 = options.longs === String ? String(message.int64) : message.int64;
                    else
                        object.int64 = options.longs === String ? $util.Long.prototype.toString.call(message.int64) : options.longs === Number ? new $util.LongBits(message.int64.low >>> 0, message.int64.high >>> 0).toNumber() : message.int64;
                    if (options.oneofs)
                        object.value = "int64";
                }
                if (message.uint64 != null && message.hasOwnProperty("uint64")) {
                    if (typeof message.uint64 === "number")
                        object.uint64 = options.longs === String ? String(message.uint64) : message.uint64;
                    else
                        object.uint64 = options.longs === String ? $util.Long.prototype.toString.call(message.uint64) : options.longs === Number ? new $util.LongBits(message.uint64.low >>> 0, message.uint64.high >>> 0).toNumber(true) : message.uint64;
                    if (options.oneofs)
                        object.value = "uint64";
                }
                if (message.date != null && message.hasOwnProperty("date")) {
                    object.date = $root.google.protobuf.Timestamp.toObject(message.date, options);
                    if (options.oneofs)
                        object.value = "date";
                }
                if (message.operator != null && message.hasOwnProperty("operator"))
                    object.operator = options.enums === String ? $root.exocore.index.TraitFieldPredicate.Operator[message.operator] : message.operator;
                return object;
            };

            /**
             * Converts this TraitFieldPredicate to JSON.
             * @function toJSON
             * @memberof exocore.index.TraitFieldPredicate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TraitFieldPredicate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Operator enum.
             * @name exocore.index.TraitFieldPredicate.Operator
             * @enum {number}
             * @property {number} EQUAL=0 EQUAL value
             * @property {number} GT=1 GT value
             * @property {number} GTE=2 GTE value
             * @property {number} LT=3 LT value
             * @property {number} LTE=4 LTE value
             */
            TraitFieldPredicate.Operator = (function() {
                const valuesById = {}, values = Object.create(valuesById);
                values[valuesById[0] = "EQUAL"] = 0;
                values[valuesById[1] = "GT"] = 1;
                values[valuesById[2] = "GTE"] = 2;
                values[valuesById[3] = "LT"] = 3;
                values[valuesById[4] = "LTE"] = 4;
                return values;
            })();

            return TraitFieldPredicate;
        })();

        index.TraitFieldReferencePredicate = (function() {

            /**
             * Properties of a TraitFieldReferencePredicate.
             * @memberof exocore.index
             * @interface ITraitFieldReferencePredicate
             * @property {string|null} [field] TraitFieldReferencePredicate field
             * @property {exocore.index.IReferencePredicate|null} [reference] TraitFieldReferencePredicate reference
             */

            /**
             * Constructs a new TraitFieldReferencePredicate.
             * @memberof exocore.index
             * @classdesc Represents a TraitFieldReferencePredicate.
             * @implements ITraitFieldReferencePredicate
             * @constructor
             * @param {exocore.index.ITraitFieldReferencePredicate=} [properties] Properties to set
             */
            function TraitFieldReferencePredicate(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TraitFieldReferencePredicate field.
             * @member {string} field
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @instance
             */
            TraitFieldReferencePredicate.prototype.field = "";

            /**
             * TraitFieldReferencePredicate reference.
             * @member {exocore.index.IReferencePredicate|null|undefined} reference
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @instance
             */
            TraitFieldReferencePredicate.prototype.reference = null;

            /**
             * Creates a new TraitFieldReferencePredicate instance using the specified properties.
             * @function create
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @static
             * @param {exocore.index.ITraitFieldReferencePredicate=} [properties] Properties to set
             * @returns {exocore.index.TraitFieldReferencePredicate} TraitFieldReferencePredicate instance
             */
            TraitFieldReferencePredicate.create = function create(properties) {
                return new TraitFieldReferencePredicate(properties);
            };

            /**
             * Encodes the specified TraitFieldReferencePredicate message. Does not implicitly {@link exocore.index.TraitFieldReferencePredicate.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @static
             * @param {exocore.index.ITraitFieldReferencePredicate} message TraitFieldReferencePredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TraitFieldReferencePredicate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.field != null && Object.hasOwnProperty.call(message, "field"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.field);
                if (message.reference != null && Object.hasOwnProperty.call(message, "reference"))
                    $root.exocore.index.ReferencePredicate.encode(message.reference, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified TraitFieldReferencePredicate message, length delimited. Does not implicitly {@link exocore.index.TraitFieldReferencePredicate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @static
             * @param {exocore.index.ITraitFieldReferencePredicate} message TraitFieldReferencePredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TraitFieldReferencePredicate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TraitFieldReferencePredicate message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.TraitFieldReferencePredicate} TraitFieldReferencePredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TraitFieldReferencePredicate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.TraitFieldReferencePredicate();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.field = reader.string();
                        break;
                    case 2:
                        message.reference = $root.exocore.index.ReferencePredicate.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a TraitFieldReferencePredicate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.TraitFieldReferencePredicate} TraitFieldReferencePredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TraitFieldReferencePredicate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TraitFieldReferencePredicate message.
             * @function verify
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TraitFieldReferencePredicate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.field != null && message.hasOwnProperty("field"))
                    if (!$util.isString(message.field))
                        return "field: string expected";
                if (message.reference != null && message.hasOwnProperty("reference")) {
                    let error = $root.exocore.index.ReferencePredicate.verify(message.reference);
                    if (error)
                        return "reference." + error;
                }
                return null;
            };

            /**
             * Creates a TraitFieldReferencePredicate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.TraitFieldReferencePredicate} TraitFieldReferencePredicate
             */
            TraitFieldReferencePredicate.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.TraitFieldReferencePredicate)
                    return object;
                let message = new $root.exocore.index.TraitFieldReferencePredicate();
                if (object.field != null)
                    message.field = String(object.field);
                if (object.reference != null) {
                    if (typeof object.reference !== "object")
                        throw TypeError(".exocore.index.TraitFieldReferencePredicate.reference: object expected");
                    message.reference = $root.exocore.index.ReferencePredicate.fromObject(object.reference);
                }
                return message;
            };

            /**
             * Creates a plain object from a TraitFieldReferencePredicate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @static
             * @param {exocore.index.TraitFieldReferencePredicate} message TraitFieldReferencePredicate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TraitFieldReferencePredicate.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.field = "";
                    object.reference = null;
                }
                if (message.field != null && message.hasOwnProperty("field"))
                    object.field = message.field;
                if (message.reference != null && message.hasOwnProperty("reference"))
                    object.reference = $root.exocore.index.ReferencePredicate.toObject(message.reference, options);
                return object;
            };

            /**
             * Converts this TraitFieldReferencePredicate to JSON.
             * @function toJSON
             * @memberof exocore.index.TraitFieldReferencePredicate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TraitFieldReferencePredicate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return TraitFieldReferencePredicate;
        })();

        index.ReferencePredicate = (function() {

            /**
             * Properties of a ReferencePredicate.
             * @memberof exocore.index
             * @interface IReferencePredicate
             * @property {string|null} [entityId] ReferencePredicate entityId
             * @property {string|null} [traitId] ReferencePredicate traitId
             */

            /**
             * Constructs a new ReferencePredicate.
             * @memberof exocore.index
             * @classdesc Represents a ReferencePredicate.
             * @implements IReferencePredicate
             * @constructor
             * @param {exocore.index.IReferencePredicate=} [properties] Properties to set
             */
            function ReferencePredicate(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ReferencePredicate entityId.
             * @member {string} entityId
             * @memberof exocore.index.ReferencePredicate
             * @instance
             */
            ReferencePredicate.prototype.entityId = "";

            /**
             * ReferencePredicate traitId.
             * @member {string} traitId
             * @memberof exocore.index.ReferencePredicate
             * @instance
             */
            ReferencePredicate.prototype.traitId = "";

            /**
             * Creates a new ReferencePredicate instance using the specified properties.
             * @function create
             * @memberof exocore.index.ReferencePredicate
             * @static
             * @param {exocore.index.IReferencePredicate=} [properties] Properties to set
             * @returns {exocore.index.ReferencePredicate} ReferencePredicate instance
             */
            ReferencePredicate.create = function create(properties) {
                return new ReferencePredicate(properties);
            };

            /**
             * Encodes the specified ReferencePredicate message. Does not implicitly {@link exocore.index.ReferencePredicate.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.ReferencePredicate
             * @static
             * @param {exocore.index.IReferencePredicate} message ReferencePredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ReferencePredicate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.entityId != null && Object.hasOwnProperty.call(message, "entityId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.entityId);
                if (message.traitId != null && Object.hasOwnProperty.call(message, "traitId"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.traitId);
                return writer;
            };

            /**
             * Encodes the specified ReferencePredicate message, length delimited. Does not implicitly {@link exocore.index.ReferencePredicate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.ReferencePredicate
             * @static
             * @param {exocore.index.IReferencePredicate} message ReferencePredicate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ReferencePredicate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a ReferencePredicate message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.ReferencePredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.ReferencePredicate} ReferencePredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ReferencePredicate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.ReferencePredicate();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.entityId = reader.string();
                        break;
                    case 2:
                        message.traitId = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a ReferencePredicate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.ReferencePredicate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.ReferencePredicate} ReferencePredicate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ReferencePredicate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a ReferencePredicate message.
             * @function verify
             * @memberof exocore.index.ReferencePredicate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ReferencePredicate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.entityId != null && message.hasOwnProperty("entityId"))
                    if (!$util.isString(message.entityId))
                        return "entityId: string expected";
                if (message.traitId != null && message.hasOwnProperty("traitId"))
                    if (!$util.isString(message.traitId))
                        return "traitId: string expected";
                return null;
            };

            /**
             * Creates a ReferencePredicate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.ReferencePredicate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.ReferencePredicate} ReferencePredicate
             */
            ReferencePredicate.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.ReferencePredicate)
                    return object;
                let message = new $root.exocore.index.ReferencePredicate();
                if (object.entityId != null)
                    message.entityId = String(object.entityId);
                if (object.traitId != null)
                    message.traitId = String(object.traitId);
                return message;
            };

            /**
             * Creates a plain object from a ReferencePredicate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.ReferencePredicate
             * @static
             * @param {exocore.index.ReferencePredicate} message ReferencePredicate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ReferencePredicate.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.entityId = "";
                    object.traitId = "";
                }
                if (message.entityId != null && message.hasOwnProperty("entityId"))
                    object.entityId = message.entityId;
                if (message.traitId != null && message.hasOwnProperty("traitId"))
                    object.traitId = message.traitId;
                return object;
            };

            /**
             * Converts this ReferencePredicate to JSON.
             * @function toJSON
             * @memberof exocore.index.ReferencePredicate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ReferencePredicate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return ReferencePredicate;
        })();

        index.Paging = (function() {

            /**
             * Properties of a Paging.
             * @memberof exocore.index
             * @interface IPaging
             * @property {exocore.index.IOrderingValue|null} [afterOrderingValue] Returns results after this given ordering value.
             * @property {exocore.index.IOrderingValue|null} [beforeOrderingValue] Returns results before this given ordering value.
             * @property {number|null} [count] Desired results count. Default if 0.
             */

            /**
             * Constructs a new Paging.
             * @memberof exocore.index
             * @classdesc Represents a Paging.
             * @implements IPaging
             * @constructor
             * @param {exocore.index.IPaging=} [properties] Properties to set
             */
            function Paging(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Returns results after this given ordering value.
             * @member {exocore.index.IOrderingValue|null|undefined} afterOrderingValue
             * @memberof exocore.index.Paging
             * @instance
             */
            Paging.prototype.afterOrderingValue = null;

            /**
             * Returns results before this given ordering value.
             * @member {exocore.index.IOrderingValue|null|undefined} beforeOrderingValue
             * @memberof exocore.index.Paging
             * @instance
             */
            Paging.prototype.beforeOrderingValue = null;

            /**
             * Desired results count. Default if 0.
             * @member {number} count
             * @memberof exocore.index.Paging
             * @instance
             */
            Paging.prototype.count = 0;

            /**
             * Creates a new Paging instance using the specified properties.
             * @function create
             * @memberof exocore.index.Paging
             * @static
             * @param {exocore.index.IPaging=} [properties] Properties to set
             * @returns {exocore.index.Paging} Paging instance
             */
            Paging.create = function create(properties) {
                return new Paging(properties);
            };

            /**
             * Encodes the specified Paging message. Does not implicitly {@link exocore.index.Paging.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.Paging
             * @static
             * @param {exocore.index.IPaging} message Paging message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Paging.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.afterOrderingValue != null && Object.hasOwnProperty.call(message, "afterOrderingValue"))
                    $root.exocore.index.OrderingValue.encode(message.afterOrderingValue, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.beforeOrderingValue != null && Object.hasOwnProperty.call(message, "beforeOrderingValue"))
                    $root.exocore.index.OrderingValue.encode(message.beforeOrderingValue, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.count != null && Object.hasOwnProperty.call(message, "count"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint32(message.count);
                return writer;
            };

            /**
             * Encodes the specified Paging message, length delimited. Does not implicitly {@link exocore.index.Paging.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.Paging
             * @static
             * @param {exocore.index.IPaging} message Paging message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Paging.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a Paging message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.Paging
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.Paging} Paging
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Paging.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.Paging();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.afterOrderingValue = $root.exocore.index.OrderingValue.decode(reader, reader.uint32());
                        break;
                    case 2:
                        message.beforeOrderingValue = $root.exocore.index.OrderingValue.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.count = reader.uint32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a Paging message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.Paging
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.Paging} Paging
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Paging.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a Paging message.
             * @function verify
             * @memberof exocore.index.Paging
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Paging.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.afterOrderingValue != null && message.hasOwnProperty("afterOrderingValue")) {
                    let error = $root.exocore.index.OrderingValue.verify(message.afterOrderingValue);
                    if (error)
                        return "afterOrderingValue." + error;
                }
                if (message.beforeOrderingValue != null && message.hasOwnProperty("beforeOrderingValue")) {
                    let error = $root.exocore.index.OrderingValue.verify(message.beforeOrderingValue);
                    if (error)
                        return "beforeOrderingValue." + error;
                }
                if (message.count != null && message.hasOwnProperty("count"))
                    if (!$util.isInteger(message.count))
                        return "count: integer expected";
                return null;
            };

            /**
             * Creates a Paging message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.Paging
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.Paging} Paging
             */
            Paging.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.Paging)
                    return object;
                let message = new $root.exocore.index.Paging();
                if (object.afterOrderingValue != null) {
                    if (typeof object.afterOrderingValue !== "object")
                        throw TypeError(".exocore.index.Paging.afterOrderingValue: object expected");
                    message.afterOrderingValue = $root.exocore.index.OrderingValue.fromObject(object.afterOrderingValue);
                }
                if (object.beforeOrderingValue != null) {
                    if (typeof object.beforeOrderingValue !== "object")
                        throw TypeError(".exocore.index.Paging.beforeOrderingValue: object expected");
                    message.beforeOrderingValue = $root.exocore.index.OrderingValue.fromObject(object.beforeOrderingValue);
                }
                if (object.count != null)
                    message.count = object.count >>> 0;
                return message;
            };

            /**
             * Creates a plain object from a Paging message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.Paging
             * @static
             * @param {exocore.index.Paging} message Paging
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Paging.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.afterOrderingValue = null;
                    object.beforeOrderingValue = null;
                    object.count = 0;
                }
                if (message.afterOrderingValue != null && message.hasOwnProperty("afterOrderingValue"))
                    object.afterOrderingValue = $root.exocore.index.OrderingValue.toObject(message.afterOrderingValue, options);
                if (message.beforeOrderingValue != null && message.hasOwnProperty("beforeOrderingValue"))
                    object.beforeOrderingValue = $root.exocore.index.OrderingValue.toObject(message.beforeOrderingValue, options);
                if (message.count != null && message.hasOwnProperty("count"))
                    object.count = message.count;
                return object;
            };

            /**
             * Converts this Paging to JSON.
             * @function toJSON
             * @memberof exocore.index.Paging
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Paging.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return Paging;
        })();

        index.Ordering = (function() {

            /**
             * Properties of an Ordering.
             * @memberof exocore.index
             * @interface IOrdering
             * @property {boolean|null} [score] Ordering score
             * @property {boolean|null} [operationId] Ordering operationId
             * @property {string|null} [field] Ordering field
             * @property {boolean|null} [ascending] Direction of ordering.
             */

            /**
             * Constructs a new Ordering.
             * @memberof exocore.index
             * @classdesc Represents an Ordering.
             * @implements IOrdering
             * @constructor
             * @param {exocore.index.IOrdering=} [properties] Properties to set
             */
            function Ordering(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Ordering score.
             * @member {boolean} score
             * @memberof exocore.index.Ordering
             * @instance
             */
            Ordering.prototype.score = false;

            /**
             * Ordering operationId.
             * @member {boolean} operationId
             * @memberof exocore.index.Ordering
             * @instance
             */
            Ordering.prototype.operationId = false;

            /**
             * Ordering field.
             * @member {string} field
             * @memberof exocore.index.Ordering
             * @instance
             */
            Ordering.prototype.field = "";

            /**
             * Direction of ordering.
             * @member {boolean} ascending
             * @memberof exocore.index.Ordering
             * @instance
             */
            Ordering.prototype.ascending = false;

            // OneOf field names bound to virtual getters and setters
            let $oneOfFields;

            /**
             * Value by which we want results to be ordered.
             * @member {"score"|"operationId"|"field"|undefined} value
             * @memberof exocore.index.Ordering
             * @instance
             */
            Object.defineProperty(Ordering.prototype, "value", {
                get: $util.oneOfGetter($oneOfFields = ["score", "operationId", "field"]),
                set: $util.oneOfSetter($oneOfFields)
            });

            /**
             * Creates a new Ordering instance using the specified properties.
             * @function create
             * @memberof exocore.index.Ordering
             * @static
             * @param {exocore.index.IOrdering=} [properties] Properties to set
             * @returns {exocore.index.Ordering} Ordering instance
             */
            Ordering.create = function create(properties) {
                return new Ordering(properties);
            };

            /**
             * Encodes the specified Ordering message. Does not implicitly {@link exocore.index.Ordering.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.Ordering
             * @static
             * @param {exocore.index.IOrdering} message Ordering message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Ordering.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.score != null && Object.hasOwnProperty.call(message, "score"))
                    writer.uint32(/* id 1, wireType 0 =*/8).bool(message.score);
                if (message.operationId != null && Object.hasOwnProperty.call(message, "operationId"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.operationId);
                if (message.field != null && Object.hasOwnProperty.call(message, "field"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.field);
                if (message.ascending != null && Object.hasOwnProperty.call(message, "ascending"))
                    writer.uint32(/* id 4, wireType 0 =*/32).bool(message.ascending);
                return writer;
            };

            /**
             * Encodes the specified Ordering message, length delimited. Does not implicitly {@link exocore.index.Ordering.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.Ordering
             * @static
             * @param {exocore.index.IOrdering} message Ordering message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Ordering.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an Ordering message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.Ordering
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.Ordering} Ordering
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Ordering.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.Ordering();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.score = reader.bool();
                        break;
                    case 2:
                        message.operationId = reader.bool();
                        break;
                    case 3:
                        message.field = reader.string();
                        break;
                    case 4:
                        message.ascending = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an Ordering message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.Ordering
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.Ordering} Ordering
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Ordering.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an Ordering message.
             * @function verify
             * @memberof exocore.index.Ordering
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Ordering.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                let properties = {};
                if (message.score != null && message.hasOwnProperty("score")) {
                    properties.value = 1;
                    if (typeof message.score !== "boolean")
                        return "score: boolean expected";
                }
                if (message.operationId != null && message.hasOwnProperty("operationId")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    if (typeof message.operationId !== "boolean")
                        return "operationId: boolean expected";
                }
                if (message.field != null && message.hasOwnProperty("field")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    if (!$util.isString(message.field))
                        return "field: string expected";
                }
                if (message.ascending != null && message.hasOwnProperty("ascending"))
                    if (typeof message.ascending !== "boolean")
                        return "ascending: boolean expected";
                return null;
            };

            /**
             * Creates an Ordering message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.Ordering
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.Ordering} Ordering
             */
            Ordering.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.Ordering)
                    return object;
                let message = new $root.exocore.index.Ordering();
                if (object.score != null)
                    message.score = Boolean(object.score);
                if (object.operationId != null)
                    message.operationId = Boolean(object.operationId);
                if (object.field != null)
                    message.field = String(object.field);
                if (object.ascending != null)
                    message.ascending = Boolean(object.ascending);
                return message;
            };

            /**
             * Creates a plain object from an Ordering message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.Ordering
             * @static
             * @param {exocore.index.Ordering} message Ordering
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Ordering.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults)
                    object.ascending = false;
                if (message.score != null && message.hasOwnProperty("score")) {
                    object.score = message.score;
                    if (options.oneofs)
                        object.value = "score";
                }
                if (message.operationId != null && message.hasOwnProperty("operationId")) {
                    object.operationId = message.operationId;
                    if (options.oneofs)
                        object.value = "operationId";
                }
                if (message.field != null && message.hasOwnProperty("field")) {
                    object.field = message.field;
                    if (options.oneofs)
                        object.value = "field";
                }
                if (message.ascending != null && message.hasOwnProperty("ascending"))
                    object.ascending = message.ascending;
                return object;
            };

            /**
             * Converts this Ordering to JSON.
             * @function toJSON
             * @memberof exocore.index.Ordering
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Ordering.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return Ordering;
        })();

        index.OrderingValue = (function() {

            /**
             * Properties of an OrderingValue.
             * @memberof exocore.index
             * @interface IOrderingValue
             * @property {number|null} [float] OrderingValue float
             * @property {number|Long|null} [uint64] OrderingValue uint64
             * @property {google.protobuf.ITimestamp|null} [date] OrderingValue date
             * @property {boolean|null} [min] OrderingValue min
             * @property {boolean|null} [max] OrderingValue max
             * @property {number|Long|null} [operationId] ID operation used to tie break equal results
             */

            /**
             * Constructs a new OrderingValue.
             * @memberof exocore.index
             * @classdesc Represents an OrderingValue.
             * @implements IOrderingValue
             * @constructor
             * @param {exocore.index.IOrderingValue=} [properties] Properties to set
             */
            function OrderingValue(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * OrderingValue float.
             * @member {number} float
             * @memberof exocore.index.OrderingValue
             * @instance
             */
            OrderingValue.prototype.float = 0;

            /**
             * OrderingValue uint64.
             * @member {number|Long} uint64
             * @memberof exocore.index.OrderingValue
             * @instance
             */
            OrderingValue.prototype.uint64 = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * OrderingValue date.
             * @member {google.protobuf.ITimestamp|null|undefined} date
             * @memberof exocore.index.OrderingValue
             * @instance
             */
            OrderingValue.prototype.date = null;

            /**
             * OrderingValue min.
             * @member {boolean} min
             * @memberof exocore.index.OrderingValue
             * @instance
             */
            OrderingValue.prototype.min = false;

            /**
             * OrderingValue max.
             * @member {boolean} max
             * @memberof exocore.index.OrderingValue
             * @instance
             */
            OrderingValue.prototype.max = false;

            /**
             * ID operation used to tie break equal results
             * @member {number|Long} operationId
             * @memberof exocore.index.OrderingValue
             * @instance
             */
            OrderingValue.prototype.operationId = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            // OneOf field names bound to virtual getters and setters
            let $oneOfFields;

            /**
             * OrderingValue value.
             * @member {"float"|"uint64"|"date"|"min"|"max"|undefined} value
             * @memberof exocore.index.OrderingValue
             * @instance
             */
            Object.defineProperty(OrderingValue.prototype, "value", {
                get: $util.oneOfGetter($oneOfFields = ["float", "uint64", "date", "min", "max"]),
                set: $util.oneOfSetter($oneOfFields)
            });

            /**
             * Creates a new OrderingValue instance using the specified properties.
             * @function create
             * @memberof exocore.index.OrderingValue
             * @static
             * @param {exocore.index.IOrderingValue=} [properties] Properties to set
             * @returns {exocore.index.OrderingValue} OrderingValue instance
             */
            OrderingValue.create = function create(properties) {
                return new OrderingValue(properties);
            };

            /**
             * Encodes the specified OrderingValue message. Does not implicitly {@link exocore.index.OrderingValue.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.OrderingValue
             * @static
             * @param {exocore.index.IOrderingValue} message OrderingValue message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OrderingValue.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.float != null && Object.hasOwnProperty.call(message, "float"))
                    writer.uint32(/* id 1, wireType 5 =*/13).float(message.float);
                if (message.uint64 != null && Object.hasOwnProperty.call(message, "uint64"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.uint64);
                if (message.date != null && Object.hasOwnProperty.call(message, "date"))
                    $root.google.protobuf.Timestamp.encode(message.date, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.min != null && Object.hasOwnProperty.call(message, "min"))
                    writer.uint32(/* id 4, wireType 0 =*/32).bool(message.min);
                if (message.max != null && Object.hasOwnProperty.call(message, "max"))
                    writer.uint32(/* id 5, wireType 0 =*/40).bool(message.max);
                if (message.operationId != null && Object.hasOwnProperty.call(message, "operationId"))
                    writer.uint32(/* id 6, wireType 0 =*/48).uint64(message.operationId);
                return writer;
            };

            /**
             * Encodes the specified OrderingValue message, length delimited. Does not implicitly {@link exocore.index.OrderingValue.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.OrderingValue
             * @static
             * @param {exocore.index.IOrderingValue} message OrderingValue message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OrderingValue.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an OrderingValue message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.OrderingValue
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.OrderingValue} OrderingValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OrderingValue.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.OrderingValue();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.float = reader.float();
                        break;
                    case 2:
                        message.uint64 = reader.uint64();
                        break;
                    case 3:
                        message.date = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                        break;
                    case 4:
                        message.min = reader.bool();
                        break;
                    case 5:
                        message.max = reader.bool();
                        break;
                    case 6:
                        message.operationId = reader.uint64();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an OrderingValue message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.OrderingValue
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.OrderingValue} OrderingValue
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OrderingValue.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an OrderingValue message.
             * @function verify
             * @memberof exocore.index.OrderingValue
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OrderingValue.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                let properties = {};
                if (message.float != null && message.hasOwnProperty("float")) {
                    properties.value = 1;
                    if (typeof message.float !== "number")
                        return "float: number expected";
                }
                if (message.uint64 != null && message.hasOwnProperty("uint64")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    if (!$util.isInteger(message.uint64) && !(message.uint64 && $util.isInteger(message.uint64.low) && $util.isInteger(message.uint64.high)))
                        return "uint64: integer|Long expected";
                }
                if (message.date != null && message.hasOwnProperty("date")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    {
                        let error = $root.google.protobuf.Timestamp.verify(message.date);
                        if (error)
                            return "date." + error;
                    }
                }
                if (message.min != null && message.hasOwnProperty("min")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    if (typeof message.min !== "boolean")
                        return "min: boolean expected";
                }
                if (message.max != null && message.hasOwnProperty("max")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    if (typeof message.max !== "boolean")
                        return "max: boolean expected";
                }
                if (message.operationId != null && message.hasOwnProperty("operationId"))
                    if (!$util.isInteger(message.operationId) && !(message.operationId && $util.isInteger(message.operationId.low) && $util.isInteger(message.operationId.high)))
                        return "operationId: integer|Long expected";
                return null;
            };

            /**
             * Creates an OrderingValue message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.OrderingValue
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.OrderingValue} OrderingValue
             */
            OrderingValue.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.OrderingValue)
                    return object;
                let message = new $root.exocore.index.OrderingValue();
                if (object.float != null)
                    message.float = Number(object.float);
                if (object.uint64 != null)
                    if ($util.Long)
                        (message.uint64 = $util.Long.fromValue(object.uint64)).unsigned = true;
                    else if (typeof object.uint64 === "string")
                        message.uint64 = parseInt(object.uint64, 10);
                    else if (typeof object.uint64 === "number")
                        message.uint64 = object.uint64;
                    else if (typeof object.uint64 === "object")
                        message.uint64 = new $util.LongBits(object.uint64.low >>> 0, object.uint64.high >>> 0).toNumber(true);
                if (object.date != null) {
                    if (typeof object.date !== "object")
                        throw TypeError(".exocore.index.OrderingValue.date: object expected");
                    message.date = $root.google.protobuf.Timestamp.fromObject(object.date);
                }
                if (object.min != null)
                    message.min = Boolean(object.min);
                if (object.max != null)
                    message.max = Boolean(object.max);
                if (object.operationId != null)
                    if ($util.Long)
                        (message.operationId = $util.Long.fromValue(object.operationId)).unsigned = true;
                    else if (typeof object.operationId === "string")
                        message.operationId = parseInt(object.operationId, 10);
                    else if (typeof object.operationId === "number")
                        message.operationId = object.operationId;
                    else if (typeof object.operationId === "object")
                        message.operationId = new $util.LongBits(object.operationId.low >>> 0, object.operationId.high >>> 0).toNumber(true);
                return message;
            };

            /**
             * Creates a plain object from an OrderingValue message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.OrderingValue
             * @static
             * @param {exocore.index.OrderingValue} message OrderingValue
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OrderingValue.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults)
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.operationId = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.operationId = options.longs === String ? "0" : 0;
                if (message.float != null && message.hasOwnProperty("float")) {
                    object.float = options.json && !isFinite(message.float) ? String(message.float) : message.float;
                    if (options.oneofs)
                        object.value = "float";
                }
                if (message.uint64 != null && message.hasOwnProperty("uint64")) {
                    if (typeof message.uint64 === "number")
                        object.uint64 = options.longs === String ? String(message.uint64) : message.uint64;
                    else
                        object.uint64 = options.longs === String ? $util.Long.prototype.toString.call(message.uint64) : options.longs === Number ? new $util.LongBits(message.uint64.low >>> 0, message.uint64.high >>> 0).toNumber(true) : message.uint64;
                    if (options.oneofs)
                        object.value = "uint64";
                }
                if (message.date != null && message.hasOwnProperty("date")) {
                    object.date = $root.google.protobuf.Timestamp.toObject(message.date, options);
                    if (options.oneofs)
                        object.value = "date";
                }
                if (message.min != null && message.hasOwnProperty("min")) {
                    object.min = message.min;
                    if (options.oneofs)
                        object.value = "min";
                }
                if (message.max != null && message.hasOwnProperty("max")) {
                    object.max = message.max;
                    if (options.oneofs)
                        object.value = "max";
                }
                if (message.operationId != null && message.hasOwnProperty("operationId"))
                    if (typeof message.operationId === "number")
                        object.operationId = options.longs === String ? String(message.operationId) : message.operationId;
                    else
                        object.operationId = options.longs === String ? $util.Long.prototype.toString.call(message.operationId) : options.longs === Number ? new $util.LongBits(message.operationId.low >>> 0, message.operationId.high >>> 0).toNumber(true) : message.operationId;
                return object;
            };

            /**
             * Converts this OrderingValue to JSON.
             * @function toJSON
             * @memberof exocore.index.OrderingValue
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OrderingValue.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return OrderingValue;
        })();

        index.EntityResults = (function() {

            /**
             * Properties of an EntityResults.
             * @memberof exocore.index
             * @interface IEntityResults
             * @property {Array.<exocore.index.IEntityResult>|null} [entities] EntityResults entities
             * @property {boolean|null} [summary] EntityResults summary
             * @property {number|null} [estimatedCount] EntityResults estimatedCount
             * @property {exocore.index.IPaging|null} [currentPage] EntityResults currentPage
             * @property {exocore.index.IPaging|null} [nextPage] EntityResults nextPage
             * @property {number|Long|null} [hash] EntityResults hash
             */

            /**
             * Constructs a new EntityResults.
             * @memberof exocore.index
             * @classdesc Represents an EntityResults.
             * @implements IEntityResults
             * @constructor
             * @param {exocore.index.IEntityResults=} [properties] Properties to set
             */
            function EntityResults(properties) {
                this.entities = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * EntityResults entities.
             * @member {Array.<exocore.index.IEntityResult>} entities
             * @memberof exocore.index.EntityResults
             * @instance
             */
            EntityResults.prototype.entities = $util.emptyArray;

            /**
             * EntityResults summary.
             * @member {boolean} summary
             * @memberof exocore.index.EntityResults
             * @instance
             */
            EntityResults.prototype.summary = false;

            /**
             * EntityResults estimatedCount.
             * @member {number} estimatedCount
             * @memberof exocore.index.EntityResults
             * @instance
             */
            EntityResults.prototype.estimatedCount = 0;

            /**
             * EntityResults currentPage.
             * @member {exocore.index.IPaging|null|undefined} currentPage
             * @memberof exocore.index.EntityResults
             * @instance
             */
            EntityResults.prototype.currentPage = null;

            /**
             * EntityResults nextPage.
             * @member {exocore.index.IPaging|null|undefined} nextPage
             * @memberof exocore.index.EntityResults
             * @instance
             */
            EntityResults.prototype.nextPage = null;

            /**
             * EntityResults hash.
             * @member {number|Long} hash
             * @memberof exocore.index.EntityResults
             * @instance
             */
            EntityResults.prototype.hash = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * Creates a new EntityResults instance using the specified properties.
             * @function create
             * @memberof exocore.index.EntityResults
             * @static
             * @param {exocore.index.IEntityResults=} [properties] Properties to set
             * @returns {exocore.index.EntityResults} EntityResults instance
             */
            EntityResults.create = function create(properties) {
                return new EntityResults(properties);
            };

            /**
             * Encodes the specified EntityResults message. Does not implicitly {@link exocore.index.EntityResults.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.EntityResults
             * @static
             * @param {exocore.index.IEntityResults} message EntityResults message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EntityResults.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.entities != null && message.entities.length)
                    for (let i = 0; i < message.entities.length; ++i)
                        $root.exocore.index.EntityResult.encode(message.entities[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.summary != null && Object.hasOwnProperty.call(message, "summary"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.summary);
                if (message.estimatedCount != null && Object.hasOwnProperty.call(message, "estimatedCount"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint32(message.estimatedCount);
                if (message.currentPage != null && Object.hasOwnProperty.call(message, "currentPage"))
                    $root.exocore.index.Paging.encode(message.currentPage, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.nextPage != null && Object.hasOwnProperty.call(message, "nextPage"))
                    $root.exocore.index.Paging.encode(message.nextPage, writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                if (message.hash != null && Object.hasOwnProperty.call(message, "hash"))
                    writer.uint32(/* id 6, wireType 0 =*/48).uint64(message.hash);
                return writer;
            };

            /**
             * Encodes the specified EntityResults message, length delimited. Does not implicitly {@link exocore.index.EntityResults.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.EntityResults
             * @static
             * @param {exocore.index.IEntityResults} message EntityResults message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EntityResults.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an EntityResults message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.EntityResults
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.EntityResults} EntityResults
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EntityResults.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.EntityResults();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.entities && message.entities.length))
                            message.entities = [];
                        message.entities.push($root.exocore.index.EntityResult.decode(reader, reader.uint32()));
                        break;
                    case 2:
                        message.summary = reader.bool();
                        break;
                    case 3:
                        message.estimatedCount = reader.uint32();
                        break;
                    case 4:
                        message.currentPage = $root.exocore.index.Paging.decode(reader, reader.uint32());
                        break;
                    case 5:
                        message.nextPage = $root.exocore.index.Paging.decode(reader, reader.uint32());
                        break;
                    case 6:
                        message.hash = reader.uint64();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an EntityResults message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.EntityResults
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.EntityResults} EntityResults
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EntityResults.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an EntityResults message.
             * @function verify
             * @memberof exocore.index.EntityResults
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            EntityResults.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.entities != null && message.hasOwnProperty("entities")) {
                    if (!Array.isArray(message.entities))
                        return "entities: array expected";
                    for (let i = 0; i < message.entities.length; ++i) {
                        let error = $root.exocore.index.EntityResult.verify(message.entities[i]);
                        if (error)
                            return "entities." + error;
                    }
                }
                if (message.summary != null && message.hasOwnProperty("summary"))
                    if (typeof message.summary !== "boolean")
                        return "summary: boolean expected";
                if (message.estimatedCount != null && message.hasOwnProperty("estimatedCount"))
                    if (!$util.isInteger(message.estimatedCount))
                        return "estimatedCount: integer expected";
                if (message.currentPage != null && message.hasOwnProperty("currentPage")) {
                    let error = $root.exocore.index.Paging.verify(message.currentPage);
                    if (error)
                        return "currentPage." + error;
                }
                if (message.nextPage != null && message.hasOwnProperty("nextPage")) {
                    let error = $root.exocore.index.Paging.verify(message.nextPage);
                    if (error)
                        return "nextPage." + error;
                }
                if (message.hash != null && message.hasOwnProperty("hash"))
                    if (!$util.isInteger(message.hash) && !(message.hash && $util.isInteger(message.hash.low) && $util.isInteger(message.hash.high)))
                        return "hash: integer|Long expected";
                return null;
            };

            /**
             * Creates an EntityResults message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.EntityResults
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.EntityResults} EntityResults
             */
            EntityResults.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.EntityResults)
                    return object;
                let message = new $root.exocore.index.EntityResults();
                if (object.entities) {
                    if (!Array.isArray(object.entities))
                        throw TypeError(".exocore.index.EntityResults.entities: array expected");
                    message.entities = [];
                    for (let i = 0; i < object.entities.length; ++i) {
                        if (typeof object.entities[i] !== "object")
                            throw TypeError(".exocore.index.EntityResults.entities: object expected");
                        message.entities[i] = $root.exocore.index.EntityResult.fromObject(object.entities[i]);
                    }
                }
                if (object.summary != null)
                    message.summary = Boolean(object.summary);
                if (object.estimatedCount != null)
                    message.estimatedCount = object.estimatedCount >>> 0;
                if (object.currentPage != null) {
                    if (typeof object.currentPage !== "object")
                        throw TypeError(".exocore.index.EntityResults.currentPage: object expected");
                    message.currentPage = $root.exocore.index.Paging.fromObject(object.currentPage);
                }
                if (object.nextPage != null) {
                    if (typeof object.nextPage !== "object")
                        throw TypeError(".exocore.index.EntityResults.nextPage: object expected");
                    message.nextPage = $root.exocore.index.Paging.fromObject(object.nextPage);
                }
                if (object.hash != null)
                    if ($util.Long)
                        (message.hash = $util.Long.fromValue(object.hash)).unsigned = true;
                    else if (typeof object.hash === "string")
                        message.hash = parseInt(object.hash, 10);
                    else if (typeof object.hash === "number")
                        message.hash = object.hash;
                    else if (typeof object.hash === "object")
                        message.hash = new $util.LongBits(object.hash.low >>> 0, object.hash.high >>> 0).toNumber(true);
                return message;
            };

            /**
             * Creates a plain object from an EntityResults message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.EntityResults
             * @static
             * @param {exocore.index.EntityResults} message EntityResults
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            EntityResults.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.entities = [];
                if (options.defaults) {
                    object.summary = false;
                    object.estimatedCount = 0;
                    object.currentPage = null;
                    object.nextPage = null;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.hash = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.hash = options.longs === String ? "0" : 0;
                }
                if (message.entities && message.entities.length) {
                    object.entities = [];
                    for (let j = 0; j < message.entities.length; ++j)
                        object.entities[j] = $root.exocore.index.EntityResult.toObject(message.entities[j], options);
                }
                if (message.summary != null && message.hasOwnProperty("summary"))
                    object.summary = message.summary;
                if (message.estimatedCount != null && message.hasOwnProperty("estimatedCount"))
                    object.estimatedCount = message.estimatedCount;
                if (message.currentPage != null && message.hasOwnProperty("currentPage"))
                    object.currentPage = $root.exocore.index.Paging.toObject(message.currentPage, options);
                if (message.nextPage != null && message.hasOwnProperty("nextPage"))
                    object.nextPage = $root.exocore.index.Paging.toObject(message.nextPage, options);
                if (message.hash != null && message.hasOwnProperty("hash"))
                    if (typeof message.hash === "number")
                        object.hash = options.longs === String ? String(message.hash) : message.hash;
                    else
                        object.hash = options.longs === String ? $util.Long.prototype.toString.call(message.hash) : options.longs === Number ? new $util.LongBits(message.hash.low >>> 0, message.hash.high >>> 0).toNumber(true) : message.hash;
                return object;
            };

            /**
             * Converts this EntityResults to JSON.
             * @function toJSON
             * @memberof exocore.index.EntityResults
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            EntityResults.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return EntityResults;
        })();

        index.EntityResult = (function() {

            /**
             * Properties of an EntityResult.
             * @memberof exocore.index
             * @interface IEntityResult
             * @property {exocore.index.IEntity|null} [entity] EntityResult entity
             * @property {exocore.index.EntityResultSource|null} [source] EntityResult source
             * @property {exocore.index.IOrderingValue|null} [orderingValue] EntityResult orderingValue
             */

            /**
             * Constructs a new EntityResult.
             * @memberof exocore.index
             * @classdesc Represents an EntityResult.
             * @implements IEntityResult
             * @constructor
             * @param {exocore.index.IEntityResult=} [properties] Properties to set
             */
            function EntityResult(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * EntityResult entity.
             * @member {exocore.index.IEntity|null|undefined} entity
             * @memberof exocore.index.EntityResult
             * @instance
             */
            EntityResult.prototype.entity = null;

            /**
             * EntityResult source.
             * @member {exocore.index.EntityResultSource} source
             * @memberof exocore.index.EntityResult
             * @instance
             */
            EntityResult.prototype.source = 0;

            /**
             * EntityResult orderingValue.
             * @member {exocore.index.IOrderingValue|null|undefined} orderingValue
             * @memberof exocore.index.EntityResult
             * @instance
             */
            EntityResult.prototype.orderingValue = null;

            /**
             * Creates a new EntityResult instance using the specified properties.
             * @function create
             * @memberof exocore.index.EntityResult
             * @static
             * @param {exocore.index.IEntityResult=} [properties] Properties to set
             * @returns {exocore.index.EntityResult} EntityResult instance
             */
            EntityResult.create = function create(properties) {
                return new EntityResult(properties);
            };

            /**
             * Encodes the specified EntityResult message. Does not implicitly {@link exocore.index.EntityResult.verify|verify} messages.
             * @function encode
             * @memberof exocore.index.EntityResult
             * @static
             * @param {exocore.index.IEntityResult} message EntityResult message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EntityResult.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.entity != null && Object.hasOwnProperty.call(message, "entity"))
                    $root.exocore.index.Entity.encode(message.entity, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.source != null && Object.hasOwnProperty.call(message, "source"))
                    writer.uint32(/* id 2, wireType 0 =*/16).int32(message.source);
                if (message.orderingValue != null && Object.hasOwnProperty.call(message, "orderingValue"))
                    $root.exocore.index.OrderingValue.encode(message.orderingValue, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified EntityResult message, length delimited. Does not implicitly {@link exocore.index.EntityResult.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.index.EntityResult
             * @static
             * @param {exocore.index.IEntityResult} message EntityResult message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EntityResult.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an EntityResult message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.index.EntityResult
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.index.EntityResult} EntityResult
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EntityResult.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.index.EntityResult();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.entity = $root.exocore.index.Entity.decode(reader, reader.uint32());
                        break;
                    case 2:
                        message.source = reader.int32();
                        break;
                    case 3:
                        message.orderingValue = $root.exocore.index.OrderingValue.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an EntityResult message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.index.EntityResult
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.index.EntityResult} EntityResult
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EntityResult.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an EntityResult message.
             * @function verify
             * @memberof exocore.index.EntityResult
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            EntityResult.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.entity != null && message.hasOwnProperty("entity")) {
                    let error = $root.exocore.index.Entity.verify(message.entity);
                    if (error)
                        return "entity." + error;
                }
                if (message.source != null && message.hasOwnProperty("source"))
                    switch (message.source) {
                    default:
                        return "source: enum value expected";
                    case 0:
                    case 1:
                    case 2:
                        break;
                    }
                if (message.orderingValue != null && message.hasOwnProperty("orderingValue")) {
                    let error = $root.exocore.index.OrderingValue.verify(message.orderingValue);
                    if (error)
                        return "orderingValue." + error;
                }
                return null;
            };

            /**
             * Creates an EntityResult message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.index.EntityResult
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.index.EntityResult} EntityResult
             */
            EntityResult.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.index.EntityResult)
                    return object;
                let message = new $root.exocore.index.EntityResult();
                if (object.entity != null) {
                    if (typeof object.entity !== "object")
                        throw TypeError(".exocore.index.EntityResult.entity: object expected");
                    message.entity = $root.exocore.index.Entity.fromObject(object.entity);
                }
                switch (object.source) {
                case "UNKNOWN":
                case 0:
                    message.source = 0;
                    break;
                case "PENDING":
                case 1:
                    message.source = 1;
                    break;
                case "CHAIN":
                case 2:
                    message.source = 2;
                    break;
                }
                if (object.orderingValue != null) {
                    if (typeof object.orderingValue !== "object")
                        throw TypeError(".exocore.index.EntityResult.orderingValue: object expected");
                    message.orderingValue = $root.exocore.index.OrderingValue.fromObject(object.orderingValue);
                }
                return message;
            };

            /**
             * Creates a plain object from an EntityResult message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.index.EntityResult
             * @static
             * @param {exocore.index.EntityResult} message EntityResult
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            EntityResult.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.entity = null;
                    object.source = options.enums === String ? "UNKNOWN" : 0;
                    object.orderingValue = null;
                }
                if (message.entity != null && message.hasOwnProperty("entity"))
                    object.entity = $root.exocore.index.Entity.toObject(message.entity, options);
                if (message.source != null && message.hasOwnProperty("source"))
                    object.source = options.enums === String ? $root.exocore.index.EntityResultSource[message.source] : message.source;
                if (message.orderingValue != null && message.hasOwnProperty("orderingValue"))
                    object.orderingValue = $root.exocore.index.OrderingValue.toObject(message.orderingValue, options);
                return object;
            };

            /**
             * Converts this EntityResult to JSON.
             * @function toJSON
             * @memberof exocore.index.EntityResult
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            EntityResult.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return EntityResult;
        })();

        /**
         * EntityResultSource enum.
         * @name exocore.index.EntityResultSource
         * @enum {number}
         * @property {number} UNKNOWN=0 UNKNOWN value
         * @property {number} PENDING=1 PENDING value
         * @property {number} CHAIN=2 CHAIN value
         */
        index.EntityResultSource = (function() {
            const valuesById = {}, values = Object.create(valuesById);
            values[valuesById[0] = "UNKNOWN"] = 0;
            values[valuesById[1] = "PENDING"] = 1;
            values[valuesById[2] = "CHAIN"] = 2;
            return values;
        })();

        return index;
    })();

    exocore.test = (function() {

        /**
         * Namespace test.
         * @memberof exocore
         * @namespace
         */
        const test = {};

        test.TestMessage = (function() {

            /**
             * Properties of a TestMessage.
             * @memberof exocore.test
             * @interface ITestMessage
             * @property {string|null} [string1] TestMessage string1
             * @property {string|null} [string2] TestMessage string2
             * @property {string|null} [string3] TestMessage string3
             * @property {exocore.test.ITestStruct|null} [struct1] TestMessage struct1
             * @property {string|null} [oneofString1] TestMessage oneofString1
             * @property {number|null} [oneofInt1] TestMessage oneofInt1
             * @property {google.protobuf.ITimestamp|null} [date1] TestMessage date1
             * @property {google.protobuf.ITimestamp|null} [date2] TestMessage date2
             * @property {google.protobuf.ITimestamp|null} [date3] TestMessage date3
             * @property {number|null} [uint1] TestMessage uint1
             * @property {number|null} [uint2] TestMessage uint2
             * @property {number|null} [uint3] TestMessage uint3
             * @property {number|null} [int1] TestMessage int1
             * @property {number|null} [int2] TestMessage int2
             * @property {number|null} [int3] TestMessage int3
             * @property {exocore.index.IReference|null} [ref1] TestMessage ref1
             * @property {exocore.index.IReference|null} [ref2] TestMessage ref2
             */

            /**
             * Constructs a new TestMessage.
             * @memberof exocore.test
             * @classdesc Represents a TestMessage.
             * @implements ITestMessage
             * @constructor
             * @param {exocore.test.ITestMessage=} [properties] Properties to set
             */
            function TestMessage(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TestMessage string1.
             * @member {string} string1
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.string1 = "";

            /**
             * TestMessage string2.
             * @member {string} string2
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.string2 = "";

            /**
             * TestMessage string3.
             * @member {string} string3
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.string3 = "";

            /**
             * TestMessage struct1.
             * @member {exocore.test.ITestStruct|null|undefined} struct1
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.struct1 = null;

            /**
             * TestMessage oneofString1.
             * @member {string} oneofString1
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.oneofString1 = "";

            /**
             * TestMessage oneofInt1.
             * @member {number} oneofInt1
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.oneofInt1 = 0;

            /**
             * TestMessage date1.
             * @member {google.protobuf.ITimestamp|null|undefined} date1
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.date1 = null;

            /**
             * TestMessage date2.
             * @member {google.protobuf.ITimestamp|null|undefined} date2
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.date2 = null;

            /**
             * TestMessage date3.
             * @member {google.protobuf.ITimestamp|null|undefined} date3
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.date3 = null;

            /**
             * TestMessage uint1.
             * @member {number} uint1
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.uint1 = 0;

            /**
             * TestMessage uint2.
             * @member {number} uint2
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.uint2 = 0;

            /**
             * TestMessage uint3.
             * @member {number} uint3
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.uint3 = 0;

            /**
             * TestMessage int1.
             * @member {number} int1
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.int1 = 0;

            /**
             * TestMessage int2.
             * @member {number} int2
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.int2 = 0;

            /**
             * TestMessage int3.
             * @member {number} int3
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.int3 = 0;

            /**
             * TestMessage ref1.
             * @member {exocore.index.IReference|null|undefined} ref1
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.ref1 = null;

            /**
             * TestMessage ref2.
             * @member {exocore.index.IReference|null|undefined} ref2
             * @memberof exocore.test.TestMessage
             * @instance
             */
            TestMessage.prototype.ref2 = null;

            // OneOf field names bound to virtual getters and setters
            let $oneOfFields;

            /**
             * TestMessage fields.
             * @member {"oneofString1"|"oneofInt1"|undefined} fields
             * @memberof exocore.test.TestMessage
             * @instance
             */
            Object.defineProperty(TestMessage.prototype, "fields", {
                get: $util.oneOfGetter($oneOfFields = ["oneofString1", "oneofInt1"]),
                set: $util.oneOfSetter($oneOfFields)
            });

            /**
             * Creates a new TestMessage instance using the specified properties.
             * @function create
             * @memberof exocore.test.TestMessage
             * @static
             * @param {exocore.test.ITestMessage=} [properties] Properties to set
             * @returns {exocore.test.TestMessage} TestMessage instance
             */
            TestMessage.create = function create(properties) {
                return new TestMessage(properties);
            };

            /**
             * Encodes the specified TestMessage message. Does not implicitly {@link exocore.test.TestMessage.verify|verify} messages.
             * @function encode
             * @memberof exocore.test.TestMessage
             * @static
             * @param {exocore.test.ITestMessage} message TestMessage message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestMessage.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.string1 != null && Object.hasOwnProperty.call(message, "string1"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.string1);
                if (message.string2 != null && Object.hasOwnProperty.call(message, "string2"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.string2);
                if (message.struct1 != null && Object.hasOwnProperty.call(message, "struct1"))
                    $root.exocore.test.TestStruct.encode(message.struct1, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.oneofString1 != null && Object.hasOwnProperty.call(message, "oneofString1"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.oneofString1);
                if (message.oneofInt1 != null && Object.hasOwnProperty.call(message, "oneofInt1"))
                    writer.uint32(/* id 5, wireType 0 =*/40).uint32(message.oneofInt1);
                if (message.date1 != null && Object.hasOwnProperty.call(message, "date1"))
                    $root.google.protobuf.Timestamp.encode(message.date1, writer.uint32(/* id 8, wireType 2 =*/66).fork()).ldelim();
                if (message.date2 != null && Object.hasOwnProperty.call(message, "date2"))
                    $root.google.protobuf.Timestamp.encode(message.date2, writer.uint32(/* id 9, wireType 2 =*/74).fork()).ldelim();
                if (message.uint1 != null && Object.hasOwnProperty.call(message, "uint1"))
                    writer.uint32(/* id 10, wireType 0 =*/80).uint32(message.uint1);
                if (message.uint2 != null && Object.hasOwnProperty.call(message, "uint2"))
                    writer.uint32(/* id 11, wireType 0 =*/88).uint32(message.uint2);
                if (message.string3 != null && Object.hasOwnProperty.call(message, "string3"))
                    writer.uint32(/* id 12, wireType 2 =*/98).string(message.string3);
                if (message.ref1 != null && Object.hasOwnProperty.call(message, "ref1"))
                    $root.exocore.index.Reference.encode(message.ref1, writer.uint32(/* id 13, wireType 2 =*/106).fork()).ldelim();
                if (message.ref2 != null && Object.hasOwnProperty.call(message, "ref2"))
                    $root.exocore.index.Reference.encode(message.ref2, writer.uint32(/* id 14, wireType 2 =*/114).fork()).ldelim();
                if (message.int1 != null && Object.hasOwnProperty.call(message, "int1"))
                    writer.uint32(/* id 15, wireType 0 =*/120).int32(message.int1);
                if (message.int2 != null && Object.hasOwnProperty.call(message, "int2"))
                    writer.uint32(/* id 16, wireType 0 =*/128).int32(message.int2);
                if (message.date3 != null && Object.hasOwnProperty.call(message, "date3"))
                    $root.google.protobuf.Timestamp.encode(message.date3, writer.uint32(/* id 17, wireType 2 =*/138).fork()).ldelim();
                if (message.uint3 != null && Object.hasOwnProperty.call(message, "uint3"))
                    writer.uint32(/* id 18, wireType 0 =*/144).uint32(message.uint3);
                if (message.int3 != null && Object.hasOwnProperty.call(message, "int3"))
                    writer.uint32(/* id 19, wireType 0 =*/152).int32(message.int3);
                return writer;
            };

            /**
             * Encodes the specified TestMessage message, length delimited. Does not implicitly {@link exocore.test.TestMessage.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.test.TestMessage
             * @static
             * @param {exocore.test.ITestMessage} message TestMessage message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestMessage.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TestMessage message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.test.TestMessage
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.test.TestMessage} TestMessage
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestMessage.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.test.TestMessage();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.string1 = reader.string();
                        break;
                    case 2:
                        message.string2 = reader.string();
                        break;
                    case 12:
                        message.string3 = reader.string();
                        break;
                    case 3:
                        message.struct1 = $root.exocore.test.TestStruct.decode(reader, reader.uint32());
                        break;
                    case 4:
                        message.oneofString1 = reader.string();
                        break;
                    case 5:
                        message.oneofInt1 = reader.uint32();
                        break;
                    case 8:
                        message.date1 = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                        break;
                    case 9:
                        message.date2 = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                        break;
                    case 17:
                        message.date3 = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                        break;
                    case 10:
                        message.uint1 = reader.uint32();
                        break;
                    case 11:
                        message.uint2 = reader.uint32();
                        break;
                    case 18:
                        message.uint3 = reader.uint32();
                        break;
                    case 15:
                        message.int1 = reader.int32();
                        break;
                    case 16:
                        message.int2 = reader.int32();
                        break;
                    case 19:
                        message.int3 = reader.int32();
                        break;
                    case 13:
                        message.ref1 = $root.exocore.index.Reference.decode(reader, reader.uint32());
                        break;
                    case 14:
                        message.ref2 = $root.exocore.index.Reference.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a TestMessage message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.test.TestMessage
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.test.TestMessage} TestMessage
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestMessage.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TestMessage message.
             * @function verify
             * @memberof exocore.test.TestMessage
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TestMessage.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                let properties = {};
                if (message.string1 != null && message.hasOwnProperty("string1"))
                    if (!$util.isString(message.string1))
                        return "string1: string expected";
                if (message.string2 != null && message.hasOwnProperty("string2"))
                    if (!$util.isString(message.string2))
                        return "string2: string expected";
                if (message.string3 != null && message.hasOwnProperty("string3"))
                    if (!$util.isString(message.string3))
                        return "string3: string expected";
                if (message.struct1 != null && message.hasOwnProperty("struct1")) {
                    let error = $root.exocore.test.TestStruct.verify(message.struct1);
                    if (error)
                        return "struct1." + error;
                }
                if (message.oneofString1 != null && message.hasOwnProperty("oneofString1")) {
                    properties.fields = 1;
                    if (!$util.isString(message.oneofString1))
                        return "oneofString1: string expected";
                }
                if (message.oneofInt1 != null && message.hasOwnProperty("oneofInt1")) {
                    if (properties.fields === 1)
                        return "fields: multiple values";
                    properties.fields = 1;
                    if (!$util.isInteger(message.oneofInt1))
                        return "oneofInt1: integer expected";
                }
                if (message.date1 != null && message.hasOwnProperty("date1")) {
                    let error = $root.google.protobuf.Timestamp.verify(message.date1);
                    if (error)
                        return "date1." + error;
                }
                if (message.date2 != null && message.hasOwnProperty("date2")) {
                    let error = $root.google.protobuf.Timestamp.verify(message.date2);
                    if (error)
                        return "date2." + error;
                }
                if (message.date3 != null && message.hasOwnProperty("date3")) {
                    let error = $root.google.protobuf.Timestamp.verify(message.date3);
                    if (error)
                        return "date3." + error;
                }
                if (message.uint1 != null && message.hasOwnProperty("uint1"))
                    if (!$util.isInteger(message.uint1))
                        return "uint1: integer expected";
                if (message.uint2 != null && message.hasOwnProperty("uint2"))
                    if (!$util.isInteger(message.uint2))
                        return "uint2: integer expected";
                if (message.uint3 != null && message.hasOwnProperty("uint3"))
                    if (!$util.isInteger(message.uint3))
                        return "uint3: integer expected";
                if (message.int1 != null && message.hasOwnProperty("int1"))
                    if (!$util.isInteger(message.int1))
                        return "int1: integer expected";
                if (message.int2 != null && message.hasOwnProperty("int2"))
                    if (!$util.isInteger(message.int2))
                        return "int2: integer expected";
                if (message.int3 != null && message.hasOwnProperty("int3"))
                    if (!$util.isInteger(message.int3))
                        return "int3: integer expected";
                if (message.ref1 != null && message.hasOwnProperty("ref1")) {
                    let error = $root.exocore.index.Reference.verify(message.ref1);
                    if (error)
                        return "ref1." + error;
                }
                if (message.ref2 != null && message.hasOwnProperty("ref2")) {
                    let error = $root.exocore.index.Reference.verify(message.ref2);
                    if (error)
                        return "ref2." + error;
                }
                return null;
            };

            /**
             * Creates a TestMessage message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.test.TestMessage
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.test.TestMessage} TestMessage
             */
            TestMessage.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.test.TestMessage)
                    return object;
                let message = new $root.exocore.test.TestMessage();
                if (object.string1 != null)
                    message.string1 = String(object.string1);
                if (object.string2 != null)
                    message.string2 = String(object.string2);
                if (object.string3 != null)
                    message.string3 = String(object.string3);
                if (object.struct1 != null) {
                    if (typeof object.struct1 !== "object")
                        throw TypeError(".exocore.test.TestMessage.struct1: object expected");
                    message.struct1 = $root.exocore.test.TestStruct.fromObject(object.struct1);
                }
                if (object.oneofString1 != null)
                    message.oneofString1 = String(object.oneofString1);
                if (object.oneofInt1 != null)
                    message.oneofInt1 = object.oneofInt1 >>> 0;
                if (object.date1 != null) {
                    if (typeof object.date1 !== "object")
                        throw TypeError(".exocore.test.TestMessage.date1: object expected");
                    message.date1 = $root.google.protobuf.Timestamp.fromObject(object.date1);
                }
                if (object.date2 != null) {
                    if (typeof object.date2 !== "object")
                        throw TypeError(".exocore.test.TestMessage.date2: object expected");
                    message.date2 = $root.google.protobuf.Timestamp.fromObject(object.date2);
                }
                if (object.date3 != null) {
                    if (typeof object.date3 !== "object")
                        throw TypeError(".exocore.test.TestMessage.date3: object expected");
                    message.date3 = $root.google.protobuf.Timestamp.fromObject(object.date3);
                }
                if (object.uint1 != null)
                    message.uint1 = object.uint1 >>> 0;
                if (object.uint2 != null)
                    message.uint2 = object.uint2 >>> 0;
                if (object.uint3 != null)
                    message.uint3 = object.uint3 >>> 0;
                if (object.int1 != null)
                    message.int1 = object.int1 | 0;
                if (object.int2 != null)
                    message.int2 = object.int2 | 0;
                if (object.int3 != null)
                    message.int3 = object.int3 | 0;
                if (object.ref1 != null) {
                    if (typeof object.ref1 !== "object")
                        throw TypeError(".exocore.test.TestMessage.ref1: object expected");
                    message.ref1 = $root.exocore.index.Reference.fromObject(object.ref1);
                }
                if (object.ref2 != null) {
                    if (typeof object.ref2 !== "object")
                        throw TypeError(".exocore.test.TestMessage.ref2: object expected");
                    message.ref2 = $root.exocore.index.Reference.fromObject(object.ref2);
                }
                return message;
            };

            /**
             * Creates a plain object from a TestMessage message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.test.TestMessage
             * @static
             * @param {exocore.test.TestMessage} message TestMessage
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TestMessage.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.string1 = "";
                    object.string2 = "";
                    object.struct1 = null;
                    object.date1 = null;
                    object.date2 = null;
                    object.uint1 = 0;
                    object.uint2 = 0;
                    object.string3 = "";
                    object.ref1 = null;
                    object.ref2 = null;
                    object.int1 = 0;
                    object.int2 = 0;
                    object.date3 = null;
                    object.uint3 = 0;
                    object.int3 = 0;
                }
                if (message.string1 != null && message.hasOwnProperty("string1"))
                    object.string1 = message.string1;
                if (message.string2 != null && message.hasOwnProperty("string2"))
                    object.string2 = message.string2;
                if (message.struct1 != null && message.hasOwnProperty("struct1"))
                    object.struct1 = $root.exocore.test.TestStruct.toObject(message.struct1, options);
                if (message.oneofString1 != null && message.hasOwnProperty("oneofString1")) {
                    object.oneofString1 = message.oneofString1;
                    if (options.oneofs)
                        object.fields = "oneofString1";
                }
                if (message.oneofInt1 != null && message.hasOwnProperty("oneofInt1")) {
                    object.oneofInt1 = message.oneofInt1;
                    if (options.oneofs)
                        object.fields = "oneofInt1";
                }
                if (message.date1 != null && message.hasOwnProperty("date1"))
                    object.date1 = $root.google.protobuf.Timestamp.toObject(message.date1, options);
                if (message.date2 != null && message.hasOwnProperty("date2"))
                    object.date2 = $root.google.protobuf.Timestamp.toObject(message.date2, options);
                if (message.uint1 != null && message.hasOwnProperty("uint1"))
                    object.uint1 = message.uint1;
                if (message.uint2 != null && message.hasOwnProperty("uint2"))
                    object.uint2 = message.uint2;
                if (message.string3 != null && message.hasOwnProperty("string3"))
                    object.string3 = message.string3;
                if (message.ref1 != null && message.hasOwnProperty("ref1"))
                    object.ref1 = $root.exocore.index.Reference.toObject(message.ref1, options);
                if (message.ref2 != null && message.hasOwnProperty("ref2"))
                    object.ref2 = $root.exocore.index.Reference.toObject(message.ref2, options);
                if (message.int1 != null && message.hasOwnProperty("int1"))
                    object.int1 = message.int1;
                if (message.int2 != null && message.hasOwnProperty("int2"))
                    object.int2 = message.int2;
                if (message.date3 != null && message.hasOwnProperty("date3"))
                    object.date3 = $root.google.protobuf.Timestamp.toObject(message.date3, options);
                if (message.uint3 != null && message.hasOwnProperty("uint3"))
                    object.uint3 = message.uint3;
                if (message.int3 != null && message.hasOwnProperty("int3"))
                    object.int3 = message.int3;
                return object;
            };

            /**
             * Converts this TestMessage to JSON.
             * @function toJSON
             * @memberof exocore.test.TestMessage
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TestMessage.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return TestMessage;
        })();

        test.TestStruct = (function() {

            /**
             * Properties of a TestStruct.
             * @memberof exocore.test
             * @interface ITestStruct
             * @property {string|null} [string1] TestStruct string1
             */

            /**
             * Constructs a new TestStruct.
             * @memberof exocore.test
             * @classdesc Represents a TestStruct.
             * @implements ITestStruct
             * @constructor
             * @param {exocore.test.ITestStruct=} [properties] Properties to set
             */
            function TestStruct(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TestStruct string1.
             * @member {string} string1
             * @memberof exocore.test.TestStruct
             * @instance
             */
            TestStruct.prototype.string1 = "";

            /**
             * Creates a new TestStruct instance using the specified properties.
             * @function create
             * @memberof exocore.test.TestStruct
             * @static
             * @param {exocore.test.ITestStruct=} [properties] Properties to set
             * @returns {exocore.test.TestStruct} TestStruct instance
             */
            TestStruct.create = function create(properties) {
                return new TestStruct(properties);
            };

            /**
             * Encodes the specified TestStruct message. Does not implicitly {@link exocore.test.TestStruct.verify|verify} messages.
             * @function encode
             * @memberof exocore.test.TestStruct
             * @static
             * @param {exocore.test.ITestStruct} message TestStruct message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestStruct.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.string1 != null && Object.hasOwnProperty.call(message, "string1"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.string1);
                return writer;
            };

            /**
             * Encodes the specified TestStruct message, length delimited. Does not implicitly {@link exocore.test.TestStruct.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.test.TestStruct
             * @static
             * @param {exocore.test.ITestStruct} message TestStruct message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestStruct.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TestStruct message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.test.TestStruct
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.test.TestStruct} TestStruct
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestStruct.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.test.TestStruct();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.string1 = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a TestStruct message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.test.TestStruct
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.test.TestStruct} TestStruct
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestStruct.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TestStruct message.
             * @function verify
             * @memberof exocore.test.TestStruct
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TestStruct.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.string1 != null && message.hasOwnProperty("string1"))
                    if (!$util.isString(message.string1))
                        return "string1: string expected";
                return null;
            };

            /**
             * Creates a TestStruct message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.test.TestStruct
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.test.TestStruct} TestStruct
             */
            TestStruct.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.test.TestStruct)
                    return object;
                let message = new $root.exocore.test.TestStruct();
                if (object.string1 != null)
                    message.string1 = String(object.string1);
                return message;
            };

            /**
             * Creates a plain object from a TestStruct message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.test.TestStruct
             * @static
             * @param {exocore.test.TestStruct} message TestStruct
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TestStruct.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults)
                    object.string1 = "";
                if (message.string1 != null && message.hasOwnProperty("string1"))
                    object.string1 = message.string1;
                return object;
            };

            /**
             * Converts this TestStruct to JSON.
             * @function toJSON
             * @memberof exocore.test.TestStruct
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TestStruct.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return TestStruct;
        })();

        test.TestMessage2 = (function() {

            /**
             * Properties of a TestMessage2.
             * @memberof exocore.test
             * @interface ITestMessage2
             * @property {string|null} [string1] TestMessage2 string1
             * @property {string|null} [string2] TestMessage2 string2
             */

            /**
             * Constructs a new TestMessage2.
             * @memberof exocore.test
             * @classdesc Represents a TestMessage2.
             * @implements ITestMessage2
             * @constructor
             * @param {exocore.test.ITestMessage2=} [properties] Properties to set
             */
            function TestMessage2(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * TestMessage2 string1.
             * @member {string} string1
             * @memberof exocore.test.TestMessage2
             * @instance
             */
            TestMessage2.prototype.string1 = "";

            /**
             * TestMessage2 string2.
             * @member {string} string2
             * @memberof exocore.test.TestMessage2
             * @instance
             */
            TestMessage2.prototype.string2 = "";

            /**
             * Creates a new TestMessage2 instance using the specified properties.
             * @function create
             * @memberof exocore.test.TestMessage2
             * @static
             * @param {exocore.test.ITestMessage2=} [properties] Properties to set
             * @returns {exocore.test.TestMessage2} TestMessage2 instance
             */
            TestMessage2.create = function create(properties) {
                return new TestMessage2(properties);
            };

            /**
             * Encodes the specified TestMessage2 message. Does not implicitly {@link exocore.test.TestMessage2.verify|verify} messages.
             * @function encode
             * @memberof exocore.test.TestMessage2
             * @static
             * @param {exocore.test.ITestMessage2} message TestMessage2 message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestMessage2.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.string1 != null && Object.hasOwnProperty.call(message, "string1"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.string1);
                if (message.string2 != null && Object.hasOwnProperty.call(message, "string2"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.string2);
                return writer;
            };

            /**
             * Encodes the specified TestMessage2 message, length delimited. Does not implicitly {@link exocore.test.TestMessage2.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.test.TestMessage2
             * @static
             * @param {exocore.test.ITestMessage2} message TestMessage2 message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TestMessage2.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a TestMessage2 message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.test.TestMessage2
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.test.TestMessage2} TestMessage2
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestMessage2.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.test.TestMessage2();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.string1 = reader.string();
                        break;
                    case 2:
                        message.string2 = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a TestMessage2 message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof exocore.test.TestMessage2
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.test.TestMessage2} TestMessage2
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TestMessage2.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a TestMessage2 message.
             * @function verify
             * @memberof exocore.test.TestMessage2
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TestMessage2.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.string1 != null && message.hasOwnProperty("string1"))
                    if (!$util.isString(message.string1))
                        return "string1: string expected";
                if (message.string2 != null && message.hasOwnProperty("string2"))
                    if (!$util.isString(message.string2))
                        return "string2: string expected";
                return null;
            };

            /**
             * Creates a TestMessage2 message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof exocore.test.TestMessage2
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.test.TestMessage2} TestMessage2
             */
            TestMessage2.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.test.TestMessage2)
                    return object;
                let message = new $root.exocore.test.TestMessage2();
                if (object.string1 != null)
                    message.string1 = String(object.string1);
                if (object.string2 != null)
                    message.string2 = String(object.string2);
                return message;
            };

            /**
             * Creates a plain object from a TestMessage2 message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.test.TestMessage2
             * @static
             * @param {exocore.test.TestMessage2} message TestMessage2
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TestMessage2.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.string1 = "";
                    object.string2 = "";
                }
                if (message.string1 != null && message.hasOwnProperty("string1"))
                    object.string1 = message.string1;
                if (message.string2 != null && message.hasOwnProperty("string2"))
                    object.string2 = message.string2;
                return object;
            };

            /**
             * Converts this TestMessage2 to JSON.
             * @function toJSON
             * @memberof exocore.test.TestMessage2
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TestMessage2.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return TestMessage2;
        })();

        return test;
    })();

    return exocore;
})();

export const google = $root.google = (() => {

    /**
     * Namespace google.
     * @exports google
     * @namespace
     */
    const google = {};

    google.protobuf = (function() {

        /**
         * Namespace protobuf.
         * @memberof google
         * @namespace
         */
        const protobuf = {};

        protobuf.Timestamp = (function() {

            /**
             * Properties of a Timestamp.
             * @memberof google.protobuf
             * @interface ITimestamp
             * @property {number|Long|null} [seconds] Timestamp seconds
             * @property {number|null} [nanos] Timestamp nanos
             */

            /**
             * Constructs a new Timestamp.
             * @memberof google.protobuf
             * @classdesc Represents a Timestamp.
             * @implements ITimestamp
             * @constructor
             * @param {google.protobuf.ITimestamp=} [properties] Properties to set
             */
            function Timestamp(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Timestamp seconds.
             * @member {number|Long} seconds
             * @memberof google.protobuf.Timestamp
             * @instance
             */
            Timestamp.prototype.seconds = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

            /**
             * Timestamp nanos.
             * @member {number} nanos
             * @memberof google.protobuf.Timestamp
             * @instance
             */
            Timestamp.prototype.nanos = 0;

            /**
             * Creates a new Timestamp instance using the specified properties.
             * @function create
             * @memberof google.protobuf.Timestamp
             * @static
             * @param {google.protobuf.ITimestamp=} [properties] Properties to set
             * @returns {google.protobuf.Timestamp} Timestamp instance
             */
            Timestamp.create = function create(properties) {
                return new Timestamp(properties);
            };

            /**
             * Encodes the specified Timestamp message. Does not implicitly {@link google.protobuf.Timestamp.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.Timestamp
             * @static
             * @param {google.protobuf.ITimestamp} message Timestamp message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Timestamp.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.seconds != null && Object.hasOwnProperty.call(message, "seconds"))
                    writer.uint32(/* id 1, wireType 0 =*/8).int64(message.seconds);
                if (message.nanos != null && Object.hasOwnProperty.call(message, "nanos"))
                    writer.uint32(/* id 2, wireType 0 =*/16).int32(message.nanos);
                return writer;
            };

            /**
             * Encodes the specified Timestamp message, length delimited. Does not implicitly {@link google.protobuf.Timestamp.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.Timestamp
             * @static
             * @param {google.protobuf.ITimestamp} message Timestamp message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Timestamp.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a Timestamp message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.Timestamp
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.Timestamp} Timestamp
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Timestamp.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.Timestamp();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.seconds = reader.int64();
                        break;
                    case 2:
                        message.nanos = reader.int32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a Timestamp message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.Timestamp
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.Timestamp} Timestamp
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Timestamp.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a Timestamp message.
             * @function verify
             * @memberof google.protobuf.Timestamp
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Timestamp.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.seconds != null && message.hasOwnProperty("seconds"))
                    if (!$util.isInteger(message.seconds) && !(message.seconds && $util.isInteger(message.seconds.low) && $util.isInteger(message.seconds.high)))
                        return "seconds: integer|Long expected";
                if (message.nanos != null && message.hasOwnProperty("nanos"))
                    if (!$util.isInteger(message.nanos))
                        return "nanos: integer expected";
                return null;
            };

            /**
             * Creates a Timestamp message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.Timestamp
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.Timestamp} Timestamp
             */
            Timestamp.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.Timestamp)
                    return object;
                let message = new $root.google.protobuf.Timestamp();
                if (object.seconds != null)
                    if ($util.Long)
                        (message.seconds = $util.Long.fromValue(object.seconds)).unsigned = false;
                    else if (typeof object.seconds === "string")
                        message.seconds = parseInt(object.seconds, 10);
                    else if (typeof object.seconds === "number")
                        message.seconds = object.seconds;
                    else if (typeof object.seconds === "object")
                        message.seconds = new $util.LongBits(object.seconds.low >>> 0, object.seconds.high >>> 0).toNumber();
                if (object.nanos != null)
                    message.nanos = object.nanos | 0;
                return message;
            };

            /**
             * Creates a plain object from a Timestamp message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.Timestamp
             * @static
             * @param {google.protobuf.Timestamp} message Timestamp
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Timestamp.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, false);
                        object.seconds = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.seconds = options.longs === String ? "0" : 0;
                    object.nanos = 0;
                }
                if (message.seconds != null && message.hasOwnProperty("seconds"))
                    if (typeof message.seconds === "number")
                        object.seconds = options.longs === String ? String(message.seconds) : message.seconds;
                    else
                        object.seconds = options.longs === String ? $util.Long.prototype.toString.call(message.seconds) : options.longs === Number ? new $util.LongBits(message.seconds.low >>> 0, message.seconds.high >>> 0).toNumber() : message.seconds;
                if (message.nanos != null && message.hasOwnProperty("nanos"))
                    object.nanos = message.nanos;
                return object;
            };

            /**
             * Converts this Timestamp to JSON.
             * @function toJSON
             * @memberof google.protobuf.Timestamp
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Timestamp.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return Timestamp;
        })();

        protobuf.Any = (function() {

            /**
             * Properties of an Any.
             * @memberof google.protobuf
             * @interface IAny
             * @property {string|null} [type_url] Any type_url
             * @property {Uint8Array|null} [value] Any value
             */

            /**
             * Constructs a new Any.
             * @memberof google.protobuf
             * @classdesc Represents an Any.
             * @implements IAny
             * @constructor
             * @param {google.protobuf.IAny=} [properties] Properties to set
             */
            function Any(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * Any type_url.
             * @member {string} type_url
             * @memberof google.protobuf.Any
             * @instance
             */
            Any.prototype.type_url = "";

            /**
             * Any value.
             * @member {Uint8Array} value
             * @memberof google.protobuf.Any
             * @instance
             */
            Any.prototype.value = $util.newBuffer([]);

            /**
             * Creates a new Any instance using the specified properties.
             * @function create
             * @memberof google.protobuf.Any
             * @static
             * @param {google.protobuf.IAny=} [properties] Properties to set
             * @returns {google.protobuf.Any} Any instance
             */
            Any.create = function create(properties) {
                return new Any(properties);
            };

            /**
             * Encodes the specified Any message. Does not implicitly {@link google.protobuf.Any.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.Any
             * @static
             * @param {google.protobuf.IAny} message Any message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Any.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.type_url != null && Object.hasOwnProperty.call(message, "type_url"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.type_url);
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.value);
                return writer;
            };

            /**
             * Encodes the specified Any message, length delimited. Does not implicitly {@link google.protobuf.Any.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.Any
             * @static
             * @param {google.protobuf.IAny} message Any message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Any.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an Any message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.Any
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.Any} Any
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Any.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.Any();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.type_url = reader.string();
                        break;
                    case 2:
                        message.value = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an Any message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.Any
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.Any} Any
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Any.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an Any message.
             * @function verify
             * @memberof google.protobuf.Any
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Any.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.type_url != null && message.hasOwnProperty("type_url"))
                    if (!$util.isString(message.type_url))
                        return "type_url: string expected";
                if (message.value != null && message.hasOwnProperty("value"))
                    if (!(message.value && typeof message.value.length === "number" || $util.isString(message.value)))
                        return "value: buffer expected";
                return null;
            };

            /**
             * Creates an Any message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.Any
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.Any} Any
             */
            Any.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.Any)
                    return object;
                let message = new $root.google.protobuf.Any();
                if (object.type_url != null)
                    message.type_url = String(object.type_url);
                if (object.value != null)
                    if (typeof object.value === "string")
                        $util.base64.decode(object.value, message.value = $util.newBuffer($util.base64.length(object.value)), 0);
                    else if (object.value.length)
                        message.value = object.value;
                return message;
            };

            /**
             * Creates a plain object from an Any message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.Any
             * @static
             * @param {google.protobuf.Any} message Any
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Any.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.type_url = "";
                    if (options.bytes === String)
                        object.value = "";
                    else {
                        object.value = [];
                        if (options.bytes !== Array)
                            object.value = $util.newBuffer(object.value);
                    }
                }
                if (message.type_url != null && message.hasOwnProperty("type_url"))
                    object.type_url = message.type_url;
                if (message.value != null && message.hasOwnProperty("value"))
                    object.value = options.bytes === String ? $util.base64.encode(message.value, 0, message.value.length) : options.bytes === Array ? Array.prototype.slice.call(message.value) : message.value;
                return object;
            };

            /**
             * Converts this Any to JSON.
             * @function toJSON
             * @memberof google.protobuf.Any
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Any.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return Any;
        })();

        protobuf.FieldMask = (function() {

            /**
             * Properties of a FieldMask.
             * @memberof google.protobuf
             * @interface IFieldMask
             * @property {Array.<string>|null} [paths] FieldMask paths
             */

            /**
             * Constructs a new FieldMask.
             * @memberof google.protobuf
             * @classdesc Represents a FieldMask.
             * @implements IFieldMask
             * @constructor
             * @param {google.protobuf.IFieldMask=} [properties] Properties to set
             */
            function FieldMask(properties) {
                this.paths = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FieldMask paths.
             * @member {Array.<string>} paths
             * @memberof google.protobuf.FieldMask
             * @instance
             */
            FieldMask.prototype.paths = $util.emptyArray;

            /**
             * Creates a new FieldMask instance using the specified properties.
             * @function create
             * @memberof google.protobuf.FieldMask
             * @static
             * @param {google.protobuf.IFieldMask=} [properties] Properties to set
             * @returns {google.protobuf.FieldMask} FieldMask instance
             */
            FieldMask.create = function create(properties) {
                return new FieldMask(properties);
            };

            /**
             * Encodes the specified FieldMask message. Does not implicitly {@link google.protobuf.FieldMask.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.FieldMask
             * @static
             * @param {google.protobuf.IFieldMask} message FieldMask message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FieldMask.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.paths != null && message.paths.length)
                    for (let i = 0; i < message.paths.length; ++i)
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.paths[i]);
                return writer;
            };

            /**
             * Encodes the specified FieldMask message, length delimited. Does not implicitly {@link google.protobuf.FieldMask.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.FieldMask
             * @static
             * @param {google.protobuf.IFieldMask} message FieldMask message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FieldMask.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FieldMask message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.FieldMask
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.FieldMask} FieldMask
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FieldMask.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.FieldMask();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.paths && message.paths.length))
                            message.paths = [];
                        message.paths.push(reader.string());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FieldMask message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.FieldMask
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.FieldMask} FieldMask
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FieldMask.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FieldMask message.
             * @function verify
             * @memberof google.protobuf.FieldMask
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FieldMask.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.paths != null && message.hasOwnProperty("paths")) {
                    if (!Array.isArray(message.paths))
                        return "paths: array expected";
                    for (let i = 0; i < message.paths.length; ++i)
                        if (!$util.isString(message.paths[i]))
                            return "paths: string[] expected";
                }
                return null;
            };

            /**
             * Creates a FieldMask message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.FieldMask
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.FieldMask} FieldMask
             */
            FieldMask.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.FieldMask)
                    return object;
                let message = new $root.google.protobuf.FieldMask();
                if (object.paths) {
                    if (!Array.isArray(object.paths))
                        throw TypeError(".google.protobuf.FieldMask.paths: array expected");
                    message.paths = [];
                    for (let i = 0; i < object.paths.length; ++i)
                        message.paths[i] = String(object.paths[i]);
                }
                return message;
            };

            /**
             * Creates a plain object from a FieldMask message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.FieldMask
             * @static
             * @param {google.protobuf.FieldMask} message FieldMask
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FieldMask.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.paths = [];
                if (message.paths && message.paths.length) {
                    object.paths = [];
                    for (let j = 0; j < message.paths.length; ++j)
                        object.paths[j] = message.paths[j];
                }
                return object;
            };

            /**
             * Converts this FieldMask to JSON.
             * @function toJSON
             * @memberof google.protobuf.FieldMask
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FieldMask.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return FieldMask;
        })();

        protobuf.FileDescriptorSet = (function() {

            /**
             * Properties of a FileDescriptorSet.
             * @memberof google.protobuf
             * @interface IFileDescriptorSet
             * @property {Array.<google.protobuf.IFileDescriptorProto>|null} [file] FileDescriptorSet file
             */

            /**
             * Constructs a new FileDescriptorSet.
             * @memberof google.protobuf
             * @classdesc Represents a FileDescriptorSet.
             * @implements IFileDescriptorSet
             * @constructor
             * @param {google.protobuf.IFileDescriptorSet=} [properties] Properties to set
             */
            function FileDescriptorSet(properties) {
                this.file = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FileDescriptorSet file.
             * @member {Array.<google.protobuf.IFileDescriptorProto>} file
             * @memberof google.protobuf.FileDescriptorSet
             * @instance
             */
            FileDescriptorSet.prototype.file = $util.emptyArray;

            /**
             * Creates a new FileDescriptorSet instance using the specified properties.
             * @function create
             * @memberof google.protobuf.FileDescriptorSet
             * @static
             * @param {google.protobuf.IFileDescriptorSet=} [properties] Properties to set
             * @returns {google.protobuf.FileDescriptorSet} FileDescriptorSet instance
             */
            FileDescriptorSet.create = function create(properties) {
                return new FileDescriptorSet(properties);
            };

            /**
             * Encodes the specified FileDescriptorSet message. Does not implicitly {@link google.protobuf.FileDescriptorSet.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.FileDescriptorSet
             * @static
             * @param {google.protobuf.IFileDescriptorSet} message FileDescriptorSet message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FileDescriptorSet.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.file != null && message.file.length)
                    for (let i = 0; i < message.file.length; ++i)
                        $root.google.protobuf.FileDescriptorProto.encode(message.file[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified FileDescriptorSet message, length delimited. Does not implicitly {@link google.protobuf.FileDescriptorSet.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.FileDescriptorSet
             * @static
             * @param {google.protobuf.IFileDescriptorSet} message FileDescriptorSet message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FileDescriptorSet.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FileDescriptorSet message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.FileDescriptorSet
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.FileDescriptorSet} FileDescriptorSet
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FileDescriptorSet.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.FileDescriptorSet();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.file && message.file.length))
                            message.file = [];
                        message.file.push($root.google.protobuf.FileDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FileDescriptorSet message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.FileDescriptorSet
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.FileDescriptorSet} FileDescriptorSet
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FileDescriptorSet.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FileDescriptorSet message.
             * @function verify
             * @memberof google.protobuf.FileDescriptorSet
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FileDescriptorSet.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.file != null && message.hasOwnProperty("file")) {
                    if (!Array.isArray(message.file))
                        return "file: array expected";
                    for (let i = 0; i < message.file.length; ++i) {
                        let error = $root.google.protobuf.FileDescriptorProto.verify(message.file[i]);
                        if (error)
                            return "file." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a FileDescriptorSet message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.FileDescriptorSet
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.FileDescriptorSet} FileDescriptorSet
             */
            FileDescriptorSet.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.FileDescriptorSet)
                    return object;
                let message = new $root.google.protobuf.FileDescriptorSet();
                if (object.file) {
                    if (!Array.isArray(object.file))
                        throw TypeError(".google.protobuf.FileDescriptorSet.file: array expected");
                    message.file = [];
                    for (let i = 0; i < object.file.length; ++i) {
                        if (typeof object.file[i] !== "object")
                            throw TypeError(".google.protobuf.FileDescriptorSet.file: object expected");
                        message.file[i] = $root.google.protobuf.FileDescriptorProto.fromObject(object.file[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a FileDescriptorSet message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.FileDescriptorSet
             * @static
             * @param {google.protobuf.FileDescriptorSet} message FileDescriptorSet
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FileDescriptorSet.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.file = [];
                if (message.file && message.file.length) {
                    object.file = [];
                    for (let j = 0; j < message.file.length; ++j)
                        object.file[j] = $root.google.protobuf.FileDescriptorProto.toObject(message.file[j], options);
                }
                return object;
            };

            /**
             * Converts this FileDescriptorSet to JSON.
             * @function toJSON
             * @memberof google.protobuf.FileDescriptorSet
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FileDescriptorSet.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return FileDescriptorSet;
        })();

        protobuf.FileDescriptorProto = (function() {

            /**
             * Properties of a FileDescriptorProto.
             * @memberof google.protobuf
             * @interface IFileDescriptorProto
             * @property {string|null} [name] FileDescriptorProto name
             * @property {string|null} ["package"] FileDescriptorProto package
             * @property {Array.<string>|null} [dependency] FileDescriptorProto dependency
             * @property {Array.<number>|null} [publicDependency] FileDescriptorProto publicDependency
             * @property {Array.<number>|null} [weakDependency] FileDescriptorProto weakDependency
             * @property {Array.<google.protobuf.IDescriptorProto>|null} [messageType] FileDescriptorProto messageType
             * @property {Array.<google.protobuf.IEnumDescriptorProto>|null} [enumType] FileDescriptorProto enumType
             * @property {Array.<google.protobuf.IServiceDescriptorProto>|null} [service] FileDescriptorProto service
             * @property {Array.<google.protobuf.IFieldDescriptorProto>|null} [extension] FileDescriptorProto extension
             * @property {google.protobuf.IFileOptions|null} [options] FileDescriptorProto options
             * @property {google.protobuf.ISourceCodeInfo|null} [sourceCodeInfo] FileDescriptorProto sourceCodeInfo
             * @property {string|null} [syntax] FileDescriptorProto syntax
             */

            /**
             * Constructs a new FileDescriptorProto.
             * @memberof google.protobuf
             * @classdesc Represents a FileDescriptorProto.
             * @implements IFileDescriptorProto
             * @constructor
             * @param {google.protobuf.IFileDescriptorProto=} [properties] Properties to set
             */
            function FileDescriptorProto(properties) {
                this.dependency = [];
                this.publicDependency = [];
                this.weakDependency = [];
                this.messageType = [];
                this.enumType = [];
                this.service = [];
                this.extension = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FileDescriptorProto name.
             * @member {string} name
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.name = "";

            /**
             * FileDescriptorProto package.
             * @member {string} package
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype["package"] = "";

            /**
             * FileDescriptorProto dependency.
             * @member {Array.<string>} dependency
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.dependency = $util.emptyArray;

            /**
             * FileDescriptorProto publicDependency.
             * @member {Array.<number>} publicDependency
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.publicDependency = $util.emptyArray;

            /**
             * FileDescriptorProto weakDependency.
             * @member {Array.<number>} weakDependency
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.weakDependency = $util.emptyArray;

            /**
             * FileDescriptorProto messageType.
             * @member {Array.<google.protobuf.IDescriptorProto>} messageType
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.messageType = $util.emptyArray;

            /**
             * FileDescriptorProto enumType.
             * @member {Array.<google.protobuf.IEnumDescriptorProto>} enumType
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.enumType = $util.emptyArray;

            /**
             * FileDescriptorProto service.
             * @member {Array.<google.protobuf.IServiceDescriptorProto>} service
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.service = $util.emptyArray;

            /**
             * FileDescriptorProto extension.
             * @member {Array.<google.protobuf.IFieldDescriptorProto>} extension
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.extension = $util.emptyArray;

            /**
             * FileDescriptorProto options.
             * @member {google.protobuf.IFileOptions|null|undefined} options
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.options = null;

            /**
             * FileDescriptorProto sourceCodeInfo.
             * @member {google.protobuf.ISourceCodeInfo|null|undefined} sourceCodeInfo
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.sourceCodeInfo = null;

            /**
             * FileDescriptorProto syntax.
             * @member {string} syntax
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             */
            FileDescriptorProto.prototype.syntax = "";

            /**
             * Creates a new FileDescriptorProto instance using the specified properties.
             * @function create
             * @memberof google.protobuf.FileDescriptorProto
             * @static
             * @param {google.protobuf.IFileDescriptorProto=} [properties] Properties to set
             * @returns {google.protobuf.FileDescriptorProto} FileDescriptorProto instance
             */
            FileDescriptorProto.create = function create(properties) {
                return new FileDescriptorProto(properties);
            };

            /**
             * Encodes the specified FileDescriptorProto message. Does not implicitly {@link google.protobuf.FileDescriptorProto.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.FileDescriptorProto
             * @static
             * @param {google.protobuf.IFileDescriptorProto} message FileDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FileDescriptorProto.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message["package"] != null && Object.hasOwnProperty.call(message, "package"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message["package"]);
                if (message.dependency != null && message.dependency.length)
                    for (let i = 0; i < message.dependency.length; ++i)
                        writer.uint32(/* id 3, wireType 2 =*/26).string(message.dependency[i]);
                if (message.messageType != null && message.messageType.length)
                    for (let i = 0; i < message.messageType.length; ++i)
                        $root.google.protobuf.DescriptorProto.encode(message.messageType[i], writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.enumType != null && message.enumType.length)
                    for (let i = 0; i < message.enumType.length; ++i)
                        $root.google.protobuf.EnumDescriptorProto.encode(message.enumType[i], writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                if (message.service != null && message.service.length)
                    for (let i = 0; i < message.service.length; ++i)
                        $root.google.protobuf.ServiceDescriptorProto.encode(message.service[i], writer.uint32(/* id 6, wireType 2 =*/50).fork()).ldelim();
                if (message.extension != null && message.extension.length)
                    for (let i = 0; i < message.extension.length; ++i)
                        $root.google.protobuf.FieldDescriptorProto.encode(message.extension[i], writer.uint32(/* id 7, wireType 2 =*/58).fork()).ldelim();
                if (message.options != null && Object.hasOwnProperty.call(message, "options"))
                    $root.google.protobuf.FileOptions.encode(message.options, writer.uint32(/* id 8, wireType 2 =*/66).fork()).ldelim();
                if (message.sourceCodeInfo != null && Object.hasOwnProperty.call(message, "sourceCodeInfo"))
                    $root.google.protobuf.SourceCodeInfo.encode(message.sourceCodeInfo, writer.uint32(/* id 9, wireType 2 =*/74).fork()).ldelim();
                if (message.publicDependency != null && message.publicDependency.length)
                    for (let i = 0; i < message.publicDependency.length; ++i)
                        writer.uint32(/* id 10, wireType 0 =*/80).int32(message.publicDependency[i]);
                if (message.weakDependency != null && message.weakDependency.length)
                    for (let i = 0; i < message.weakDependency.length; ++i)
                        writer.uint32(/* id 11, wireType 0 =*/88).int32(message.weakDependency[i]);
                if (message.syntax != null && Object.hasOwnProperty.call(message, "syntax"))
                    writer.uint32(/* id 12, wireType 2 =*/98).string(message.syntax);
                return writer;
            };

            /**
             * Encodes the specified FileDescriptorProto message, length delimited. Does not implicitly {@link google.protobuf.FileDescriptorProto.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.FileDescriptorProto
             * @static
             * @param {google.protobuf.IFileDescriptorProto} message FileDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FileDescriptorProto.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FileDescriptorProto message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.FileDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.FileDescriptorProto} FileDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FileDescriptorProto.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.FileDescriptorProto();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.name = reader.string();
                        break;
                    case 2:
                        message["package"] = reader.string();
                        break;
                    case 3:
                        if (!(message.dependency && message.dependency.length))
                            message.dependency = [];
                        message.dependency.push(reader.string());
                        break;
                    case 10:
                        if (!(message.publicDependency && message.publicDependency.length))
                            message.publicDependency = [];
                        if ((tag & 7) === 2) {
                            let end2 = reader.uint32() + reader.pos;
                            while (reader.pos < end2)
                                message.publicDependency.push(reader.int32());
                        } else
                            message.publicDependency.push(reader.int32());
                        break;
                    case 11:
                        if (!(message.weakDependency && message.weakDependency.length))
                            message.weakDependency = [];
                        if ((tag & 7) === 2) {
                            let end2 = reader.uint32() + reader.pos;
                            while (reader.pos < end2)
                                message.weakDependency.push(reader.int32());
                        } else
                            message.weakDependency.push(reader.int32());
                        break;
                    case 4:
                        if (!(message.messageType && message.messageType.length))
                            message.messageType = [];
                        message.messageType.push($root.google.protobuf.DescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 5:
                        if (!(message.enumType && message.enumType.length))
                            message.enumType = [];
                        message.enumType.push($root.google.protobuf.EnumDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 6:
                        if (!(message.service && message.service.length))
                            message.service = [];
                        message.service.push($root.google.protobuf.ServiceDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 7:
                        if (!(message.extension && message.extension.length))
                            message.extension = [];
                        message.extension.push($root.google.protobuf.FieldDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 8:
                        message.options = $root.google.protobuf.FileOptions.decode(reader, reader.uint32());
                        break;
                    case 9:
                        message.sourceCodeInfo = $root.google.protobuf.SourceCodeInfo.decode(reader, reader.uint32());
                        break;
                    case 12:
                        message.syntax = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FileDescriptorProto message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.FileDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.FileDescriptorProto} FileDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FileDescriptorProto.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FileDescriptorProto message.
             * @function verify
             * @memberof google.protobuf.FileDescriptorProto
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FileDescriptorProto.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.name != null && message.hasOwnProperty("name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message["package"] != null && message.hasOwnProperty("package"))
                    if (!$util.isString(message["package"]))
                        return "package: string expected";
                if (message.dependency != null && message.hasOwnProperty("dependency")) {
                    if (!Array.isArray(message.dependency))
                        return "dependency: array expected";
                    for (let i = 0; i < message.dependency.length; ++i)
                        if (!$util.isString(message.dependency[i]))
                            return "dependency: string[] expected";
                }
                if (message.publicDependency != null && message.hasOwnProperty("publicDependency")) {
                    if (!Array.isArray(message.publicDependency))
                        return "publicDependency: array expected";
                    for (let i = 0; i < message.publicDependency.length; ++i)
                        if (!$util.isInteger(message.publicDependency[i]))
                            return "publicDependency: integer[] expected";
                }
                if (message.weakDependency != null && message.hasOwnProperty("weakDependency")) {
                    if (!Array.isArray(message.weakDependency))
                        return "weakDependency: array expected";
                    for (let i = 0; i < message.weakDependency.length; ++i)
                        if (!$util.isInteger(message.weakDependency[i]))
                            return "weakDependency: integer[] expected";
                }
                if (message.messageType != null && message.hasOwnProperty("messageType")) {
                    if (!Array.isArray(message.messageType))
                        return "messageType: array expected";
                    for (let i = 0; i < message.messageType.length; ++i) {
                        let error = $root.google.protobuf.DescriptorProto.verify(message.messageType[i]);
                        if (error)
                            return "messageType." + error;
                    }
                }
                if (message.enumType != null && message.hasOwnProperty("enumType")) {
                    if (!Array.isArray(message.enumType))
                        return "enumType: array expected";
                    for (let i = 0; i < message.enumType.length; ++i) {
                        let error = $root.google.protobuf.EnumDescriptorProto.verify(message.enumType[i]);
                        if (error)
                            return "enumType." + error;
                    }
                }
                if (message.service != null && message.hasOwnProperty("service")) {
                    if (!Array.isArray(message.service))
                        return "service: array expected";
                    for (let i = 0; i < message.service.length; ++i) {
                        let error = $root.google.protobuf.ServiceDescriptorProto.verify(message.service[i]);
                        if (error)
                            return "service." + error;
                    }
                }
                if (message.extension != null && message.hasOwnProperty("extension")) {
                    if (!Array.isArray(message.extension))
                        return "extension: array expected";
                    for (let i = 0; i < message.extension.length; ++i) {
                        let error = $root.google.protobuf.FieldDescriptorProto.verify(message.extension[i]);
                        if (error)
                            return "extension." + error;
                    }
                }
                if (message.options != null && message.hasOwnProperty("options")) {
                    let error = $root.google.protobuf.FileOptions.verify(message.options);
                    if (error)
                        return "options." + error;
                }
                if (message.sourceCodeInfo != null && message.hasOwnProperty("sourceCodeInfo")) {
                    let error = $root.google.protobuf.SourceCodeInfo.verify(message.sourceCodeInfo);
                    if (error)
                        return "sourceCodeInfo." + error;
                }
                if (message.syntax != null && message.hasOwnProperty("syntax"))
                    if (!$util.isString(message.syntax))
                        return "syntax: string expected";
                return null;
            };

            /**
             * Creates a FileDescriptorProto message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.FileDescriptorProto
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.FileDescriptorProto} FileDescriptorProto
             */
            FileDescriptorProto.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.FileDescriptorProto)
                    return object;
                let message = new $root.google.protobuf.FileDescriptorProto();
                if (object.name != null)
                    message.name = String(object.name);
                if (object["package"] != null)
                    message["package"] = String(object["package"]);
                if (object.dependency) {
                    if (!Array.isArray(object.dependency))
                        throw TypeError(".google.protobuf.FileDescriptorProto.dependency: array expected");
                    message.dependency = [];
                    for (let i = 0; i < object.dependency.length; ++i)
                        message.dependency[i] = String(object.dependency[i]);
                }
                if (object.publicDependency) {
                    if (!Array.isArray(object.publicDependency))
                        throw TypeError(".google.protobuf.FileDescriptorProto.publicDependency: array expected");
                    message.publicDependency = [];
                    for (let i = 0; i < object.publicDependency.length; ++i)
                        message.publicDependency[i] = object.publicDependency[i] | 0;
                }
                if (object.weakDependency) {
                    if (!Array.isArray(object.weakDependency))
                        throw TypeError(".google.protobuf.FileDescriptorProto.weakDependency: array expected");
                    message.weakDependency = [];
                    for (let i = 0; i < object.weakDependency.length; ++i)
                        message.weakDependency[i] = object.weakDependency[i] | 0;
                }
                if (object.messageType) {
                    if (!Array.isArray(object.messageType))
                        throw TypeError(".google.protobuf.FileDescriptorProto.messageType: array expected");
                    message.messageType = [];
                    for (let i = 0; i < object.messageType.length; ++i) {
                        if (typeof object.messageType[i] !== "object")
                            throw TypeError(".google.protobuf.FileDescriptorProto.messageType: object expected");
                        message.messageType[i] = $root.google.protobuf.DescriptorProto.fromObject(object.messageType[i]);
                    }
                }
                if (object.enumType) {
                    if (!Array.isArray(object.enumType))
                        throw TypeError(".google.protobuf.FileDescriptorProto.enumType: array expected");
                    message.enumType = [];
                    for (let i = 0; i < object.enumType.length; ++i) {
                        if (typeof object.enumType[i] !== "object")
                            throw TypeError(".google.protobuf.FileDescriptorProto.enumType: object expected");
                        message.enumType[i] = $root.google.protobuf.EnumDescriptorProto.fromObject(object.enumType[i]);
                    }
                }
                if (object.service) {
                    if (!Array.isArray(object.service))
                        throw TypeError(".google.protobuf.FileDescriptorProto.service: array expected");
                    message.service = [];
                    for (let i = 0; i < object.service.length; ++i) {
                        if (typeof object.service[i] !== "object")
                            throw TypeError(".google.protobuf.FileDescriptorProto.service: object expected");
                        message.service[i] = $root.google.protobuf.ServiceDescriptorProto.fromObject(object.service[i]);
                    }
                }
                if (object.extension) {
                    if (!Array.isArray(object.extension))
                        throw TypeError(".google.protobuf.FileDescriptorProto.extension: array expected");
                    message.extension = [];
                    for (let i = 0; i < object.extension.length; ++i) {
                        if (typeof object.extension[i] !== "object")
                            throw TypeError(".google.protobuf.FileDescriptorProto.extension: object expected");
                        message.extension[i] = $root.google.protobuf.FieldDescriptorProto.fromObject(object.extension[i]);
                    }
                }
                if (object.options != null) {
                    if (typeof object.options !== "object")
                        throw TypeError(".google.protobuf.FileDescriptorProto.options: object expected");
                    message.options = $root.google.protobuf.FileOptions.fromObject(object.options);
                }
                if (object.sourceCodeInfo != null) {
                    if (typeof object.sourceCodeInfo !== "object")
                        throw TypeError(".google.protobuf.FileDescriptorProto.sourceCodeInfo: object expected");
                    message.sourceCodeInfo = $root.google.protobuf.SourceCodeInfo.fromObject(object.sourceCodeInfo);
                }
                if (object.syntax != null)
                    message.syntax = String(object.syntax);
                return message;
            };

            /**
             * Creates a plain object from a FileDescriptorProto message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.FileDescriptorProto
             * @static
             * @param {google.protobuf.FileDescriptorProto} message FileDescriptorProto
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FileDescriptorProto.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults) {
                    object.dependency = [];
                    object.messageType = [];
                    object.enumType = [];
                    object.service = [];
                    object.extension = [];
                    object.publicDependency = [];
                    object.weakDependency = [];
                }
                if (options.defaults) {
                    object.name = "";
                    object["package"] = "";
                    object.options = null;
                    object.sourceCodeInfo = null;
                    object.syntax = "";
                }
                if (message.name != null && message.hasOwnProperty("name"))
                    object.name = message.name;
                if (message["package"] != null && message.hasOwnProperty("package"))
                    object["package"] = message["package"];
                if (message.dependency && message.dependency.length) {
                    object.dependency = [];
                    for (let j = 0; j < message.dependency.length; ++j)
                        object.dependency[j] = message.dependency[j];
                }
                if (message.messageType && message.messageType.length) {
                    object.messageType = [];
                    for (let j = 0; j < message.messageType.length; ++j)
                        object.messageType[j] = $root.google.protobuf.DescriptorProto.toObject(message.messageType[j], options);
                }
                if (message.enumType && message.enumType.length) {
                    object.enumType = [];
                    for (let j = 0; j < message.enumType.length; ++j)
                        object.enumType[j] = $root.google.protobuf.EnumDescriptorProto.toObject(message.enumType[j], options);
                }
                if (message.service && message.service.length) {
                    object.service = [];
                    for (let j = 0; j < message.service.length; ++j)
                        object.service[j] = $root.google.protobuf.ServiceDescriptorProto.toObject(message.service[j], options);
                }
                if (message.extension && message.extension.length) {
                    object.extension = [];
                    for (let j = 0; j < message.extension.length; ++j)
                        object.extension[j] = $root.google.protobuf.FieldDescriptorProto.toObject(message.extension[j], options);
                }
                if (message.options != null && message.hasOwnProperty("options"))
                    object.options = $root.google.protobuf.FileOptions.toObject(message.options, options);
                if (message.sourceCodeInfo != null && message.hasOwnProperty("sourceCodeInfo"))
                    object.sourceCodeInfo = $root.google.protobuf.SourceCodeInfo.toObject(message.sourceCodeInfo, options);
                if (message.publicDependency && message.publicDependency.length) {
                    object.publicDependency = [];
                    for (let j = 0; j < message.publicDependency.length; ++j)
                        object.publicDependency[j] = message.publicDependency[j];
                }
                if (message.weakDependency && message.weakDependency.length) {
                    object.weakDependency = [];
                    for (let j = 0; j < message.weakDependency.length; ++j)
                        object.weakDependency[j] = message.weakDependency[j];
                }
                if (message.syntax != null && message.hasOwnProperty("syntax"))
                    object.syntax = message.syntax;
                return object;
            };

            /**
             * Converts this FileDescriptorProto to JSON.
             * @function toJSON
             * @memberof google.protobuf.FileDescriptorProto
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FileDescriptorProto.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return FileDescriptorProto;
        })();

        protobuf.DescriptorProto = (function() {

            /**
             * Properties of a DescriptorProto.
             * @memberof google.protobuf
             * @interface IDescriptorProto
             * @property {string|null} [name] DescriptorProto name
             * @property {Array.<google.protobuf.IFieldDescriptorProto>|null} [field] DescriptorProto field
             * @property {Array.<google.protobuf.IFieldDescriptorProto>|null} [extension] DescriptorProto extension
             * @property {Array.<google.protobuf.IDescriptorProto>|null} [nestedType] DescriptorProto nestedType
             * @property {Array.<google.protobuf.IEnumDescriptorProto>|null} [enumType] DescriptorProto enumType
             * @property {Array.<google.protobuf.DescriptorProto.IExtensionRange>|null} [extensionRange] DescriptorProto extensionRange
             * @property {Array.<google.protobuf.IOneofDescriptorProto>|null} [oneofDecl] DescriptorProto oneofDecl
             * @property {google.protobuf.IMessageOptions|null} [options] DescriptorProto options
             * @property {Array.<google.protobuf.DescriptorProto.IReservedRange>|null} [reservedRange] DescriptorProto reservedRange
             * @property {Array.<string>|null} [reservedName] DescriptorProto reservedName
             */

            /**
             * Constructs a new DescriptorProto.
             * @memberof google.protobuf
             * @classdesc Represents a DescriptorProto.
             * @implements IDescriptorProto
             * @constructor
             * @param {google.protobuf.IDescriptorProto=} [properties] Properties to set
             */
            function DescriptorProto(properties) {
                this.field = [];
                this.extension = [];
                this.nestedType = [];
                this.enumType = [];
                this.extensionRange = [];
                this.oneofDecl = [];
                this.reservedRange = [];
                this.reservedName = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * DescriptorProto name.
             * @member {string} name
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.name = "";

            /**
             * DescriptorProto field.
             * @member {Array.<google.protobuf.IFieldDescriptorProto>} field
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.field = $util.emptyArray;

            /**
             * DescriptorProto extension.
             * @member {Array.<google.protobuf.IFieldDescriptorProto>} extension
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.extension = $util.emptyArray;

            /**
             * DescriptorProto nestedType.
             * @member {Array.<google.protobuf.IDescriptorProto>} nestedType
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.nestedType = $util.emptyArray;

            /**
             * DescriptorProto enumType.
             * @member {Array.<google.protobuf.IEnumDescriptorProto>} enumType
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.enumType = $util.emptyArray;

            /**
             * DescriptorProto extensionRange.
             * @member {Array.<google.protobuf.DescriptorProto.IExtensionRange>} extensionRange
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.extensionRange = $util.emptyArray;

            /**
             * DescriptorProto oneofDecl.
             * @member {Array.<google.protobuf.IOneofDescriptorProto>} oneofDecl
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.oneofDecl = $util.emptyArray;

            /**
             * DescriptorProto options.
             * @member {google.protobuf.IMessageOptions|null|undefined} options
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.options = null;

            /**
             * DescriptorProto reservedRange.
             * @member {Array.<google.protobuf.DescriptorProto.IReservedRange>} reservedRange
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.reservedRange = $util.emptyArray;

            /**
             * DescriptorProto reservedName.
             * @member {Array.<string>} reservedName
             * @memberof google.protobuf.DescriptorProto
             * @instance
             */
            DescriptorProto.prototype.reservedName = $util.emptyArray;

            /**
             * Creates a new DescriptorProto instance using the specified properties.
             * @function create
             * @memberof google.protobuf.DescriptorProto
             * @static
             * @param {google.protobuf.IDescriptorProto=} [properties] Properties to set
             * @returns {google.protobuf.DescriptorProto} DescriptorProto instance
             */
            DescriptorProto.create = function create(properties) {
                return new DescriptorProto(properties);
            };

            /**
             * Encodes the specified DescriptorProto message. Does not implicitly {@link google.protobuf.DescriptorProto.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.DescriptorProto
             * @static
             * @param {google.protobuf.IDescriptorProto} message DescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DescriptorProto.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.field != null && message.field.length)
                    for (let i = 0; i < message.field.length; ++i)
                        $root.google.protobuf.FieldDescriptorProto.encode(message.field[i], writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.nestedType != null && message.nestedType.length)
                    for (let i = 0; i < message.nestedType.length; ++i)
                        $root.google.protobuf.DescriptorProto.encode(message.nestedType[i], writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.enumType != null && message.enumType.length)
                    for (let i = 0; i < message.enumType.length; ++i)
                        $root.google.protobuf.EnumDescriptorProto.encode(message.enumType[i], writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.extensionRange != null && message.extensionRange.length)
                    for (let i = 0; i < message.extensionRange.length; ++i)
                        $root.google.protobuf.DescriptorProto.ExtensionRange.encode(message.extensionRange[i], writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                if (message.extension != null && message.extension.length)
                    for (let i = 0; i < message.extension.length; ++i)
                        $root.google.protobuf.FieldDescriptorProto.encode(message.extension[i], writer.uint32(/* id 6, wireType 2 =*/50).fork()).ldelim();
                if (message.options != null && Object.hasOwnProperty.call(message, "options"))
                    $root.google.protobuf.MessageOptions.encode(message.options, writer.uint32(/* id 7, wireType 2 =*/58).fork()).ldelim();
                if (message.oneofDecl != null && message.oneofDecl.length)
                    for (let i = 0; i < message.oneofDecl.length; ++i)
                        $root.google.protobuf.OneofDescriptorProto.encode(message.oneofDecl[i], writer.uint32(/* id 8, wireType 2 =*/66).fork()).ldelim();
                if (message.reservedRange != null && message.reservedRange.length)
                    for (let i = 0; i < message.reservedRange.length; ++i)
                        $root.google.protobuf.DescriptorProto.ReservedRange.encode(message.reservedRange[i], writer.uint32(/* id 9, wireType 2 =*/74).fork()).ldelim();
                if (message.reservedName != null && message.reservedName.length)
                    for (let i = 0; i < message.reservedName.length; ++i)
                        writer.uint32(/* id 10, wireType 2 =*/82).string(message.reservedName[i]);
                return writer;
            };

            /**
             * Encodes the specified DescriptorProto message, length delimited. Does not implicitly {@link google.protobuf.DescriptorProto.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.DescriptorProto
             * @static
             * @param {google.protobuf.IDescriptorProto} message DescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DescriptorProto.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a DescriptorProto message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.DescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.DescriptorProto} DescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DescriptorProto.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.DescriptorProto();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.name = reader.string();
                        break;
                    case 2:
                        if (!(message.field && message.field.length))
                            message.field = [];
                        message.field.push($root.google.protobuf.FieldDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 6:
                        if (!(message.extension && message.extension.length))
                            message.extension = [];
                        message.extension.push($root.google.protobuf.FieldDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 3:
                        if (!(message.nestedType && message.nestedType.length))
                            message.nestedType = [];
                        message.nestedType.push($root.google.protobuf.DescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 4:
                        if (!(message.enumType && message.enumType.length))
                            message.enumType = [];
                        message.enumType.push($root.google.protobuf.EnumDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 5:
                        if (!(message.extensionRange && message.extensionRange.length))
                            message.extensionRange = [];
                        message.extensionRange.push($root.google.protobuf.DescriptorProto.ExtensionRange.decode(reader, reader.uint32()));
                        break;
                    case 8:
                        if (!(message.oneofDecl && message.oneofDecl.length))
                            message.oneofDecl = [];
                        message.oneofDecl.push($root.google.protobuf.OneofDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 7:
                        message.options = $root.google.protobuf.MessageOptions.decode(reader, reader.uint32());
                        break;
                    case 9:
                        if (!(message.reservedRange && message.reservedRange.length))
                            message.reservedRange = [];
                        message.reservedRange.push($root.google.protobuf.DescriptorProto.ReservedRange.decode(reader, reader.uint32()));
                        break;
                    case 10:
                        if (!(message.reservedName && message.reservedName.length))
                            message.reservedName = [];
                        message.reservedName.push(reader.string());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a DescriptorProto message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.DescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.DescriptorProto} DescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DescriptorProto.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a DescriptorProto message.
             * @function verify
             * @memberof google.protobuf.DescriptorProto
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            DescriptorProto.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.name != null && message.hasOwnProperty("name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.field != null && message.hasOwnProperty("field")) {
                    if (!Array.isArray(message.field))
                        return "field: array expected";
                    for (let i = 0; i < message.field.length; ++i) {
                        let error = $root.google.protobuf.FieldDescriptorProto.verify(message.field[i]);
                        if (error)
                            return "field." + error;
                    }
                }
                if (message.extension != null && message.hasOwnProperty("extension")) {
                    if (!Array.isArray(message.extension))
                        return "extension: array expected";
                    for (let i = 0; i < message.extension.length; ++i) {
                        let error = $root.google.protobuf.FieldDescriptorProto.verify(message.extension[i]);
                        if (error)
                            return "extension." + error;
                    }
                }
                if (message.nestedType != null && message.hasOwnProperty("nestedType")) {
                    if (!Array.isArray(message.nestedType))
                        return "nestedType: array expected";
                    for (let i = 0; i < message.nestedType.length; ++i) {
                        let error = $root.google.protobuf.DescriptorProto.verify(message.nestedType[i]);
                        if (error)
                            return "nestedType." + error;
                    }
                }
                if (message.enumType != null && message.hasOwnProperty("enumType")) {
                    if (!Array.isArray(message.enumType))
                        return "enumType: array expected";
                    for (let i = 0; i < message.enumType.length; ++i) {
                        let error = $root.google.protobuf.EnumDescriptorProto.verify(message.enumType[i]);
                        if (error)
                            return "enumType." + error;
                    }
                }
                if (message.extensionRange != null && message.hasOwnProperty("extensionRange")) {
                    if (!Array.isArray(message.extensionRange))
                        return "extensionRange: array expected";
                    for (let i = 0; i < message.extensionRange.length; ++i) {
                        let error = $root.google.protobuf.DescriptorProto.ExtensionRange.verify(message.extensionRange[i]);
                        if (error)
                            return "extensionRange." + error;
                    }
                }
                if (message.oneofDecl != null && message.hasOwnProperty("oneofDecl")) {
                    if (!Array.isArray(message.oneofDecl))
                        return "oneofDecl: array expected";
                    for (let i = 0; i < message.oneofDecl.length; ++i) {
                        let error = $root.google.protobuf.OneofDescriptorProto.verify(message.oneofDecl[i]);
                        if (error)
                            return "oneofDecl." + error;
                    }
                }
                if (message.options != null && message.hasOwnProperty("options")) {
                    let error = $root.google.protobuf.MessageOptions.verify(message.options);
                    if (error)
                        return "options." + error;
                }
                if (message.reservedRange != null && message.hasOwnProperty("reservedRange")) {
                    if (!Array.isArray(message.reservedRange))
                        return "reservedRange: array expected";
                    for (let i = 0; i < message.reservedRange.length; ++i) {
                        let error = $root.google.protobuf.DescriptorProto.ReservedRange.verify(message.reservedRange[i]);
                        if (error)
                            return "reservedRange." + error;
                    }
                }
                if (message.reservedName != null && message.hasOwnProperty("reservedName")) {
                    if (!Array.isArray(message.reservedName))
                        return "reservedName: array expected";
                    for (let i = 0; i < message.reservedName.length; ++i)
                        if (!$util.isString(message.reservedName[i]))
                            return "reservedName: string[] expected";
                }
                return null;
            };

            /**
             * Creates a DescriptorProto message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.DescriptorProto
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.DescriptorProto} DescriptorProto
             */
            DescriptorProto.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.DescriptorProto)
                    return object;
                let message = new $root.google.protobuf.DescriptorProto();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.field) {
                    if (!Array.isArray(object.field))
                        throw TypeError(".google.protobuf.DescriptorProto.field: array expected");
                    message.field = [];
                    for (let i = 0; i < object.field.length; ++i) {
                        if (typeof object.field[i] !== "object")
                            throw TypeError(".google.protobuf.DescriptorProto.field: object expected");
                        message.field[i] = $root.google.protobuf.FieldDescriptorProto.fromObject(object.field[i]);
                    }
                }
                if (object.extension) {
                    if (!Array.isArray(object.extension))
                        throw TypeError(".google.protobuf.DescriptorProto.extension: array expected");
                    message.extension = [];
                    for (let i = 0; i < object.extension.length; ++i) {
                        if (typeof object.extension[i] !== "object")
                            throw TypeError(".google.protobuf.DescriptorProto.extension: object expected");
                        message.extension[i] = $root.google.protobuf.FieldDescriptorProto.fromObject(object.extension[i]);
                    }
                }
                if (object.nestedType) {
                    if (!Array.isArray(object.nestedType))
                        throw TypeError(".google.protobuf.DescriptorProto.nestedType: array expected");
                    message.nestedType = [];
                    for (let i = 0; i < object.nestedType.length; ++i) {
                        if (typeof object.nestedType[i] !== "object")
                            throw TypeError(".google.protobuf.DescriptorProto.nestedType: object expected");
                        message.nestedType[i] = $root.google.protobuf.DescriptorProto.fromObject(object.nestedType[i]);
                    }
                }
                if (object.enumType) {
                    if (!Array.isArray(object.enumType))
                        throw TypeError(".google.protobuf.DescriptorProto.enumType: array expected");
                    message.enumType = [];
                    for (let i = 0; i < object.enumType.length; ++i) {
                        if (typeof object.enumType[i] !== "object")
                            throw TypeError(".google.protobuf.DescriptorProto.enumType: object expected");
                        message.enumType[i] = $root.google.protobuf.EnumDescriptorProto.fromObject(object.enumType[i]);
                    }
                }
                if (object.extensionRange) {
                    if (!Array.isArray(object.extensionRange))
                        throw TypeError(".google.protobuf.DescriptorProto.extensionRange: array expected");
                    message.extensionRange = [];
                    for (let i = 0; i < object.extensionRange.length; ++i) {
                        if (typeof object.extensionRange[i] !== "object")
                            throw TypeError(".google.protobuf.DescriptorProto.extensionRange: object expected");
                        message.extensionRange[i] = $root.google.protobuf.DescriptorProto.ExtensionRange.fromObject(object.extensionRange[i]);
                    }
                }
                if (object.oneofDecl) {
                    if (!Array.isArray(object.oneofDecl))
                        throw TypeError(".google.protobuf.DescriptorProto.oneofDecl: array expected");
                    message.oneofDecl = [];
                    for (let i = 0; i < object.oneofDecl.length; ++i) {
                        if (typeof object.oneofDecl[i] !== "object")
                            throw TypeError(".google.protobuf.DescriptorProto.oneofDecl: object expected");
                        message.oneofDecl[i] = $root.google.protobuf.OneofDescriptorProto.fromObject(object.oneofDecl[i]);
                    }
                }
                if (object.options != null) {
                    if (typeof object.options !== "object")
                        throw TypeError(".google.protobuf.DescriptorProto.options: object expected");
                    message.options = $root.google.protobuf.MessageOptions.fromObject(object.options);
                }
                if (object.reservedRange) {
                    if (!Array.isArray(object.reservedRange))
                        throw TypeError(".google.protobuf.DescriptorProto.reservedRange: array expected");
                    message.reservedRange = [];
                    for (let i = 0; i < object.reservedRange.length; ++i) {
                        if (typeof object.reservedRange[i] !== "object")
                            throw TypeError(".google.protobuf.DescriptorProto.reservedRange: object expected");
                        message.reservedRange[i] = $root.google.protobuf.DescriptorProto.ReservedRange.fromObject(object.reservedRange[i]);
                    }
                }
                if (object.reservedName) {
                    if (!Array.isArray(object.reservedName))
                        throw TypeError(".google.protobuf.DescriptorProto.reservedName: array expected");
                    message.reservedName = [];
                    for (let i = 0; i < object.reservedName.length; ++i)
                        message.reservedName[i] = String(object.reservedName[i]);
                }
                return message;
            };

            /**
             * Creates a plain object from a DescriptorProto message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.DescriptorProto
             * @static
             * @param {google.protobuf.DescriptorProto} message DescriptorProto
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            DescriptorProto.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults) {
                    object.field = [];
                    object.nestedType = [];
                    object.enumType = [];
                    object.extensionRange = [];
                    object.extension = [];
                    object.oneofDecl = [];
                    object.reservedRange = [];
                    object.reservedName = [];
                }
                if (options.defaults) {
                    object.name = "";
                    object.options = null;
                }
                if (message.name != null && message.hasOwnProperty("name"))
                    object.name = message.name;
                if (message.field && message.field.length) {
                    object.field = [];
                    for (let j = 0; j < message.field.length; ++j)
                        object.field[j] = $root.google.protobuf.FieldDescriptorProto.toObject(message.field[j], options);
                }
                if (message.nestedType && message.nestedType.length) {
                    object.nestedType = [];
                    for (let j = 0; j < message.nestedType.length; ++j)
                        object.nestedType[j] = $root.google.protobuf.DescriptorProto.toObject(message.nestedType[j], options);
                }
                if (message.enumType && message.enumType.length) {
                    object.enumType = [];
                    for (let j = 0; j < message.enumType.length; ++j)
                        object.enumType[j] = $root.google.protobuf.EnumDescriptorProto.toObject(message.enumType[j], options);
                }
                if (message.extensionRange && message.extensionRange.length) {
                    object.extensionRange = [];
                    for (let j = 0; j < message.extensionRange.length; ++j)
                        object.extensionRange[j] = $root.google.protobuf.DescriptorProto.ExtensionRange.toObject(message.extensionRange[j], options);
                }
                if (message.extension && message.extension.length) {
                    object.extension = [];
                    for (let j = 0; j < message.extension.length; ++j)
                        object.extension[j] = $root.google.protobuf.FieldDescriptorProto.toObject(message.extension[j], options);
                }
                if (message.options != null && message.hasOwnProperty("options"))
                    object.options = $root.google.protobuf.MessageOptions.toObject(message.options, options);
                if (message.oneofDecl && message.oneofDecl.length) {
                    object.oneofDecl = [];
                    for (let j = 0; j < message.oneofDecl.length; ++j)
                        object.oneofDecl[j] = $root.google.protobuf.OneofDescriptorProto.toObject(message.oneofDecl[j], options);
                }
                if (message.reservedRange && message.reservedRange.length) {
                    object.reservedRange = [];
                    for (let j = 0; j < message.reservedRange.length; ++j)
                        object.reservedRange[j] = $root.google.protobuf.DescriptorProto.ReservedRange.toObject(message.reservedRange[j], options);
                }
                if (message.reservedName && message.reservedName.length) {
                    object.reservedName = [];
                    for (let j = 0; j < message.reservedName.length; ++j)
                        object.reservedName[j] = message.reservedName[j];
                }
                return object;
            };

            /**
             * Converts this DescriptorProto to JSON.
             * @function toJSON
             * @memberof google.protobuf.DescriptorProto
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            DescriptorProto.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            DescriptorProto.ExtensionRange = (function() {

                /**
                 * Properties of an ExtensionRange.
                 * @memberof google.protobuf.DescriptorProto
                 * @interface IExtensionRange
                 * @property {number|null} [start] ExtensionRange start
                 * @property {number|null} [end] ExtensionRange end
                 */

                /**
                 * Constructs a new ExtensionRange.
                 * @memberof google.protobuf.DescriptorProto
                 * @classdesc Represents an ExtensionRange.
                 * @implements IExtensionRange
                 * @constructor
                 * @param {google.protobuf.DescriptorProto.IExtensionRange=} [properties] Properties to set
                 */
                function ExtensionRange(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * ExtensionRange start.
                 * @member {number} start
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @instance
                 */
                ExtensionRange.prototype.start = 0;

                /**
                 * ExtensionRange end.
                 * @member {number} end
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @instance
                 */
                ExtensionRange.prototype.end = 0;

                /**
                 * Creates a new ExtensionRange instance using the specified properties.
                 * @function create
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @static
                 * @param {google.protobuf.DescriptorProto.IExtensionRange=} [properties] Properties to set
                 * @returns {google.protobuf.DescriptorProto.ExtensionRange} ExtensionRange instance
                 */
                ExtensionRange.create = function create(properties) {
                    return new ExtensionRange(properties);
                };

                /**
                 * Encodes the specified ExtensionRange message. Does not implicitly {@link google.protobuf.DescriptorProto.ExtensionRange.verify|verify} messages.
                 * @function encode
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @static
                 * @param {google.protobuf.DescriptorProto.IExtensionRange} message ExtensionRange message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                ExtensionRange.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.start != null && Object.hasOwnProperty.call(message, "start"))
                        writer.uint32(/* id 1, wireType 0 =*/8).int32(message.start);
                    if (message.end != null && Object.hasOwnProperty.call(message, "end"))
                        writer.uint32(/* id 2, wireType 0 =*/16).int32(message.end);
                    return writer;
                };

                /**
                 * Encodes the specified ExtensionRange message, length delimited. Does not implicitly {@link google.protobuf.DescriptorProto.ExtensionRange.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @static
                 * @param {google.protobuf.DescriptorProto.IExtensionRange} message ExtensionRange message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                ExtensionRange.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes an ExtensionRange message from the specified reader or buffer.
                 * @function decode
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {google.protobuf.DescriptorProto.ExtensionRange} ExtensionRange
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                ExtensionRange.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.DescriptorProto.ExtensionRange();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1:
                            message.start = reader.int32();
                            break;
                        case 2:
                            message.end = reader.int32();
                            break;
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes an ExtensionRange message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {google.protobuf.DescriptorProto.ExtensionRange} ExtensionRange
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                ExtensionRange.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies an ExtensionRange message.
                 * @function verify
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                ExtensionRange.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.start != null && message.hasOwnProperty("start"))
                        if (!$util.isInteger(message.start))
                            return "start: integer expected";
                    if (message.end != null && message.hasOwnProperty("end"))
                        if (!$util.isInteger(message.end))
                            return "end: integer expected";
                    return null;
                };

                /**
                 * Creates an ExtensionRange message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {google.protobuf.DescriptorProto.ExtensionRange} ExtensionRange
                 */
                ExtensionRange.fromObject = function fromObject(object) {
                    if (object instanceof $root.google.protobuf.DescriptorProto.ExtensionRange)
                        return object;
                    let message = new $root.google.protobuf.DescriptorProto.ExtensionRange();
                    if (object.start != null)
                        message.start = object.start | 0;
                    if (object.end != null)
                        message.end = object.end | 0;
                    return message;
                };

                /**
                 * Creates a plain object from an ExtensionRange message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @static
                 * @param {google.protobuf.DescriptorProto.ExtensionRange} message ExtensionRange
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                ExtensionRange.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.start = 0;
                        object.end = 0;
                    }
                    if (message.start != null && message.hasOwnProperty("start"))
                        object.start = message.start;
                    if (message.end != null && message.hasOwnProperty("end"))
                        object.end = message.end;
                    return object;
                };

                /**
                 * Converts this ExtensionRange to JSON.
                 * @function toJSON
                 * @memberof google.protobuf.DescriptorProto.ExtensionRange
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                ExtensionRange.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                return ExtensionRange;
            })();

            DescriptorProto.ReservedRange = (function() {

                /**
                 * Properties of a ReservedRange.
                 * @memberof google.protobuf.DescriptorProto
                 * @interface IReservedRange
                 * @property {number|null} [start] ReservedRange start
                 * @property {number|null} [end] ReservedRange end
                 */

                /**
                 * Constructs a new ReservedRange.
                 * @memberof google.protobuf.DescriptorProto
                 * @classdesc Represents a ReservedRange.
                 * @implements IReservedRange
                 * @constructor
                 * @param {google.protobuf.DescriptorProto.IReservedRange=} [properties] Properties to set
                 */
                function ReservedRange(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * ReservedRange start.
                 * @member {number} start
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @instance
                 */
                ReservedRange.prototype.start = 0;

                /**
                 * ReservedRange end.
                 * @member {number} end
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @instance
                 */
                ReservedRange.prototype.end = 0;

                /**
                 * Creates a new ReservedRange instance using the specified properties.
                 * @function create
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @static
                 * @param {google.protobuf.DescriptorProto.IReservedRange=} [properties] Properties to set
                 * @returns {google.protobuf.DescriptorProto.ReservedRange} ReservedRange instance
                 */
                ReservedRange.create = function create(properties) {
                    return new ReservedRange(properties);
                };

                /**
                 * Encodes the specified ReservedRange message. Does not implicitly {@link google.protobuf.DescriptorProto.ReservedRange.verify|verify} messages.
                 * @function encode
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @static
                 * @param {google.protobuf.DescriptorProto.IReservedRange} message ReservedRange message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                ReservedRange.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.start != null && Object.hasOwnProperty.call(message, "start"))
                        writer.uint32(/* id 1, wireType 0 =*/8).int32(message.start);
                    if (message.end != null && Object.hasOwnProperty.call(message, "end"))
                        writer.uint32(/* id 2, wireType 0 =*/16).int32(message.end);
                    return writer;
                };

                /**
                 * Encodes the specified ReservedRange message, length delimited. Does not implicitly {@link google.protobuf.DescriptorProto.ReservedRange.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @static
                 * @param {google.protobuf.DescriptorProto.IReservedRange} message ReservedRange message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                ReservedRange.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a ReservedRange message from the specified reader or buffer.
                 * @function decode
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {google.protobuf.DescriptorProto.ReservedRange} ReservedRange
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                ReservedRange.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.DescriptorProto.ReservedRange();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1:
                            message.start = reader.int32();
                            break;
                        case 2:
                            message.end = reader.int32();
                            break;
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a ReservedRange message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {google.protobuf.DescriptorProto.ReservedRange} ReservedRange
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                ReservedRange.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a ReservedRange message.
                 * @function verify
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                ReservedRange.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.start != null && message.hasOwnProperty("start"))
                        if (!$util.isInteger(message.start))
                            return "start: integer expected";
                    if (message.end != null && message.hasOwnProperty("end"))
                        if (!$util.isInteger(message.end))
                            return "end: integer expected";
                    return null;
                };

                /**
                 * Creates a ReservedRange message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {google.protobuf.DescriptorProto.ReservedRange} ReservedRange
                 */
                ReservedRange.fromObject = function fromObject(object) {
                    if (object instanceof $root.google.protobuf.DescriptorProto.ReservedRange)
                        return object;
                    let message = new $root.google.protobuf.DescriptorProto.ReservedRange();
                    if (object.start != null)
                        message.start = object.start | 0;
                    if (object.end != null)
                        message.end = object.end | 0;
                    return message;
                };

                /**
                 * Creates a plain object from a ReservedRange message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @static
                 * @param {google.protobuf.DescriptorProto.ReservedRange} message ReservedRange
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                ReservedRange.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.start = 0;
                        object.end = 0;
                    }
                    if (message.start != null && message.hasOwnProperty("start"))
                        object.start = message.start;
                    if (message.end != null && message.hasOwnProperty("end"))
                        object.end = message.end;
                    return object;
                };

                /**
                 * Converts this ReservedRange to JSON.
                 * @function toJSON
                 * @memberof google.protobuf.DescriptorProto.ReservedRange
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                ReservedRange.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                return ReservedRange;
            })();

            return DescriptorProto;
        })();

        protobuf.FieldDescriptorProto = (function() {

            /**
             * Properties of a FieldDescriptorProto.
             * @memberof google.protobuf
             * @interface IFieldDescriptorProto
             * @property {string|null} [name] FieldDescriptorProto name
             * @property {number|null} [number] FieldDescriptorProto number
             * @property {google.protobuf.FieldDescriptorProto.Label|null} [label] FieldDescriptorProto label
             * @property {google.protobuf.FieldDescriptorProto.Type|null} [type] FieldDescriptorProto type
             * @property {string|null} [typeName] FieldDescriptorProto typeName
             * @property {string|null} [extendee] FieldDescriptorProto extendee
             * @property {string|null} [defaultValue] FieldDescriptorProto defaultValue
             * @property {number|null} [oneofIndex] FieldDescriptorProto oneofIndex
             * @property {string|null} [jsonName] FieldDescriptorProto jsonName
             * @property {google.protobuf.IFieldOptions|null} [options] FieldDescriptorProto options
             */

            /**
             * Constructs a new FieldDescriptorProto.
             * @memberof google.protobuf
             * @classdesc Represents a FieldDescriptorProto.
             * @implements IFieldDescriptorProto
             * @constructor
             * @param {google.protobuf.IFieldDescriptorProto=} [properties] Properties to set
             */
            function FieldDescriptorProto(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FieldDescriptorProto name.
             * @member {string} name
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.name = "";

            /**
             * FieldDescriptorProto number.
             * @member {number} number
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.number = 0;

            /**
             * FieldDescriptorProto label.
             * @member {google.protobuf.FieldDescriptorProto.Label} label
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.label = 1;

            /**
             * FieldDescriptorProto type.
             * @member {google.protobuf.FieldDescriptorProto.Type} type
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.type = 1;

            /**
             * FieldDescriptorProto typeName.
             * @member {string} typeName
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.typeName = "";

            /**
             * FieldDescriptorProto extendee.
             * @member {string} extendee
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.extendee = "";

            /**
             * FieldDescriptorProto defaultValue.
             * @member {string} defaultValue
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.defaultValue = "";

            /**
             * FieldDescriptorProto oneofIndex.
             * @member {number} oneofIndex
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.oneofIndex = 0;

            /**
             * FieldDescriptorProto jsonName.
             * @member {string} jsonName
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.jsonName = "";

            /**
             * FieldDescriptorProto options.
             * @member {google.protobuf.IFieldOptions|null|undefined} options
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             */
            FieldDescriptorProto.prototype.options = null;

            /**
             * Creates a new FieldDescriptorProto instance using the specified properties.
             * @function create
             * @memberof google.protobuf.FieldDescriptorProto
             * @static
             * @param {google.protobuf.IFieldDescriptorProto=} [properties] Properties to set
             * @returns {google.protobuf.FieldDescriptorProto} FieldDescriptorProto instance
             */
            FieldDescriptorProto.create = function create(properties) {
                return new FieldDescriptorProto(properties);
            };

            /**
             * Encodes the specified FieldDescriptorProto message. Does not implicitly {@link google.protobuf.FieldDescriptorProto.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.FieldDescriptorProto
             * @static
             * @param {google.protobuf.IFieldDescriptorProto} message FieldDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FieldDescriptorProto.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.extendee != null && Object.hasOwnProperty.call(message, "extendee"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.extendee);
                if (message.number != null && Object.hasOwnProperty.call(message, "number"))
                    writer.uint32(/* id 3, wireType 0 =*/24).int32(message.number);
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    writer.uint32(/* id 4, wireType 0 =*/32).int32(message.label);
                if (message.type != null && Object.hasOwnProperty.call(message, "type"))
                    writer.uint32(/* id 5, wireType 0 =*/40).int32(message.type);
                if (message.typeName != null && Object.hasOwnProperty.call(message, "typeName"))
                    writer.uint32(/* id 6, wireType 2 =*/50).string(message.typeName);
                if (message.defaultValue != null && Object.hasOwnProperty.call(message, "defaultValue"))
                    writer.uint32(/* id 7, wireType 2 =*/58).string(message.defaultValue);
                if (message.options != null && Object.hasOwnProperty.call(message, "options"))
                    $root.google.protobuf.FieldOptions.encode(message.options, writer.uint32(/* id 8, wireType 2 =*/66).fork()).ldelim();
                if (message.oneofIndex != null && Object.hasOwnProperty.call(message, "oneofIndex"))
                    writer.uint32(/* id 9, wireType 0 =*/72).int32(message.oneofIndex);
                if (message.jsonName != null && Object.hasOwnProperty.call(message, "jsonName"))
                    writer.uint32(/* id 10, wireType 2 =*/82).string(message.jsonName);
                return writer;
            };

            /**
             * Encodes the specified FieldDescriptorProto message, length delimited. Does not implicitly {@link google.protobuf.FieldDescriptorProto.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.FieldDescriptorProto
             * @static
             * @param {google.protobuf.IFieldDescriptorProto} message FieldDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FieldDescriptorProto.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FieldDescriptorProto message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.FieldDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.FieldDescriptorProto} FieldDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FieldDescriptorProto.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.FieldDescriptorProto();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.name = reader.string();
                        break;
                    case 3:
                        message.number = reader.int32();
                        break;
                    case 4:
                        message.label = reader.int32();
                        break;
                    case 5:
                        message.type = reader.int32();
                        break;
                    case 6:
                        message.typeName = reader.string();
                        break;
                    case 2:
                        message.extendee = reader.string();
                        break;
                    case 7:
                        message.defaultValue = reader.string();
                        break;
                    case 9:
                        message.oneofIndex = reader.int32();
                        break;
                    case 10:
                        message.jsonName = reader.string();
                        break;
                    case 8:
                        message.options = $root.google.protobuf.FieldOptions.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FieldDescriptorProto message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.FieldDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.FieldDescriptorProto} FieldDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FieldDescriptorProto.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FieldDescriptorProto message.
             * @function verify
             * @memberof google.protobuf.FieldDescriptorProto
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FieldDescriptorProto.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.name != null && message.hasOwnProperty("name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.number != null && message.hasOwnProperty("number"))
                    if (!$util.isInteger(message.number))
                        return "number: integer expected";
                if (message.label != null && message.hasOwnProperty("label"))
                    switch (message.label) {
                    default:
                        return "label: enum value expected";
                    case 1:
                    case 2:
                    case 3:
                        break;
                    }
                if (message.type != null && message.hasOwnProperty("type"))
                    switch (message.type) {
                    default:
                        return "type: enum value expected";
                    case 1:
                    case 2:
                    case 3:
                    case 4:
                    case 5:
                    case 6:
                    case 7:
                    case 8:
                    case 9:
                    case 10:
                    case 11:
                    case 12:
                    case 13:
                    case 14:
                    case 15:
                    case 16:
                    case 17:
                    case 18:
                        break;
                    }
                if (message.typeName != null && message.hasOwnProperty("typeName"))
                    if (!$util.isString(message.typeName))
                        return "typeName: string expected";
                if (message.extendee != null && message.hasOwnProperty("extendee"))
                    if (!$util.isString(message.extendee))
                        return "extendee: string expected";
                if (message.defaultValue != null && message.hasOwnProperty("defaultValue"))
                    if (!$util.isString(message.defaultValue))
                        return "defaultValue: string expected";
                if (message.oneofIndex != null && message.hasOwnProperty("oneofIndex"))
                    if (!$util.isInteger(message.oneofIndex))
                        return "oneofIndex: integer expected";
                if (message.jsonName != null && message.hasOwnProperty("jsonName"))
                    if (!$util.isString(message.jsonName))
                        return "jsonName: string expected";
                if (message.options != null && message.hasOwnProperty("options")) {
                    let error = $root.google.protobuf.FieldOptions.verify(message.options);
                    if (error)
                        return "options." + error;
                }
                return null;
            };

            /**
             * Creates a FieldDescriptorProto message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.FieldDescriptorProto
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.FieldDescriptorProto} FieldDescriptorProto
             */
            FieldDescriptorProto.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.FieldDescriptorProto)
                    return object;
                let message = new $root.google.protobuf.FieldDescriptorProto();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.number != null)
                    message.number = object.number | 0;
                switch (object.label) {
                case "LABEL_OPTIONAL":
                case 1:
                    message.label = 1;
                    break;
                case "LABEL_REQUIRED":
                case 2:
                    message.label = 2;
                    break;
                case "LABEL_REPEATED":
                case 3:
                    message.label = 3;
                    break;
                }
                switch (object.type) {
                case "TYPE_DOUBLE":
                case 1:
                    message.type = 1;
                    break;
                case "TYPE_FLOAT":
                case 2:
                    message.type = 2;
                    break;
                case "TYPE_INT64":
                case 3:
                    message.type = 3;
                    break;
                case "TYPE_UINT64":
                case 4:
                    message.type = 4;
                    break;
                case "TYPE_INT32":
                case 5:
                    message.type = 5;
                    break;
                case "TYPE_FIXED64":
                case 6:
                    message.type = 6;
                    break;
                case "TYPE_FIXED32":
                case 7:
                    message.type = 7;
                    break;
                case "TYPE_BOOL":
                case 8:
                    message.type = 8;
                    break;
                case "TYPE_STRING":
                case 9:
                    message.type = 9;
                    break;
                case "TYPE_GROUP":
                case 10:
                    message.type = 10;
                    break;
                case "TYPE_MESSAGE":
                case 11:
                    message.type = 11;
                    break;
                case "TYPE_BYTES":
                case 12:
                    message.type = 12;
                    break;
                case "TYPE_UINT32":
                case 13:
                    message.type = 13;
                    break;
                case "TYPE_ENUM":
                case 14:
                    message.type = 14;
                    break;
                case "TYPE_SFIXED32":
                case 15:
                    message.type = 15;
                    break;
                case "TYPE_SFIXED64":
                case 16:
                    message.type = 16;
                    break;
                case "TYPE_SINT32":
                case 17:
                    message.type = 17;
                    break;
                case "TYPE_SINT64":
                case 18:
                    message.type = 18;
                    break;
                }
                if (object.typeName != null)
                    message.typeName = String(object.typeName);
                if (object.extendee != null)
                    message.extendee = String(object.extendee);
                if (object.defaultValue != null)
                    message.defaultValue = String(object.defaultValue);
                if (object.oneofIndex != null)
                    message.oneofIndex = object.oneofIndex | 0;
                if (object.jsonName != null)
                    message.jsonName = String(object.jsonName);
                if (object.options != null) {
                    if (typeof object.options !== "object")
                        throw TypeError(".google.protobuf.FieldDescriptorProto.options: object expected");
                    message.options = $root.google.protobuf.FieldOptions.fromObject(object.options);
                }
                return message;
            };

            /**
             * Creates a plain object from a FieldDescriptorProto message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.FieldDescriptorProto
             * @static
             * @param {google.protobuf.FieldDescriptorProto} message FieldDescriptorProto
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FieldDescriptorProto.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.name = "";
                    object.extendee = "";
                    object.number = 0;
                    object.label = options.enums === String ? "LABEL_OPTIONAL" : 1;
                    object.type = options.enums === String ? "TYPE_DOUBLE" : 1;
                    object.typeName = "";
                    object.defaultValue = "";
                    object.options = null;
                    object.oneofIndex = 0;
                    object.jsonName = "";
                }
                if (message.name != null && message.hasOwnProperty("name"))
                    object.name = message.name;
                if (message.extendee != null && message.hasOwnProperty("extendee"))
                    object.extendee = message.extendee;
                if (message.number != null && message.hasOwnProperty("number"))
                    object.number = message.number;
                if (message.label != null && message.hasOwnProperty("label"))
                    object.label = options.enums === String ? $root.google.protobuf.FieldDescriptorProto.Label[message.label] : message.label;
                if (message.type != null && message.hasOwnProperty("type"))
                    object.type = options.enums === String ? $root.google.protobuf.FieldDescriptorProto.Type[message.type] : message.type;
                if (message.typeName != null && message.hasOwnProperty("typeName"))
                    object.typeName = message.typeName;
                if (message.defaultValue != null && message.hasOwnProperty("defaultValue"))
                    object.defaultValue = message.defaultValue;
                if (message.options != null && message.hasOwnProperty("options"))
                    object.options = $root.google.protobuf.FieldOptions.toObject(message.options, options);
                if (message.oneofIndex != null && message.hasOwnProperty("oneofIndex"))
                    object.oneofIndex = message.oneofIndex;
                if (message.jsonName != null && message.hasOwnProperty("jsonName"))
                    object.jsonName = message.jsonName;
                return object;
            };

            /**
             * Converts this FieldDescriptorProto to JSON.
             * @function toJSON
             * @memberof google.protobuf.FieldDescriptorProto
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FieldDescriptorProto.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Type enum.
             * @name google.protobuf.FieldDescriptorProto.Type
             * @enum {number}
             * @property {number} TYPE_DOUBLE=1 TYPE_DOUBLE value
             * @property {number} TYPE_FLOAT=2 TYPE_FLOAT value
             * @property {number} TYPE_INT64=3 TYPE_INT64 value
             * @property {number} TYPE_UINT64=4 TYPE_UINT64 value
             * @property {number} TYPE_INT32=5 TYPE_INT32 value
             * @property {number} TYPE_FIXED64=6 TYPE_FIXED64 value
             * @property {number} TYPE_FIXED32=7 TYPE_FIXED32 value
             * @property {number} TYPE_BOOL=8 TYPE_BOOL value
             * @property {number} TYPE_STRING=9 TYPE_STRING value
             * @property {number} TYPE_GROUP=10 TYPE_GROUP value
             * @property {number} TYPE_MESSAGE=11 TYPE_MESSAGE value
             * @property {number} TYPE_BYTES=12 TYPE_BYTES value
             * @property {number} TYPE_UINT32=13 TYPE_UINT32 value
             * @property {number} TYPE_ENUM=14 TYPE_ENUM value
             * @property {number} TYPE_SFIXED32=15 TYPE_SFIXED32 value
             * @property {number} TYPE_SFIXED64=16 TYPE_SFIXED64 value
             * @property {number} TYPE_SINT32=17 TYPE_SINT32 value
             * @property {number} TYPE_SINT64=18 TYPE_SINT64 value
             */
            FieldDescriptorProto.Type = (function() {
                const valuesById = {}, values = Object.create(valuesById);
                values[valuesById[1] = "TYPE_DOUBLE"] = 1;
                values[valuesById[2] = "TYPE_FLOAT"] = 2;
                values[valuesById[3] = "TYPE_INT64"] = 3;
                values[valuesById[4] = "TYPE_UINT64"] = 4;
                values[valuesById[5] = "TYPE_INT32"] = 5;
                values[valuesById[6] = "TYPE_FIXED64"] = 6;
                values[valuesById[7] = "TYPE_FIXED32"] = 7;
                values[valuesById[8] = "TYPE_BOOL"] = 8;
                values[valuesById[9] = "TYPE_STRING"] = 9;
                values[valuesById[10] = "TYPE_GROUP"] = 10;
                values[valuesById[11] = "TYPE_MESSAGE"] = 11;
                values[valuesById[12] = "TYPE_BYTES"] = 12;
                values[valuesById[13] = "TYPE_UINT32"] = 13;
                values[valuesById[14] = "TYPE_ENUM"] = 14;
                values[valuesById[15] = "TYPE_SFIXED32"] = 15;
                values[valuesById[16] = "TYPE_SFIXED64"] = 16;
                values[valuesById[17] = "TYPE_SINT32"] = 17;
                values[valuesById[18] = "TYPE_SINT64"] = 18;
                return values;
            })();

            /**
             * Label enum.
             * @name google.protobuf.FieldDescriptorProto.Label
             * @enum {number}
             * @property {number} LABEL_OPTIONAL=1 LABEL_OPTIONAL value
             * @property {number} LABEL_REQUIRED=2 LABEL_REQUIRED value
             * @property {number} LABEL_REPEATED=3 LABEL_REPEATED value
             */
            FieldDescriptorProto.Label = (function() {
                const valuesById = {}, values = Object.create(valuesById);
                values[valuesById[1] = "LABEL_OPTIONAL"] = 1;
                values[valuesById[2] = "LABEL_REQUIRED"] = 2;
                values[valuesById[3] = "LABEL_REPEATED"] = 3;
                return values;
            })();

            return FieldDescriptorProto;
        })();

        protobuf.OneofDescriptorProto = (function() {

            /**
             * Properties of an OneofDescriptorProto.
             * @memberof google.protobuf
             * @interface IOneofDescriptorProto
             * @property {string|null} [name] OneofDescriptorProto name
             * @property {google.protobuf.IOneofOptions|null} [options] OneofDescriptorProto options
             */

            /**
             * Constructs a new OneofDescriptorProto.
             * @memberof google.protobuf
             * @classdesc Represents an OneofDescriptorProto.
             * @implements IOneofDescriptorProto
             * @constructor
             * @param {google.protobuf.IOneofDescriptorProto=} [properties] Properties to set
             */
            function OneofDescriptorProto(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * OneofDescriptorProto name.
             * @member {string} name
             * @memberof google.protobuf.OneofDescriptorProto
             * @instance
             */
            OneofDescriptorProto.prototype.name = "";

            /**
             * OneofDescriptorProto options.
             * @member {google.protobuf.IOneofOptions|null|undefined} options
             * @memberof google.protobuf.OneofDescriptorProto
             * @instance
             */
            OneofDescriptorProto.prototype.options = null;

            /**
             * Creates a new OneofDescriptorProto instance using the specified properties.
             * @function create
             * @memberof google.protobuf.OneofDescriptorProto
             * @static
             * @param {google.protobuf.IOneofDescriptorProto=} [properties] Properties to set
             * @returns {google.protobuf.OneofDescriptorProto} OneofDescriptorProto instance
             */
            OneofDescriptorProto.create = function create(properties) {
                return new OneofDescriptorProto(properties);
            };

            /**
             * Encodes the specified OneofDescriptorProto message. Does not implicitly {@link google.protobuf.OneofDescriptorProto.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.OneofDescriptorProto
             * @static
             * @param {google.protobuf.IOneofDescriptorProto} message OneofDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OneofDescriptorProto.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.options != null && Object.hasOwnProperty.call(message, "options"))
                    $root.google.protobuf.OneofOptions.encode(message.options, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified OneofDescriptorProto message, length delimited. Does not implicitly {@link google.protobuf.OneofDescriptorProto.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.OneofDescriptorProto
             * @static
             * @param {google.protobuf.IOneofDescriptorProto} message OneofDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OneofDescriptorProto.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an OneofDescriptorProto message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.OneofDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.OneofDescriptorProto} OneofDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OneofDescriptorProto.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.OneofDescriptorProto();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.name = reader.string();
                        break;
                    case 2:
                        message.options = $root.google.protobuf.OneofOptions.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an OneofDescriptorProto message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.OneofDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.OneofDescriptorProto} OneofDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OneofDescriptorProto.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an OneofDescriptorProto message.
             * @function verify
             * @memberof google.protobuf.OneofDescriptorProto
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OneofDescriptorProto.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.name != null && message.hasOwnProperty("name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.options != null && message.hasOwnProperty("options")) {
                    let error = $root.google.protobuf.OneofOptions.verify(message.options);
                    if (error)
                        return "options." + error;
                }
                return null;
            };

            /**
             * Creates an OneofDescriptorProto message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.OneofDescriptorProto
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.OneofDescriptorProto} OneofDescriptorProto
             */
            OneofDescriptorProto.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.OneofDescriptorProto)
                    return object;
                let message = new $root.google.protobuf.OneofDescriptorProto();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.options != null) {
                    if (typeof object.options !== "object")
                        throw TypeError(".google.protobuf.OneofDescriptorProto.options: object expected");
                    message.options = $root.google.protobuf.OneofOptions.fromObject(object.options);
                }
                return message;
            };

            /**
             * Creates a plain object from an OneofDescriptorProto message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.OneofDescriptorProto
             * @static
             * @param {google.protobuf.OneofDescriptorProto} message OneofDescriptorProto
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OneofDescriptorProto.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.name = "";
                    object.options = null;
                }
                if (message.name != null && message.hasOwnProperty("name"))
                    object.name = message.name;
                if (message.options != null && message.hasOwnProperty("options"))
                    object.options = $root.google.protobuf.OneofOptions.toObject(message.options, options);
                return object;
            };

            /**
             * Converts this OneofDescriptorProto to JSON.
             * @function toJSON
             * @memberof google.protobuf.OneofDescriptorProto
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OneofDescriptorProto.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return OneofDescriptorProto;
        })();

        protobuf.EnumDescriptorProto = (function() {

            /**
             * Properties of an EnumDescriptorProto.
             * @memberof google.protobuf
             * @interface IEnumDescriptorProto
             * @property {string|null} [name] EnumDescriptorProto name
             * @property {Array.<google.protobuf.IEnumValueDescriptorProto>|null} [value] EnumDescriptorProto value
             * @property {google.protobuf.IEnumOptions|null} [options] EnumDescriptorProto options
             */

            /**
             * Constructs a new EnumDescriptorProto.
             * @memberof google.protobuf
             * @classdesc Represents an EnumDescriptorProto.
             * @implements IEnumDescriptorProto
             * @constructor
             * @param {google.protobuf.IEnumDescriptorProto=} [properties] Properties to set
             */
            function EnumDescriptorProto(properties) {
                this.value = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * EnumDescriptorProto name.
             * @member {string} name
             * @memberof google.protobuf.EnumDescriptorProto
             * @instance
             */
            EnumDescriptorProto.prototype.name = "";

            /**
             * EnumDescriptorProto value.
             * @member {Array.<google.protobuf.IEnumValueDescriptorProto>} value
             * @memberof google.protobuf.EnumDescriptorProto
             * @instance
             */
            EnumDescriptorProto.prototype.value = $util.emptyArray;

            /**
             * EnumDescriptorProto options.
             * @member {google.protobuf.IEnumOptions|null|undefined} options
             * @memberof google.protobuf.EnumDescriptorProto
             * @instance
             */
            EnumDescriptorProto.prototype.options = null;

            /**
             * Creates a new EnumDescriptorProto instance using the specified properties.
             * @function create
             * @memberof google.protobuf.EnumDescriptorProto
             * @static
             * @param {google.protobuf.IEnumDescriptorProto=} [properties] Properties to set
             * @returns {google.protobuf.EnumDescriptorProto} EnumDescriptorProto instance
             */
            EnumDescriptorProto.create = function create(properties) {
                return new EnumDescriptorProto(properties);
            };

            /**
             * Encodes the specified EnumDescriptorProto message. Does not implicitly {@link google.protobuf.EnumDescriptorProto.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.EnumDescriptorProto
             * @static
             * @param {google.protobuf.IEnumDescriptorProto} message EnumDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EnumDescriptorProto.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.value != null && message.value.length)
                    for (let i = 0; i < message.value.length; ++i)
                        $root.google.protobuf.EnumValueDescriptorProto.encode(message.value[i], writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.options != null && Object.hasOwnProperty.call(message, "options"))
                    $root.google.protobuf.EnumOptions.encode(message.options, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified EnumDescriptorProto message, length delimited. Does not implicitly {@link google.protobuf.EnumDescriptorProto.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.EnumDescriptorProto
             * @static
             * @param {google.protobuf.IEnumDescriptorProto} message EnumDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EnumDescriptorProto.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an EnumDescriptorProto message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.EnumDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.EnumDescriptorProto} EnumDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EnumDescriptorProto.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.EnumDescriptorProto();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.name = reader.string();
                        break;
                    case 2:
                        if (!(message.value && message.value.length))
                            message.value = [];
                        message.value.push($root.google.protobuf.EnumValueDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 3:
                        message.options = $root.google.protobuf.EnumOptions.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an EnumDescriptorProto message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.EnumDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.EnumDescriptorProto} EnumDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EnumDescriptorProto.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an EnumDescriptorProto message.
             * @function verify
             * @memberof google.protobuf.EnumDescriptorProto
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            EnumDescriptorProto.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.name != null && message.hasOwnProperty("name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.value != null && message.hasOwnProperty("value")) {
                    if (!Array.isArray(message.value))
                        return "value: array expected";
                    for (let i = 0; i < message.value.length; ++i) {
                        let error = $root.google.protobuf.EnumValueDescriptorProto.verify(message.value[i]);
                        if (error)
                            return "value." + error;
                    }
                }
                if (message.options != null && message.hasOwnProperty("options")) {
                    let error = $root.google.protobuf.EnumOptions.verify(message.options);
                    if (error)
                        return "options." + error;
                }
                return null;
            };

            /**
             * Creates an EnumDescriptorProto message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.EnumDescriptorProto
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.EnumDescriptorProto} EnumDescriptorProto
             */
            EnumDescriptorProto.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.EnumDescriptorProto)
                    return object;
                let message = new $root.google.protobuf.EnumDescriptorProto();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.value) {
                    if (!Array.isArray(object.value))
                        throw TypeError(".google.protobuf.EnumDescriptorProto.value: array expected");
                    message.value = [];
                    for (let i = 0; i < object.value.length; ++i) {
                        if (typeof object.value[i] !== "object")
                            throw TypeError(".google.protobuf.EnumDescriptorProto.value: object expected");
                        message.value[i] = $root.google.protobuf.EnumValueDescriptorProto.fromObject(object.value[i]);
                    }
                }
                if (object.options != null) {
                    if (typeof object.options !== "object")
                        throw TypeError(".google.protobuf.EnumDescriptorProto.options: object expected");
                    message.options = $root.google.protobuf.EnumOptions.fromObject(object.options);
                }
                return message;
            };

            /**
             * Creates a plain object from an EnumDescriptorProto message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.EnumDescriptorProto
             * @static
             * @param {google.protobuf.EnumDescriptorProto} message EnumDescriptorProto
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            EnumDescriptorProto.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.value = [];
                if (options.defaults) {
                    object.name = "";
                    object.options = null;
                }
                if (message.name != null && message.hasOwnProperty("name"))
                    object.name = message.name;
                if (message.value && message.value.length) {
                    object.value = [];
                    for (let j = 0; j < message.value.length; ++j)
                        object.value[j] = $root.google.protobuf.EnumValueDescriptorProto.toObject(message.value[j], options);
                }
                if (message.options != null && message.hasOwnProperty("options"))
                    object.options = $root.google.protobuf.EnumOptions.toObject(message.options, options);
                return object;
            };

            /**
             * Converts this EnumDescriptorProto to JSON.
             * @function toJSON
             * @memberof google.protobuf.EnumDescriptorProto
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            EnumDescriptorProto.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return EnumDescriptorProto;
        })();

        protobuf.EnumValueDescriptorProto = (function() {

            /**
             * Properties of an EnumValueDescriptorProto.
             * @memberof google.protobuf
             * @interface IEnumValueDescriptorProto
             * @property {string|null} [name] EnumValueDescriptorProto name
             * @property {number|null} [number] EnumValueDescriptorProto number
             * @property {google.protobuf.IEnumValueOptions|null} [options] EnumValueDescriptorProto options
             */

            /**
             * Constructs a new EnumValueDescriptorProto.
             * @memberof google.protobuf
             * @classdesc Represents an EnumValueDescriptorProto.
             * @implements IEnumValueDescriptorProto
             * @constructor
             * @param {google.protobuf.IEnumValueDescriptorProto=} [properties] Properties to set
             */
            function EnumValueDescriptorProto(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * EnumValueDescriptorProto name.
             * @member {string} name
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @instance
             */
            EnumValueDescriptorProto.prototype.name = "";

            /**
             * EnumValueDescriptorProto number.
             * @member {number} number
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @instance
             */
            EnumValueDescriptorProto.prototype.number = 0;

            /**
             * EnumValueDescriptorProto options.
             * @member {google.protobuf.IEnumValueOptions|null|undefined} options
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @instance
             */
            EnumValueDescriptorProto.prototype.options = null;

            /**
             * Creates a new EnumValueDescriptorProto instance using the specified properties.
             * @function create
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @static
             * @param {google.protobuf.IEnumValueDescriptorProto=} [properties] Properties to set
             * @returns {google.protobuf.EnumValueDescriptorProto} EnumValueDescriptorProto instance
             */
            EnumValueDescriptorProto.create = function create(properties) {
                return new EnumValueDescriptorProto(properties);
            };

            /**
             * Encodes the specified EnumValueDescriptorProto message. Does not implicitly {@link google.protobuf.EnumValueDescriptorProto.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @static
             * @param {google.protobuf.IEnumValueDescriptorProto} message EnumValueDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EnumValueDescriptorProto.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.number != null && Object.hasOwnProperty.call(message, "number"))
                    writer.uint32(/* id 2, wireType 0 =*/16).int32(message.number);
                if (message.options != null && Object.hasOwnProperty.call(message, "options"))
                    $root.google.protobuf.EnumValueOptions.encode(message.options, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified EnumValueDescriptorProto message, length delimited. Does not implicitly {@link google.protobuf.EnumValueDescriptorProto.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @static
             * @param {google.protobuf.IEnumValueDescriptorProto} message EnumValueDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EnumValueDescriptorProto.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an EnumValueDescriptorProto message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.EnumValueDescriptorProto} EnumValueDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EnumValueDescriptorProto.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.EnumValueDescriptorProto();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.name = reader.string();
                        break;
                    case 2:
                        message.number = reader.int32();
                        break;
                    case 3:
                        message.options = $root.google.protobuf.EnumValueOptions.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an EnumValueDescriptorProto message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.EnumValueDescriptorProto} EnumValueDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EnumValueDescriptorProto.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an EnumValueDescriptorProto message.
             * @function verify
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            EnumValueDescriptorProto.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.name != null && message.hasOwnProperty("name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.number != null && message.hasOwnProperty("number"))
                    if (!$util.isInteger(message.number))
                        return "number: integer expected";
                if (message.options != null && message.hasOwnProperty("options")) {
                    let error = $root.google.protobuf.EnumValueOptions.verify(message.options);
                    if (error)
                        return "options." + error;
                }
                return null;
            };

            /**
             * Creates an EnumValueDescriptorProto message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.EnumValueDescriptorProto} EnumValueDescriptorProto
             */
            EnumValueDescriptorProto.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.EnumValueDescriptorProto)
                    return object;
                let message = new $root.google.protobuf.EnumValueDescriptorProto();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.number != null)
                    message.number = object.number | 0;
                if (object.options != null) {
                    if (typeof object.options !== "object")
                        throw TypeError(".google.protobuf.EnumValueDescriptorProto.options: object expected");
                    message.options = $root.google.protobuf.EnumValueOptions.fromObject(object.options);
                }
                return message;
            };

            /**
             * Creates a plain object from an EnumValueDescriptorProto message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @static
             * @param {google.protobuf.EnumValueDescriptorProto} message EnumValueDescriptorProto
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            EnumValueDescriptorProto.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.name = "";
                    object.number = 0;
                    object.options = null;
                }
                if (message.name != null && message.hasOwnProperty("name"))
                    object.name = message.name;
                if (message.number != null && message.hasOwnProperty("number"))
                    object.number = message.number;
                if (message.options != null && message.hasOwnProperty("options"))
                    object.options = $root.google.protobuf.EnumValueOptions.toObject(message.options, options);
                return object;
            };

            /**
             * Converts this EnumValueDescriptorProto to JSON.
             * @function toJSON
             * @memberof google.protobuf.EnumValueDescriptorProto
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            EnumValueDescriptorProto.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return EnumValueDescriptorProto;
        })();

        protobuf.ServiceDescriptorProto = (function() {

            /**
             * Properties of a ServiceDescriptorProto.
             * @memberof google.protobuf
             * @interface IServiceDescriptorProto
             * @property {string|null} [name] ServiceDescriptorProto name
             * @property {Array.<google.protobuf.IMethodDescriptorProto>|null} [method] ServiceDescriptorProto method
             * @property {google.protobuf.IServiceOptions|null} [options] ServiceDescriptorProto options
             */

            /**
             * Constructs a new ServiceDescriptorProto.
             * @memberof google.protobuf
             * @classdesc Represents a ServiceDescriptorProto.
             * @implements IServiceDescriptorProto
             * @constructor
             * @param {google.protobuf.IServiceDescriptorProto=} [properties] Properties to set
             */
            function ServiceDescriptorProto(properties) {
                this.method = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ServiceDescriptorProto name.
             * @member {string} name
             * @memberof google.protobuf.ServiceDescriptorProto
             * @instance
             */
            ServiceDescriptorProto.prototype.name = "";

            /**
             * ServiceDescriptorProto method.
             * @member {Array.<google.protobuf.IMethodDescriptorProto>} method
             * @memberof google.protobuf.ServiceDescriptorProto
             * @instance
             */
            ServiceDescriptorProto.prototype.method = $util.emptyArray;

            /**
             * ServiceDescriptorProto options.
             * @member {google.protobuf.IServiceOptions|null|undefined} options
             * @memberof google.protobuf.ServiceDescriptorProto
             * @instance
             */
            ServiceDescriptorProto.prototype.options = null;

            /**
             * Creates a new ServiceDescriptorProto instance using the specified properties.
             * @function create
             * @memberof google.protobuf.ServiceDescriptorProto
             * @static
             * @param {google.protobuf.IServiceDescriptorProto=} [properties] Properties to set
             * @returns {google.protobuf.ServiceDescriptorProto} ServiceDescriptorProto instance
             */
            ServiceDescriptorProto.create = function create(properties) {
                return new ServiceDescriptorProto(properties);
            };

            /**
             * Encodes the specified ServiceDescriptorProto message. Does not implicitly {@link google.protobuf.ServiceDescriptorProto.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.ServiceDescriptorProto
             * @static
             * @param {google.protobuf.IServiceDescriptorProto} message ServiceDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ServiceDescriptorProto.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.method != null && message.method.length)
                    for (let i = 0; i < message.method.length; ++i)
                        $root.google.protobuf.MethodDescriptorProto.encode(message.method[i], writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.options != null && Object.hasOwnProperty.call(message, "options"))
                    $root.google.protobuf.ServiceOptions.encode(message.options, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified ServiceDescriptorProto message, length delimited. Does not implicitly {@link google.protobuf.ServiceDescriptorProto.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.ServiceDescriptorProto
             * @static
             * @param {google.protobuf.IServiceDescriptorProto} message ServiceDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ServiceDescriptorProto.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a ServiceDescriptorProto message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.ServiceDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.ServiceDescriptorProto} ServiceDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ServiceDescriptorProto.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.ServiceDescriptorProto();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.name = reader.string();
                        break;
                    case 2:
                        if (!(message.method && message.method.length))
                            message.method = [];
                        message.method.push($root.google.protobuf.MethodDescriptorProto.decode(reader, reader.uint32()));
                        break;
                    case 3:
                        message.options = $root.google.protobuf.ServiceOptions.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a ServiceDescriptorProto message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.ServiceDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.ServiceDescriptorProto} ServiceDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ServiceDescriptorProto.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a ServiceDescriptorProto message.
             * @function verify
             * @memberof google.protobuf.ServiceDescriptorProto
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ServiceDescriptorProto.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.name != null && message.hasOwnProperty("name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.method != null && message.hasOwnProperty("method")) {
                    if (!Array.isArray(message.method))
                        return "method: array expected";
                    for (let i = 0; i < message.method.length; ++i) {
                        let error = $root.google.protobuf.MethodDescriptorProto.verify(message.method[i]);
                        if (error)
                            return "method." + error;
                    }
                }
                if (message.options != null && message.hasOwnProperty("options")) {
                    let error = $root.google.protobuf.ServiceOptions.verify(message.options);
                    if (error)
                        return "options." + error;
                }
                return null;
            };

            /**
             * Creates a ServiceDescriptorProto message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.ServiceDescriptorProto
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.ServiceDescriptorProto} ServiceDescriptorProto
             */
            ServiceDescriptorProto.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.ServiceDescriptorProto)
                    return object;
                let message = new $root.google.protobuf.ServiceDescriptorProto();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.method) {
                    if (!Array.isArray(object.method))
                        throw TypeError(".google.protobuf.ServiceDescriptorProto.method: array expected");
                    message.method = [];
                    for (let i = 0; i < object.method.length; ++i) {
                        if (typeof object.method[i] !== "object")
                            throw TypeError(".google.protobuf.ServiceDescriptorProto.method: object expected");
                        message.method[i] = $root.google.protobuf.MethodDescriptorProto.fromObject(object.method[i]);
                    }
                }
                if (object.options != null) {
                    if (typeof object.options !== "object")
                        throw TypeError(".google.protobuf.ServiceDescriptorProto.options: object expected");
                    message.options = $root.google.protobuf.ServiceOptions.fromObject(object.options);
                }
                return message;
            };

            /**
             * Creates a plain object from a ServiceDescriptorProto message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.ServiceDescriptorProto
             * @static
             * @param {google.protobuf.ServiceDescriptorProto} message ServiceDescriptorProto
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ServiceDescriptorProto.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.method = [];
                if (options.defaults) {
                    object.name = "";
                    object.options = null;
                }
                if (message.name != null && message.hasOwnProperty("name"))
                    object.name = message.name;
                if (message.method && message.method.length) {
                    object.method = [];
                    for (let j = 0; j < message.method.length; ++j)
                        object.method[j] = $root.google.protobuf.MethodDescriptorProto.toObject(message.method[j], options);
                }
                if (message.options != null && message.hasOwnProperty("options"))
                    object.options = $root.google.protobuf.ServiceOptions.toObject(message.options, options);
                return object;
            };

            /**
             * Converts this ServiceDescriptorProto to JSON.
             * @function toJSON
             * @memberof google.protobuf.ServiceDescriptorProto
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ServiceDescriptorProto.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return ServiceDescriptorProto;
        })();

        protobuf.MethodDescriptorProto = (function() {

            /**
             * Properties of a MethodDescriptorProto.
             * @memberof google.protobuf
             * @interface IMethodDescriptorProto
             * @property {string|null} [name] MethodDescriptorProto name
             * @property {string|null} [inputType] MethodDescriptorProto inputType
             * @property {string|null} [outputType] MethodDescriptorProto outputType
             * @property {google.protobuf.IMethodOptions|null} [options] MethodDescriptorProto options
             * @property {boolean|null} [clientStreaming] MethodDescriptorProto clientStreaming
             * @property {boolean|null} [serverStreaming] MethodDescriptorProto serverStreaming
             */

            /**
             * Constructs a new MethodDescriptorProto.
             * @memberof google.protobuf
             * @classdesc Represents a MethodDescriptorProto.
             * @implements IMethodDescriptorProto
             * @constructor
             * @param {google.protobuf.IMethodDescriptorProto=} [properties] Properties to set
             */
            function MethodDescriptorProto(properties) {
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * MethodDescriptorProto name.
             * @member {string} name
             * @memberof google.protobuf.MethodDescriptorProto
             * @instance
             */
            MethodDescriptorProto.prototype.name = "";

            /**
             * MethodDescriptorProto inputType.
             * @member {string} inputType
             * @memberof google.protobuf.MethodDescriptorProto
             * @instance
             */
            MethodDescriptorProto.prototype.inputType = "";

            /**
             * MethodDescriptorProto outputType.
             * @member {string} outputType
             * @memberof google.protobuf.MethodDescriptorProto
             * @instance
             */
            MethodDescriptorProto.prototype.outputType = "";

            /**
             * MethodDescriptorProto options.
             * @member {google.protobuf.IMethodOptions|null|undefined} options
             * @memberof google.protobuf.MethodDescriptorProto
             * @instance
             */
            MethodDescriptorProto.prototype.options = null;

            /**
             * MethodDescriptorProto clientStreaming.
             * @member {boolean} clientStreaming
             * @memberof google.protobuf.MethodDescriptorProto
             * @instance
             */
            MethodDescriptorProto.prototype.clientStreaming = false;

            /**
             * MethodDescriptorProto serverStreaming.
             * @member {boolean} serverStreaming
             * @memberof google.protobuf.MethodDescriptorProto
             * @instance
             */
            MethodDescriptorProto.prototype.serverStreaming = false;

            /**
             * Creates a new MethodDescriptorProto instance using the specified properties.
             * @function create
             * @memberof google.protobuf.MethodDescriptorProto
             * @static
             * @param {google.protobuf.IMethodDescriptorProto=} [properties] Properties to set
             * @returns {google.protobuf.MethodDescriptorProto} MethodDescriptorProto instance
             */
            MethodDescriptorProto.create = function create(properties) {
                return new MethodDescriptorProto(properties);
            };

            /**
             * Encodes the specified MethodDescriptorProto message. Does not implicitly {@link google.protobuf.MethodDescriptorProto.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.MethodDescriptorProto
             * @static
             * @param {google.protobuf.IMethodDescriptorProto} message MethodDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MethodDescriptorProto.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                if (message.inputType != null && Object.hasOwnProperty.call(message, "inputType"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.inputType);
                if (message.outputType != null && Object.hasOwnProperty.call(message, "outputType"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.outputType);
                if (message.options != null && Object.hasOwnProperty.call(message, "options"))
                    $root.google.protobuf.MethodOptions.encode(message.options, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.clientStreaming != null && Object.hasOwnProperty.call(message, "clientStreaming"))
                    writer.uint32(/* id 5, wireType 0 =*/40).bool(message.clientStreaming);
                if (message.serverStreaming != null && Object.hasOwnProperty.call(message, "serverStreaming"))
                    writer.uint32(/* id 6, wireType 0 =*/48).bool(message.serverStreaming);
                return writer;
            };

            /**
             * Encodes the specified MethodDescriptorProto message, length delimited. Does not implicitly {@link google.protobuf.MethodDescriptorProto.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.MethodDescriptorProto
             * @static
             * @param {google.protobuf.IMethodDescriptorProto} message MethodDescriptorProto message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MethodDescriptorProto.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a MethodDescriptorProto message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.MethodDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.MethodDescriptorProto} MethodDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MethodDescriptorProto.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.MethodDescriptorProto();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.name = reader.string();
                        break;
                    case 2:
                        message.inputType = reader.string();
                        break;
                    case 3:
                        message.outputType = reader.string();
                        break;
                    case 4:
                        message.options = $root.google.protobuf.MethodOptions.decode(reader, reader.uint32());
                        break;
                    case 5:
                        message.clientStreaming = reader.bool();
                        break;
                    case 6:
                        message.serverStreaming = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a MethodDescriptorProto message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.MethodDescriptorProto
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.MethodDescriptorProto} MethodDescriptorProto
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MethodDescriptorProto.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a MethodDescriptorProto message.
             * @function verify
             * @memberof google.protobuf.MethodDescriptorProto
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            MethodDescriptorProto.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.name != null && message.hasOwnProperty("name"))
                    if (!$util.isString(message.name))
                        return "name: string expected";
                if (message.inputType != null && message.hasOwnProperty("inputType"))
                    if (!$util.isString(message.inputType))
                        return "inputType: string expected";
                if (message.outputType != null && message.hasOwnProperty("outputType"))
                    if (!$util.isString(message.outputType))
                        return "outputType: string expected";
                if (message.options != null && message.hasOwnProperty("options")) {
                    let error = $root.google.protobuf.MethodOptions.verify(message.options);
                    if (error)
                        return "options." + error;
                }
                if (message.clientStreaming != null && message.hasOwnProperty("clientStreaming"))
                    if (typeof message.clientStreaming !== "boolean")
                        return "clientStreaming: boolean expected";
                if (message.serverStreaming != null && message.hasOwnProperty("serverStreaming"))
                    if (typeof message.serverStreaming !== "boolean")
                        return "serverStreaming: boolean expected";
                return null;
            };

            /**
             * Creates a MethodDescriptorProto message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.MethodDescriptorProto
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.MethodDescriptorProto} MethodDescriptorProto
             */
            MethodDescriptorProto.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.MethodDescriptorProto)
                    return object;
                let message = new $root.google.protobuf.MethodDescriptorProto();
                if (object.name != null)
                    message.name = String(object.name);
                if (object.inputType != null)
                    message.inputType = String(object.inputType);
                if (object.outputType != null)
                    message.outputType = String(object.outputType);
                if (object.options != null) {
                    if (typeof object.options !== "object")
                        throw TypeError(".google.protobuf.MethodDescriptorProto.options: object expected");
                    message.options = $root.google.protobuf.MethodOptions.fromObject(object.options);
                }
                if (object.clientStreaming != null)
                    message.clientStreaming = Boolean(object.clientStreaming);
                if (object.serverStreaming != null)
                    message.serverStreaming = Boolean(object.serverStreaming);
                return message;
            };

            /**
             * Creates a plain object from a MethodDescriptorProto message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.MethodDescriptorProto
             * @static
             * @param {google.protobuf.MethodDescriptorProto} message MethodDescriptorProto
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            MethodDescriptorProto.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.defaults) {
                    object.name = "";
                    object.inputType = "";
                    object.outputType = "";
                    object.options = null;
                    object.clientStreaming = false;
                    object.serverStreaming = false;
                }
                if (message.name != null && message.hasOwnProperty("name"))
                    object.name = message.name;
                if (message.inputType != null && message.hasOwnProperty("inputType"))
                    object.inputType = message.inputType;
                if (message.outputType != null && message.hasOwnProperty("outputType"))
                    object.outputType = message.outputType;
                if (message.options != null && message.hasOwnProperty("options"))
                    object.options = $root.google.protobuf.MethodOptions.toObject(message.options, options);
                if (message.clientStreaming != null && message.hasOwnProperty("clientStreaming"))
                    object.clientStreaming = message.clientStreaming;
                if (message.serverStreaming != null && message.hasOwnProperty("serverStreaming"))
                    object.serverStreaming = message.serverStreaming;
                return object;
            };

            /**
             * Converts this MethodDescriptorProto to JSON.
             * @function toJSON
             * @memberof google.protobuf.MethodDescriptorProto
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            MethodDescriptorProto.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return MethodDescriptorProto;
        })();

        protobuf.FileOptions = (function() {

            /**
             * Properties of a FileOptions.
             * @memberof google.protobuf
             * @interface IFileOptions
             * @property {string|null} [javaPackage] FileOptions javaPackage
             * @property {string|null} [javaOuterClassname] FileOptions javaOuterClassname
             * @property {boolean|null} [javaMultipleFiles] FileOptions javaMultipleFiles
             * @property {boolean|null} [javaGenerateEqualsAndHash] FileOptions javaGenerateEqualsAndHash
             * @property {boolean|null} [javaStringCheckUtf8] FileOptions javaStringCheckUtf8
             * @property {google.protobuf.FileOptions.OptimizeMode|null} [optimizeFor] FileOptions optimizeFor
             * @property {string|null} [goPackage] FileOptions goPackage
             * @property {boolean|null} [ccGenericServices] FileOptions ccGenericServices
             * @property {boolean|null} [javaGenericServices] FileOptions javaGenericServices
             * @property {boolean|null} [pyGenericServices] FileOptions pyGenericServices
             * @property {boolean|null} [deprecated] FileOptions deprecated
             * @property {boolean|null} [ccEnableArenas] FileOptions ccEnableArenas
             * @property {string|null} [objcClassPrefix] FileOptions objcClassPrefix
             * @property {string|null} [csharpNamespace] FileOptions csharpNamespace
             * @property {Array.<google.protobuf.IUninterpretedOption>|null} [uninterpretedOption] FileOptions uninterpretedOption
             */

            /**
             * Constructs a new FileOptions.
             * @memberof google.protobuf
             * @classdesc Represents a FileOptions.
             * @implements IFileOptions
             * @constructor
             * @param {google.protobuf.IFileOptions=} [properties] Properties to set
             */
            function FileOptions(properties) {
                this.uninterpretedOption = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FileOptions javaPackage.
             * @member {string} javaPackage
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.javaPackage = "";

            /**
             * FileOptions javaOuterClassname.
             * @member {string} javaOuterClassname
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.javaOuterClassname = "";

            /**
             * FileOptions javaMultipleFiles.
             * @member {boolean} javaMultipleFiles
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.javaMultipleFiles = false;

            /**
             * FileOptions javaGenerateEqualsAndHash.
             * @member {boolean} javaGenerateEqualsAndHash
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.javaGenerateEqualsAndHash = false;

            /**
             * FileOptions javaStringCheckUtf8.
             * @member {boolean} javaStringCheckUtf8
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.javaStringCheckUtf8 = false;

            /**
             * FileOptions optimizeFor.
             * @member {google.protobuf.FileOptions.OptimizeMode} optimizeFor
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.optimizeFor = 1;

            /**
             * FileOptions goPackage.
             * @member {string} goPackage
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.goPackage = "";

            /**
             * FileOptions ccGenericServices.
             * @member {boolean} ccGenericServices
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.ccGenericServices = false;

            /**
             * FileOptions javaGenericServices.
             * @member {boolean} javaGenericServices
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.javaGenericServices = false;

            /**
             * FileOptions pyGenericServices.
             * @member {boolean} pyGenericServices
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.pyGenericServices = false;

            /**
             * FileOptions deprecated.
             * @member {boolean} deprecated
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.deprecated = false;

            /**
             * FileOptions ccEnableArenas.
             * @member {boolean} ccEnableArenas
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.ccEnableArenas = false;

            /**
             * FileOptions objcClassPrefix.
             * @member {string} objcClassPrefix
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.objcClassPrefix = "";

            /**
             * FileOptions csharpNamespace.
             * @member {string} csharpNamespace
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.csharpNamespace = "";

            /**
             * FileOptions uninterpretedOption.
             * @member {Array.<google.protobuf.IUninterpretedOption>} uninterpretedOption
             * @memberof google.protobuf.FileOptions
             * @instance
             */
            FileOptions.prototype.uninterpretedOption = $util.emptyArray;

            /**
             * Creates a new FileOptions instance using the specified properties.
             * @function create
             * @memberof google.protobuf.FileOptions
             * @static
             * @param {google.protobuf.IFileOptions=} [properties] Properties to set
             * @returns {google.protobuf.FileOptions} FileOptions instance
             */
            FileOptions.create = function create(properties) {
                return new FileOptions(properties);
            };

            /**
             * Encodes the specified FileOptions message. Does not implicitly {@link google.protobuf.FileOptions.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.FileOptions
             * @static
             * @param {google.protobuf.IFileOptions} message FileOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FileOptions.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.javaPackage != null && Object.hasOwnProperty.call(message, "javaPackage"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.javaPackage);
                if (message.javaOuterClassname != null && Object.hasOwnProperty.call(message, "javaOuterClassname"))
                    writer.uint32(/* id 8, wireType 2 =*/66).string(message.javaOuterClassname);
                if (message.optimizeFor != null && Object.hasOwnProperty.call(message, "optimizeFor"))
                    writer.uint32(/* id 9, wireType 0 =*/72).int32(message.optimizeFor);
                if (message.javaMultipleFiles != null && Object.hasOwnProperty.call(message, "javaMultipleFiles"))
                    writer.uint32(/* id 10, wireType 0 =*/80).bool(message.javaMultipleFiles);
                if (message.goPackage != null && Object.hasOwnProperty.call(message, "goPackage"))
                    writer.uint32(/* id 11, wireType 2 =*/90).string(message.goPackage);
                if (message.ccGenericServices != null && Object.hasOwnProperty.call(message, "ccGenericServices"))
                    writer.uint32(/* id 16, wireType 0 =*/128).bool(message.ccGenericServices);
                if (message.javaGenericServices != null && Object.hasOwnProperty.call(message, "javaGenericServices"))
                    writer.uint32(/* id 17, wireType 0 =*/136).bool(message.javaGenericServices);
                if (message.pyGenericServices != null && Object.hasOwnProperty.call(message, "pyGenericServices"))
                    writer.uint32(/* id 18, wireType 0 =*/144).bool(message.pyGenericServices);
                if (message.javaGenerateEqualsAndHash != null && Object.hasOwnProperty.call(message, "javaGenerateEqualsAndHash"))
                    writer.uint32(/* id 20, wireType 0 =*/160).bool(message.javaGenerateEqualsAndHash);
                if (message.deprecated != null && Object.hasOwnProperty.call(message, "deprecated"))
                    writer.uint32(/* id 23, wireType 0 =*/184).bool(message.deprecated);
                if (message.javaStringCheckUtf8 != null && Object.hasOwnProperty.call(message, "javaStringCheckUtf8"))
                    writer.uint32(/* id 27, wireType 0 =*/216).bool(message.javaStringCheckUtf8);
                if (message.ccEnableArenas != null && Object.hasOwnProperty.call(message, "ccEnableArenas"))
                    writer.uint32(/* id 31, wireType 0 =*/248).bool(message.ccEnableArenas);
                if (message.objcClassPrefix != null && Object.hasOwnProperty.call(message, "objcClassPrefix"))
                    writer.uint32(/* id 36, wireType 2 =*/290).string(message.objcClassPrefix);
                if (message.csharpNamespace != null && Object.hasOwnProperty.call(message, "csharpNamespace"))
                    writer.uint32(/* id 37, wireType 2 =*/298).string(message.csharpNamespace);
                if (message.uninterpretedOption != null && message.uninterpretedOption.length)
                    for (let i = 0; i < message.uninterpretedOption.length; ++i)
                        $root.google.protobuf.UninterpretedOption.encode(message.uninterpretedOption[i], writer.uint32(/* id 999, wireType 2 =*/7994).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified FileOptions message, length delimited. Does not implicitly {@link google.protobuf.FileOptions.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.FileOptions
             * @static
             * @param {google.protobuf.IFileOptions} message FileOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FileOptions.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FileOptions message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.FileOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.FileOptions} FileOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FileOptions.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.FileOptions();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.javaPackage = reader.string();
                        break;
                    case 8:
                        message.javaOuterClassname = reader.string();
                        break;
                    case 10:
                        message.javaMultipleFiles = reader.bool();
                        break;
                    case 20:
                        message.javaGenerateEqualsAndHash = reader.bool();
                        break;
                    case 27:
                        message.javaStringCheckUtf8 = reader.bool();
                        break;
                    case 9:
                        message.optimizeFor = reader.int32();
                        break;
                    case 11:
                        message.goPackage = reader.string();
                        break;
                    case 16:
                        message.ccGenericServices = reader.bool();
                        break;
                    case 17:
                        message.javaGenericServices = reader.bool();
                        break;
                    case 18:
                        message.pyGenericServices = reader.bool();
                        break;
                    case 23:
                        message.deprecated = reader.bool();
                        break;
                    case 31:
                        message.ccEnableArenas = reader.bool();
                        break;
                    case 36:
                        message.objcClassPrefix = reader.string();
                        break;
                    case 37:
                        message.csharpNamespace = reader.string();
                        break;
                    case 999:
                        if (!(message.uninterpretedOption && message.uninterpretedOption.length))
                            message.uninterpretedOption = [];
                        message.uninterpretedOption.push($root.google.protobuf.UninterpretedOption.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FileOptions message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.FileOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.FileOptions} FileOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FileOptions.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FileOptions message.
             * @function verify
             * @memberof google.protobuf.FileOptions
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FileOptions.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.javaPackage != null && message.hasOwnProperty("javaPackage"))
                    if (!$util.isString(message.javaPackage))
                        return "javaPackage: string expected";
                if (message.javaOuterClassname != null && message.hasOwnProperty("javaOuterClassname"))
                    if (!$util.isString(message.javaOuterClassname))
                        return "javaOuterClassname: string expected";
                if (message.javaMultipleFiles != null && message.hasOwnProperty("javaMultipleFiles"))
                    if (typeof message.javaMultipleFiles !== "boolean")
                        return "javaMultipleFiles: boolean expected";
                if (message.javaGenerateEqualsAndHash != null && message.hasOwnProperty("javaGenerateEqualsAndHash"))
                    if (typeof message.javaGenerateEqualsAndHash !== "boolean")
                        return "javaGenerateEqualsAndHash: boolean expected";
                if (message.javaStringCheckUtf8 != null && message.hasOwnProperty("javaStringCheckUtf8"))
                    if (typeof message.javaStringCheckUtf8 !== "boolean")
                        return "javaStringCheckUtf8: boolean expected";
                if (message.optimizeFor != null && message.hasOwnProperty("optimizeFor"))
                    switch (message.optimizeFor) {
                    default:
                        return "optimizeFor: enum value expected";
                    case 1:
                    case 2:
                    case 3:
                        break;
                    }
                if (message.goPackage != null && message.hasOwnProperty("goPackage"))
                    if (!$util.isString(message.goPackage))
                        return "goPackage: string expected";
                if (message.ccGenericServices != null && message.hasOwnProperty("ccGenericServices"))
                    if (typeof message.ccGenericServices !== "boolean")
                        return "ccGenericServices: boolean expected";
                if (message.javaGenericServices != null && message.hasOwnProperty("javaGenericServices"))
                    if (typeof message.javaGenericServices !== "boolean")
                        return "javaGenericServices: boolean expected";
                if (message.pyGenericServices != null && message.hasOwnProperty("pyGenericServices"))
                    if (typeof message.pyGenericServices !== "boolean")
                        return "pyGenericServices: boolean expected";
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    if (typeof message.deprecated !== "boolean")
                        return "deprecated: boolean expected";
                if (message.ccEnableArenas != null && message.hasOwnProperty("ccEnableArenas"))
                    if (typeof message.ccEnableArenas !== "boolean")
                        return "ccEnableArenas: boolean expected";
                if (message.objcClassPrefix != null && message.hasOwnProperty("objcClassPrefix"))
                    if (!$util.isString(message.objcClassPrefix))
                        return "objcClassPrefix: string expected";
                if (message.csharpNamespace != null && message.hasOwnProperty("csharpNamespace"))
                    if (!$util.isString(message.csharpNamespace))
                        return "csharpNamespace: string expected";
                if (message.uninterpretedOption != null && message.hasOwnProperty("uninterpretedOption")) {
                    if (!Array.isArray(message.uninterpretedOption))
                        return "uninterpretedOption: array expected";
                    for (let i = 0; i < message.uninterpretedOption.length; ++i) {
                        let error = $root.google.protobuf.UninterpretedOption.verify(message.uninterpretedOption[i]);
                        if (error)
                            return "uninterpretedOption." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a FileOptions message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.FileOptions
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.FileOptions} FileOptions
             */
            FileOptions.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.FileOptions)
                    return object;
                let message = new $root.google.protobuf.FileOptions();
                if (object.javaPackage != null)
                    message.javaPackage = String(object.javaPackage);
                if (object.javaOuterClassname != null)
                    message.javaOuterClassname = String(object.javaOuterClassname);
                if (object.javaMultipleFiles != null)
                    message.javaMultipleFiles = Boolean(object.javaMultipleFiles);
                if (object.javaGenerateEqualsAndHash != null)
                    message.javaGenerateEqualsAndHash = Boolean(object.javaGenerateEqualsAndHash);
                if (object.javaStringCheckUtf8 != null)
                    message.javaStringCheckUtf8 = Boolean(object.javaStringCheckUtf8);
                switch (object.optimizeFor) {
                case "SPEED":
                case 1:
                    message.optimizeFor = 1;
                    break;
                case "CODE_SIZE":
                case 2:
                    message.optimizeFor = 2;
                    break;
                case "LITE_RUNTIME":
                case 3:
                    message.optimizeFor = 3;
                    break;
                }
                if (object.goPackage != null)
                    message.goPackage = String(object.goPackage);
                if (object.ccGenericServices != null)
                    message.ccGenericServices = Boolean(object.ccGenericServices);
                if (object.javaGenericServices != null)
                    message.javaGenericServices = Boolean(object.javaGenericServices);
                if (object.pyGenericServices != null)
                    message.pyGenericServices = Boolean(object.pyGenericServices);
                if (object.deprecated != null)
                    message.deprecated = Boolean(object.deprecated);
                if (object.ccEnableArenas != null)
                    message.ccEnableArenas = Boolean(object.ccEnableArenas);
                if (object.objcClassPrefix != null)
                    message.objcClassPrefix = String(object.objcClassPrefix);
                if (object.csharpNamespace != null)
                    message.csharpNamespace = String(object.csharpNamespace);
                if (object.uninterpretedOption) {
                    if (!Array.isArray(object.uninterpretedOption))
                        throw TypeError(".google.protobuf.FileOptions.uninterpretedOption: array expected");
                    message.uninterpretedOption = [];
                    for (let i = 0; i < object.uninterpretedOption.length; ++i) {
                        if (typeof object.uninterpretedOption[i] !== "object")
                            throw TypeError(".google.protobuf.FileOptions.uninterpretedOption: object expected");
                        message.uninterpretedOption[i] = $root.google.protobuf.UninterpretedOption.fromObject(object.uninterpretedOption[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a FileOptions message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.FileOptions
             * @static
             * @param {google.protobuf.FileOptions} message FileOptions
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FileOptions.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.uninterpretedOption = [];
                if (options.defaults) {
                    object.javaPackage = "";
                    object.javaOuterClassname = "";
                    object.optimizeFor = options.enums === String ? "SPEED" : 1;
                    object.javaMultipleFiles = false;
                    object.goPackage = "";
                    object.ccGenericServices = false;
                    object.javaGenericServices = false;
                    object.pyGenericServices = false;
                    object.javaGenerateEqualsAndHash = false;
                    object.deprecated = false;
                    object.javaStringCheckUtf8 = false;
                    object.ccEnableArenas = false;
                    object.objcClassPrefix = "";
                    object.csharpNamespace = "";
                }
                if (message.javaPackage != null && message.hasOwnProperty("javaPackage"))
                    object.javaPackage = message.javaPackage;
                if (message.javaOuterClassname != null && message.hasOwnProperty("javaOuterClassname"))
                    object.javaOuterClassname = message.javaOuterClassname;
                if (message.optimizeFor != null && message.hasOwnProperty("optimizeFor"))
                    object.optimizeFor = options.enums === String ? $root.google.protobuf.FileOptions.OptimizeMode[message.optimizeFor] : message.optimizeFor;
                if (message.javaMultipleFiles != null && message.hasOwnProperty("javaMultipleFiles"))
                    object.javaMultipleFiles = message.javaMultipleFiles;
                if (message.goPackage != null && message.hasOwnProperty("goPackage"))
                    object.goPackage = message.goPackage;
                if (message.ccGenericServices != null && message.hasOwnProperty("ccGenericServices"))
                    object.ccGenericServices = message.ccGenericServices;
                if (message.javaGenericServices != null && message.hasOwnProperty("javaGenericServices"))
                    object.javaGenericServices = message.javaGenericServices;
                if (message.pyGenericServices != null && message.hasOwnProperty("pyGenericServices"))
                    object.pyGenericServices = message.pyGenericServices;
                if (message.javaGenerateEqualsAndHash != null && message.hasOwnProperty("javaGenerateEqualsAndHash"))
                    object.javaGenerateEqualsAndHash = message.javaGenerateEqualsAndHash;
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    object.deprecated = message.deprecated;
                if (message.javaStringCheckUtf8 != null && message.hasOwnProperty("javaStringCheckUtf8"))
                    object.javaStringCheckUtf8 = message.javaStringCheckUtf8;
                if (message.ccEnableArenas != null && message.hasOwnProperty("ccEnableArenas"))
                    object.ccEnableArenas = message.ccEnableArenas;
                if (message.objcClassPrefix != null && message.hasOwnProperty("objcClassPrefix"))
                    object.objcClassPrefix = message.objcClassPrefix;
                if (message.csharpNamespace != null && message.hasOwnProperty("csharpNamespace"))
                    object.csharpNamespace = message.csharpNamespace;
                if (message.uninterpretedOption && message.uninterpretedOption.length) {
                    object.uninterpretedOption = [];
                    for (let j = 0; j < message.uninterpretedOption.length; ++j)
                        object.uninterpretedOption[j] = $root.google.protobuf.UninterpretedOption.toObject(message.uninterpretedOption[j], options);
                }
                return object;
            };

            /**
             * Converts this FileOptions to JSON.
             * @function toJSON
             * @memberof google.protobuf.FileOptions
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FileOptions.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * OptimizeMode enum.
             * @name google.protobuf.FileOptions.OptimizeMode
             * @enum {number}
             * @property {number} SPEED=1 SPEED value
             * @property {number} CODE_SIZE=2 CODE_SIZE value
             * @property {number} LITE_RUNTIME=3 LITE_RUNTIME value
             */
            FileOptions.OptimizeMode = (function() {
                const valuesById = {}, values = Object.create(valuesById);
                values[valuesById[1] = "SPEED"] = 1;
                values[valuesById[2] = "CODE_SIZE"] = 2;
                values[valuesById[3] = "LITE_RUNTIME"] = 3;
                return values;
            })();

            return FileOptions;
        })();

        protobuf.MessageOptions = (function() {

            /**
             * Properties of a MessageOptions.
             * @memberof google.protobuf
             * @interface IMessageOptions
             * @property {boolean|null} [messageSetWireFormat] MessageOptions messageSetWireFormat
             * @property {boolean|null} [noStandardDescriptorAccessor] MessageOptions noStandardDescriptorAccessor
             * @property {boolean|null} [deprecated] MessageOptions deprecated
             * @property {boolean|null} [mapEntry] MessageOptions mapEntry
             * @property {Array.<google.protobuf.IUninterpretedOption>|null} [uninterpretedOption] MessageOptions uninterpretedOption
             */

            /**
             * Constructs a new MessageOptions.
             * @memberof google.protobuf
             * @classdesc Represents a MessageOptions.
             * @implements IMessageOptions
             * @constructor
             * @param {google.protobuf.IMessageOptions=} [properties] Properties to set
             */
            function MessageOptions(properties) {
                this.uninterpretedOption = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * MessageOptions messageSetWireFormat.
             * @member {boolean} messageSetWireFormat
             * @memberof google.protobuf.MessageOptions
             * @instance
             */
            MessageOptions.prototype.messageSetWireFormat = false;

            /**
             * MessageOptions noStandardDescriptorAccessor.
             * @member {boolean} noStandardDescriptorAccessor
             * @memberof google.protobuf.MessageOptions
             * @instance
             */
            MessageOptions.prototype.noStandardDescriptorAccessor = false;

            /**
             * MessageOptions deprecated.
             * @member {boolean} deprecated
             * @memberof google.protobuf.MessageOptions
             * @instance
             */
            MessageOptions.prototype.deprecated = false;

            /**
             * MessageOptions mapEntry.
             * @member {boolean} mapEntry
             * @memberof google.protobuf.MessageOptions
             * @instance
             */
            MessageOptions.prototype.mapEntry = false;

            /**
             * MessageOptions uninterpretedOption.
             * @member {Array.<google.protobuf.IUninterpretedOption>} uninterpretedOption
             * @memberof google.protobuf.MessageOptions
             * @instance
             */
            MessageOptions.prototype.uninterpretedOption = $util.emptyArray;

            /**
             * Creates a new MessageOptions instance using the specified properties.
             * @function create
             * @memberof google.protobuf.MessageOptions
             * @static
             * @param {google.protobuf.IMessageOptions=} [properties] Properties to set
             * @returns {google.protobuf.MessageOptions} MessageOptions instance
             */
            MessageOptions.create = function create(properties) {
                return new MessageOptions(properties);
            };

            /**
             * Encodes the specified MessageOptions message. Does not implicitly {@link google.protobuf.MessageOptions.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.MessageOptions
             * @static
             * @param {google.protobuf.IMessageOptions} message MessageOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MessageOptions.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.messageSetWireFormat != null && Object.hasOwnProperty.call(message, "messageSetWireFormat"))
                    writer.uint32(/* id 1, wireType 0 =*/8).bool(message.messageSetWireFormat);
                if (message.noStandardDescriptorAccessor != null && Object.hasOwnProperty.call(message, "noStandardDescriptorAccessor"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.noStandardDescriptorAccessor);
                if (message.deprecated != null && Object.hasOwnProperty.call(message, "deprecated"))
                    writer.uint32(/* id 3, wireType 0 =*/24).bool(message.deprecated);
                if (message.mapEntry != null && Object.hasOwnProperty.call(message, "mapEntry"))
                    writer.uint32(/* id 7, wireType 0 =*/56).bool(message.mapEntry);
                if (message.uninterpretedOption != null && message.uninterpretedOption.length)
                    for (let i = 0; i < message.uninterpretedOption.length; ++i)
                        $root.google.protobuf.UninterpretedOption.encode(message.uninterpretedOption[i], writer.uint32(/* id 999, wireType 2 =*/7994).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified MessageOptions message, length delimited. Does not implicitly {@link google.protobuf.MessageOptions.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.MessageOptions
             * @static
             * @param {google.protobuf.IMessageOptions} message MessageOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MessageOptions.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a MessageOptions message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.MessageOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.MessageOptions} MessageOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MessageOptions.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.MessageOptions();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.messageSetWireFormat = reader.bool();
                        break;
                    case 2:
                        message.noStandardDescriptorAccessor = reader.bool();
                        break;
                    case 3:
                        message.deprecated = reader.bool();
                        break;
                    case 7:
                        message.mapEntry = reader.bool();
                        break;
                    case 999:
                        if (!(message.uninterpretedOption && message.uninterpretedOption.length))
                            message.uninterpretedOption = [];
                        message.uninterpretedOption.push($root.google.protobuf.UninterpretedOption.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a MessageOptions message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.MessageOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.MessageOptions} MessageOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MessageOptions.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a MessageOptions message.
             * @function verify
             * @memberof google.protobuf.MessageOptions
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            MessageOptions.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.messageSetWireFormat != null && message.hasOwnProperty("messageSetWireFormat"))
                    if (typeof message.messageSetWireFormat !== "boolean")
                        return "messageSetWireFormat: boolean expected";
                if (message.noStandardDescriptorAccessor != null && message.hasOwnProperty("noStandardDescriptorAccessor"))
                    if (typeof message.noStandardDescriptorAccessor !== "boolean")
                        return "noStandardDescriptorAccessor: boolean expected";
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    if (typeof message.deprecated !== "boolean")
                        return "deprecated: boolean expected";
                if (message.mapEntry != null && message.hasOwnProperty("mapEntry"))
                    if (typeof message.mapEntry !== "boolean")
                        return "mapEntry: boolean expected";
                if (message.uninterpretedOption != null && message.hasOwnProperty("uninterpretedOption")) {
                    if (!Array.isArray(message.uninterpretedOption))
                        return "uninterpretedOption: array expected";
                    for (let i = 0; i < message.uninterpretedOption.length; ++i) {
                        let error = $root.google.protobuf.UninterpretedOption.verify(message.uninterpretedOption[i]);
                        if (error)
                            return "uninterpretedOption." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a MessageOptions message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.MessageOptions
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.MessageOptions} MessageOptions
             */
            MessageOptions.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.MessageOptions)
                    return object;
                let message = new $root.google.protobuf.MessageOptions();
                if (object.messageSetWireFormat != null)
                    message.messageSetWireFormat = Boolean(object.messageSetWireFormat);
                if (object.noStandardDescriptorAccessor != null)
                    message.noStandardDescriptorAccessor = Boolean(object.noStandardDescriptorAccessor);
                if (object.deprecated != null)
                    message.deprecated = Boolean(object.deprecated);
                if (object.mapEntry != null)
                    message.mapEntry = Boolean(object.mapEntry);
                if (object.uninterpretedOption) {
                    if (!Array.isArray(object.uninterpretedOption))
                        throw TypeError(".google.protobuf.MessageOptions.uninterpretedOption: array expected");
                    message.uninterpretedOption = [];
                    for (let i = 0; i < object.uninterpretedOption.length; ++i) {
                        if (typeof object.uninterpretedOption[i] !== "object")
                            throw TypeError(".google.protobuf.MessageOptions.uninterpretedOption: object expected");
                        message.uninterpretedOption[i] = $root.google.protobuf.UninterpretedOption.fromObject(object.uninterpretedOption[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a MessageOptions message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.MessageOptions
             * @static
             * @param {google.protobuf.MessageOptions} message MessageOptions
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            MessageOptions.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.uninterpretedOption = [];
                if (options.defaults) {
                    object.messageSetWireFormat = false;
                    object.noStandardDescriptorAccessor = false;
                    object.deprecated = false;
                    object.mapEntry = false;
                }
                if (message.messageSetWireFormat != null && message.hasOwnProperty("messageSetWireFormat"))
                    object.messageSetWireFormat = message.messageSetWireFormat;
                if (message.noStandardDescriptorAccessor != null && message.hasOwnProperty("noStandardDescriptorAccessor"))
                    object.noStandardDescriptorAccessor = message.noStandardDescriptorAccessor;
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    object.deprecated = message.deprecated;
                if (message.mapEntry != null && message.hasOwnProperty("mapEntry"))
                    object.mapEntry = message.mapEntry;
                if (message.uninterpretedOption && message.uninterpretedOption.length) {
                    object.uninterpretedOption = [];
                    for (let j = 0; j < message.uninterpretedOption.length; ++j)
                        object.uninterpretedOption[j] = $root.google.protobuf.UninterpretedOption.toObject(message.uninterpretedOption[j], options);
                }
                return object;
            };

            /**
             * Converts this MessageOptions to JSON.
             * @function toJSON
             * @memberof google.protobuf.MessageOptions
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            MessageOptions.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return MessageOptions;
        })();

        protobuf.FieldOptions = (function() {

            /**
             * Properties of a FieldOptions.
             * @memberof google.protobuf
             * @interface IFieldOptions
             * @property {google.protobuf.FieldOptions.CType|null} [ctype] FieldOptions ctype
             * @property {boolean|null} [packed] FieldOptions packed
             * @property {google.protobuf.FieldOptions.JSType|null} [jstype] FieldOptions jstype
             * @property {boolean|null} [lazy] FieldOptions lazy
             * @property {boolean|null} [deprecated] FieldOptions deprecated
             * @property {boolean|null} [weak] FieldOptions weak
             * @property {Array.<google.protobuf.IUninterpretedOption>|null} [uninterpretedOption] FieldOptions uninterpretedOption
             * @property {boolean|null} [".exocore.indexed"] FieldOptions .exocore.indexed
             * @property {boolean|null} [".exocore.sorted"] FieldOptions .exocore.sorted
             * @property {boolean|null} [".exocore.text"] FieldOptions .exocore.text
             */

            /**
             * Constructs a new FieldOptions.
             * @memberof google.protobuf
             * @classdesc Represents a FieldOptions.
             * @implements IFieldOptions
             * @constructor
             * @param {google.protobuf.IFieldOptions=} [properties] Properties to set
             */
            function FieldOptions(properties) {
                this.uninterpretedOption = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * FieldOptions ctype.
             * @member {google.protobuf.FieldOptions.CType} ctype
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype.ctype = 0;

            /**
             * FieldOptions packed.
             * @member {boolean} packed
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype.packed = false;

            /**
             * FieldOptions jstype.
             * @member {google.protobuf.FieldOptions.JSType} jstype
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype.jstype = 0;

            /**
             * FieldOptions lazy.
             * @member {boolean} lazy
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype.lazy = false;

            /**
             * FieldOptions deprecated.
             * @member {boolean} deprecated
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype.deprecated = false;

            /**
             * FieldOptions weak.
             * @member {boolean} weak
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype.weak = false;

            /**
             * FieldOptions uninterpretedOption.
             * @member {Array.<google.protobuf.IUninterpretedOption>} uninterpretedOption
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype.uninterpretedOption = $util.emptyArray;

            /**
             * FieldOptions .exocore.indexed.
             * @member {boolean} .exocore.indexed
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype[".exocore.indexed"] = false;

            /**
             * FieldOptions .exocore.sorted.
             * @member {boolean} .exocore.sorted
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype[".exocore.sorted"] = false;

            /**
             * FieldOptions .exocore.text.
             * @member {boolean} .exocore.text
             * @memberof google.protobuf.FieldOptions
             * @instance
             */
            FieldOptions.prototype[".exocore.text"] = false;

            /**
             * Creates a new FieldOptions instance using the specified properties.
             * @function create
             * @memberof google.protobuf.FieldOptions
             * @static
             * @param {google.protobuf.IFieldOptions=} [properties] Properties to set
             * @returns {google.protobuf.FieldOptions} FieldOptions instance
             */
            FieldOptions.create = function create(properties) {
                return new FieldOptions(properties);
            };

            /**
             * Encodes the specified FieldOptions message. Does not implicitly {@link google.protobuf.FieldOptions.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.FieldOptions
             * @static
             * @param {google.protobuf.IFieldOptions} message FieldOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FieldOptions.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.ctype != null && Object.hasOwnProperty.call(message, "ctype"))
                    writer.uint32(/* id 1, wireType 0 =*/8).int32(message.ctype);
                if (message.packed != null && Object.hasOwnProperty.call(message, "packed"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.packed);
                if (message.deprecated != null && Object.hasOwnProperty.call(message, "deprecated"))
                    writer.uint32(/* id 3, wireType 0 =*/24).bool(message.deprecated);
                if (message.lazy != null && Object.hasOwnProperty.call(message, "lazy"))
                    writer.uint32(/* id 5, wireType 0 =*/40).bool(message.lazy);
                if (message.jstype != null && Object.hasOwnProperty.call(message, "jstype"))
                    writer.uint32(/* id 6, wireType 0 =*/48).int32(message.jstype);
                if (message.weak != null && Object.hasOwnProperty.call(message, "weak"))
                    writer.uint32(/* id 10, wireType 0 =*/80).bool(message.weak);
                if (message.uninterpretedOption != null && message.uninterpretedOption.length)
                    for (let i = 0; i < message.uninterpretedOption.length; ++i)
                        $root.google.protobuf.UninterpretedOption.encode(message.uninterpretedOption[i], writer.uint32(/* id 999, wireType 2 =*/7994).fork()).ldelim();
                if (message[".exocore.indexed"] != null && Object.hasOwnProperty.call(message, ".exocore.indexed"))
                    writer.uint32(/* id 1373, wireType 0 =*/10984).bool(message[".exocore.indexed"]);
                if (message[".exocore.sorted"] != null && Object.hasOwnProperty.call(message, ".exocore.sorted"))
                    writer.uint32(/* id 1374, wireType 0 =*/10992).bool(message[".exocore.sorted"]);
                if (message[".exocore.text"] != null && Object.hasOwnProperty.call(message, ".exocore.text"))
                    writer.uint32(/* id 1375, wireType 0 =*/11000).bool(message[".exocore.text"]);
                return writer;
            };

            /**
             * Encodes the specified FieldOptions message, length delimited. Does not implicitly {@link google.protobuf.FieldOptions.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.FieldOptions
             * @static
             * @param {google.protobuf.IFieldOptions} message FieldOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FieldOptions.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a FieldOptions message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.FieldOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.FieldOptions} FieldOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FieldOptions.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.FieldOptions();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.ctype = reader.int32();
                        break;
                    case 2:
                        message.packed = reader.bool();
                        break;
                    case 6:
                        message.jstype = reader.int32();
                        break;
                    case 5:
                        message.lazy = reader.bool();
                        break;
                    case 3:
                        message.deprecated = reader.bool();
                        break;
                    case 10:
                        message.weak = reader.bool();
                        break;
                    case 999:
                        if (!(message.uninterpretedOption && message.uninterpretedOption.length))
                            message.uninterpretedOption = [];
                        message.uninterpretedOption.push($root.google.protobuf.UninterpretedOption.decode(reader, reader.uint32()));
                        break;
                    case 1373:
                        message[".exocore.indexed"] = reader.bool();
                        break;
                    case 1374:
                        message[".exocore.sorted"] = reader.bool();
                        break;
                    case 1375:
                        message[".exocore.text"] = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a FieldOptions message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.FieldOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.FieldOptions} FieldOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FieldOptions.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a FieldOptions message.
             * @function verify
             * @memberof google.protobuf.FieldOptions
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FieldOptions.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.ctype != null && message.hasOwnProperty("ctype"))
                    switch (message.ctype) {
                    default:
                        return "ctype: enum value expected";
                    case 0:
                    case 1:
                    case 2:
                        break;
                    }
                if (message.packed != null && message.hasOwnProperty("packed"))
                    if (typeof message.packed !== "boolean")
                        return "packed: boolean expected";
                if (message.jstype != null && message.hasOwnProperty("jstype"))
                    switch (message.jstype) {
                    default:
                        return "jstype: enum value expected";
                    case 0:
                    case 1:
                    case 2:
                        break;
                    }
                if (message.lazy != null && message.hasOwnProperty("lazy"))
                    if (typeof message.lazy !== "boolean")
                        return "lazy: boolean expected";
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    if (typeof message.deprecated !== "boolean")
                        return "deprecated: boolean expected";
                if (message.weak != null && message.hasOwnProperty("weak"))
                    if (typeof message.weak !== "boolean")
                        return "weak: boolean expected";
                if (message.uninterpretedOption != null && message.hasOwnProperty("uninterpretedOption")) {
                    if (!Array.isArray(message.uninterpretedOption))
                        return "uninterpretedOption: array expected";
                    for (let i = 0; i < message.uninterpretedOption.length; ++i) {
                        let error = $root.google.protobuf.UninterpretedOption.verify(message.uninterpretedOption[i]);
                        if (error)
                            return "uninterpretedOption." + error;
                    }
                }
                if (message[".exocore.indexed"] != null && message.hasOwnProperty(".exocore.indexed"))
                    if (typeof message[".exocore.indexed"] !== "boolean")
                        return ".exocore.indexed: boolean expected";
                if (message[".exocore.sorted"] != null && message.hasOwnProperty(".exocore.sorted"))
                    if (typeof message[".exocore.sorted"] !== "boolean")
                        return ".exocore.sorted: boolean expected";
                if (message[".exocore.text"] != null && message.hasOwnProperty(".exocore.text"))
                    if (typeof message[".exocore.text"] !== "boolean")
                        return ".exocore.text: boolean expected";
                return null;
            };

            /**
             * Creates a FieldOptions message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.FieldOptions
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.FieldOptions} FieldOptions
             */
            FieldOptions.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.FieldOptions)
                    return object;
                let message = new $root.google.protobuf.FieldOptions();
                switch (object.ctype) {
                case "STRING":
                case 0:
                    message.ctype = 0;
                    break;
                case "CORD":
                case 1:
                    message.ctype = 1;
                    break;
                case "STRING_PIECE":
                case 2:
                    message.ctype = 2;
                    break;
                }
                if (object.packed != null)
                    message.packed = Boolean(object.packed);
                switch (object.jstype) {
                case "JS_NORMAL":
                case 0:
                    message.jstype = 0;
                    break;
                case "JS_STRING":
                case 1:
                    message.jstype = 1;
                    break;
                case "JS_NUMBER":
                case 2:
                    message.jstype = 2;
                    break;
                }
                if (object.lazy != null)
                    message.lazy = Boolean(object.lazy);
                if (object.deprecated != null)
                    message.deprecated = Boolean(object.deprecated);
                if (object.weak != null)
                    message.weak = Boolean(object.weak);
                if (object.uninterpretedOption) {
                    if (!Array.isArray(object.uninterpretedOption))
                        throw TypeError(".google.protobuf.FieldOptions.uninterpretedOption: array expected");
                    message.uninterpretedOption = [];
                    for (let i = 0; i < object.uninterpretedOption.length; ++i) {
                        if (typeof object.uninterpretedOption[i] !== "object")
                            throw TypeError(".google.protobuf.FieldOptions.uninterpretedOption: object expected");
                        message.uninterpretedOption[i] = $root.google.protobuf.UninterpretedOption.fromObject(object.uninterpretedOption[i]);
                    }
                }
                if (object[".exocore.indexed"] != null)
                    message[".exocore.indexed"] = Boolean(object[".exocore.indexed"]);
                if (object[".exocore.sorted"] != null)
                    message[".exocore.sorted"] = Boolean(object[".exocore.sorted"]);
                if (object[".exocore.text"] != null)
                    message[".exocore.text"] = Boolean(object[".exocore.text"]);
                return message;
            };

            /**
             * Creates a plain object from a FieldOptions message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.FieldOptions
             * @static
             * @param {google.protobuf.FieldOptions} message FieldOptions
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FieldOptions.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.uninterpretedOption = [];
                if (options.defaults) {
                    object.ctype = options.enums === String ? "STRING" : 0;
                    object.packed = false;
                    object.deprecated = false;
                    object.lazy = false;
                    object.jstype = options.enums === String ? "JS_NORMAL" : 0;
                    object.weak = false;
                    object[".exocore.indexed"] = false;
                    object[".exocore.sorted"] = false;
                    object[".exocore.text"] = false;
                }
                if (message.ctype != null && message.hasOwnProperty("ctype"))
                    object.ctype = options.enums === String ? $root.google.protobuf.FieldOptions.CType[message.ctype] : message.ctype;
                if (message.packed != null && message.hasOwnProperty("packed"))
                    object.packed = message.packed;
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    object.deprecated = message.deprecated;
                if (message.lazy != null && message.hasOwnProperty("lazy"))
                    object.lazy = message.lazy;
                if (message.jstype != null && message.hasOwnProperty("jstype"))
                    object.jstype = options.enums === String ? $root.google.protobuf.FieldOptions.JSType[message.jstype] : message.jstype;
                if (message.weak != null && message.hasOwnProperty("weak"))
                    object.weak = message.weak;
                if (message.uninterpretedOption && message.uninterpretedOption.length) {
                    object.uninterpretedOption = [];
                    for (let j = 0; j < message.uninterpretedOption.length; ++j)
                        object.uninterpretedOption[j] = $root.google.protobuf.UninterpretedOption.toObject(message.uninterpretedOption[j], options);
                }
                if (message[".exocore.indexed"] != null && message.hasOwnProperty(".exocore.indexed"))
                    object[".exocore.indexed"] = message[".exocore.indexed"];
                if (message[".exocore.sorted"] != null && message.hasOwnProperty(".exocore.sorted"))
                    object[".exocore.sorted"] = message[".exocore.sorted"];
                if (message[".exocore.text"] != null && message.hasOwnProperty(".exocore.text"))
                    object[".exocore.text"] = message[".exocore.text"];
                return object;
            };

            /**
             * Converts this FieldOptions to JSON.
             * @function toJSON
             * @memberof google.protobuf.FieldOptions
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FieldOptions.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * CType enum.
             * @name google.protobuf.FieldOptions.CType
             * @enum {number}
             * @property {number} STRING=0 STRING value
             * @property {number} CORD=1 CORD value
             * @property {number} STRING_PIECE=2 STRING_PIECE value
             */
            FieldOptions.CType = (function() {
                const valuesById = {}, values = Object.create(valuesById);
                values[valuesById[0] = "STRING"] = 0;
                values[valuesById[1] = "CORD"] = 1;
                values[valuesById[2] = "STRING_PIECE"] = 2;
                return values;
            })();

            /**
             * JSType enum.
             * @name google.protobuf.FieldOptions.JSType
             * @enum {number}
             * @property {number} JS_NORMAL=0 JS_NORMAL value
             * @property {number} JS_STRING=1 JS_STRING value
             * @property {number} JS_NUMBER=2 JS_NUMBER value
             */
            FieldOptions.JSType = (function() {
                const valuesById = {}, values = Object.create(valuesById);
                values[valuesById[0] = "JS_NORMAL"] = 0;
                values[valuesById[1] = "JS_STRING"] = 1;
                values[valuesById[2] = "JS_NUMBER"] = 2;
                return values;
            })();

            return FieldOptions;
        })();

        protobuf.OneofOptions = (function() {

            /**
             * Properties of an OneofOptions.
             * @memberof google.protobuf
             * @interface IOneofOptions
             * @property {Array.<google.protobuf.IUninterpretedOption>|null} [uninterpretedOption] OneofOptions uninterpretedOption
             */

            /**
             * Constructs a new OneofOptions.
             * @memberof google.protobuf
             * @classdesc Represents an OneofOptions.
             * @implements IOneofOptions
             * @constructor
             * @param {google.protobuf.IOneofOptions=} [properties] Properties to set
             */
            function OneofOptions(properties) {
                this.uninterpretedOption = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * OneofOptions uninterpretedOption.
             * @member {Array.<google.protobuf.IUninterpretedOption>} uninterpretedOption
             * @memberof google.protobuf.OneofOptions
             * @instance
             */
            OneofOptions.prototype.uninterpretedOption = $util.emptyArray;

            /**
             * Creates a new OneofOptions instance using the specified properties.
             * @function create
             * @memberof google.protobuf.OneofOptions
             * @static
             * @param {google.protobuf.IOneofOptions=} [properties] Properties to set
             * @returns {google.protobuf.OneofOptions} OneofOptions instance
             */
            OneofOptions.create = function create(properties) {
                return new OneofOptions(properties);
            };

            /**
             * Encodes the specified OneofOptions message. Does not implicitly {@link google.protobuf.OneofOptions.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.OneofOptions
             * @static
             * @param {google.protobuf.IOneofOptions} message OneofOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OneofOptions.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.uninterpretedOption != null && message.uninterpretedOption.length)
                    for (let i = 0; i < message.uninterpretedOption.length; ++i)
                        $root.google.protobuf.UninterpretedOption.encode(message.uninterpretedOption[i], writer.uint32(/* id 999, wireType 2 =*/7994).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified OneofOptions message, length delimited. Does not implicitly {@link google.protobuf.OneofOptions.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.OneofOptions
             * @static
             * @param {google.protobuf.IOneofOptions} message OneofOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OneofOptions.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an OneofOptions message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.OneofOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.OneofOptions} OneofOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OneofOptions.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.OneofOptions();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 999:
                        if (!(message.uninterpretedOption && message.uninterpretedOption.length))
                            message.uninterpretedOption = [];
                        message.uninterpretedOption.push($root.google.protobuf.UninterpretedOption.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an OneofOptions message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.OneofOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.OneofOptions} OneofOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OneofOptions.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an OneofOptions message.
             * @function verify
             * @memberof google.protobuf.OneofOptions
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OneofOptions.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.uninterpretedOption != null && message.hasOwnProperty("uninterpretedOption")) {
                    if (!Array.isArray(message.uninterpretedOption))
                        return "uninterpretedOption: array expected";
                    for (let i = 0; i < message.uninterpretedOption.length; ++i) {
                        let error = $root.google.protobuf.UninterpretedOption.verify(message.uninterpretedOption[i]);
                        if (error)
                            return "uninterpretedOption." + error;
                    }
                }
                return null;
            };

            /**
             * Creates an OneofOptions message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.OneofOptions
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.OneofOptions} OneofOptions
             */
            OneofOptions.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.OneofOptions)
                    return object;
                let message = new $root.google.protobuf.OneofOptions();
                if (object.uninterpretedOption) {
                    if (!Array.isArray(object.uninterpretedOption))
                        throw TypeError(".google.protobuf.OneofOptions.uninterpretedOption: array expected");
                    message.uninterpretedOption = [];
                    for (let i = 0; i < object.uninterpretedOption.length; ++i) {
                        if (typeof object.uninterpretedOption[i] !== "object")
                            throw TypeError(".google.protobuf.OneofOptions.uninterpretedOption: object expected");
                        message.uninterpretedOption[i] = $root.google.protobuf.UninterpretedOption.fromObject(object.uninterpretedOption[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from an OneofOptions message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.OneofOptions
             * @static
             * @param {google.protobuf.OneofOptions} message OneofOptions
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OneofOptions.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.uninterpretedOption = [];
                if (message.uninterpretedOption && message.uninterpretedOption.length) {
                    object.uninterpretedOption = [];
                    for (let j = 0; j < message.uninterpretedOption.length; ++j)
                        object.uninterpretedOption[j] = $root.google.protobuf.UninterpretedOption.toObject(message.uninterpretedOption[j], options);
                }
                return object;
            };

            /**
             * Converts this OneofOptions to JSON.
             * @function toJSON
             * @memberof google.protobuf.OneofOptions
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OneofOptions.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return OneofOptions;
        })();

        protobuf.EnumOptions = (function() {

            /**
             * Properties of an EnumOptions.
             * @memberof google.protobuf
             * @interface IEnumOptions
             * @property {boolean|null} [allowAlias] EnumOptions allowAlias
             * @property {boolean|null} [deprecated] EnumOptions deprecated
             * @property {Array.<google.protobuf.IUninterpretedOption>|null} [uninterpretedOption] EnumOptions uninterpretedOption
             */

            /**
             * Constructs a new EnumOptions.
             * @memberof google.protobuf
             * @classdesc Represents an EnumOptions.
             * @implements IEnumOptions
             * @constructor
             * @param {google.protobuf.IEnumOptions=} [properties] Properties to set
             */
            function EnumOptions(properties) {
                this.uninterpretedOption = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * EnumOptions allowAlias.
             * @member {boolean} allowAlias
             * @memberof google.protobuf.EnumOptions
             * @instance
             */
            EnumOptions.prototype.allowAlias = false;

            /**
             * EnumOptions deprecated.
             * @member {boolean} deprecated
             * @memberof google.protobuf.EnumOptions
             * @instance
             */
            EnumOptions.prototype.deprecated = false;

            /**
             * EnumOptions uninterpretedOption.
             * @member {Array.<google.protobuf.IUninterpretedOption>} uninterpretedOption
             * @memberof google.protobuf.EnumOptions
             * @instance
             */
            EnumOptions.prototype.uninterpretedOption = $util.emptyArray;

            /**
             * Creates a new EnumOptions instance using the specified properties.
             * @function create
             * @memberof google.protobuf.EnumOptions
             * @static
             * @param {google.protobuf.IEnumOptions=} [properties] Properties to set
             * @returns {google.protobuf.EnumOptions} EnumOptions instance
             */
            EnumOptions.create = function create(properties) {
                return new EnumOptions(properties);
            };

            /**
             * Encodes the specified EnumOptions message. Does not implicitly {@link google.protobuf.EnumOptions.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.EnumOptions
             * @static
             * @param {google.protobuf.IEnumOptions} message EnumOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EnumOptions.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.allowAlias != null && Object.hasOwnProperty.call(message, "allowAlias"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.allowAlias);
                if (message.deprecated != null && Object.hasOwnProperty.call(message, "deprecated"))
                    writer.uint32(/* id 3, wireType 0 =*/24).bool(message.deprecated);
                if (message.uninterpretedOption != null && message.uninterpretedOption.length)
                    for (let i = 0; i < message.uninterpretedOption.length; ++i)
                        $root.google.protobuf.UninterpretedOption.encode(message.uninterpretedOption[i], writer.uint32(/* id 999, wireType 2 =*/7994).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified EnumOptions message, length delimited. Does not implicitly {@link google.protobuf.EnumOptions.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.EnumOptions
             * @static
             * @param {google.protobuf.IEnumOptions} message EnumOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EnumOptions.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an EnumOptions message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.EnumOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.EnumOptions} EnumOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EnumOptions.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.EnumOptions();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 2:
                        message.allowAlias = reader.bool();
                        break;
                    case 3:
                        message.deprecated = reader.bool();
                        break;
                    case 999:
                        if (!(message.uninterpretedOption && message.uninterpretedOption.length))
                            message.uninterpretedOption = [];
                        message.uninterpretedOption.push($root.google.protobuf.UninterpretedOption.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an EnumOptions message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.EnumOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.EnumOptions} EnumOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EnumOptions.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an EnumOptions message.
             * @function verify
             * @memberof google.protobuf.EnumOptions
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            EnumOptions.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.allowAlias != null && message.hasOwnProperty("allowAlias"))
                    if (typeof message.allowAlias !== "boolean")
                        return "allowAlias: boolean expected";
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    if (typeof message.deprecated !== "boolean")
                        return "deprecated: boolean expected";
                if (message.uninterpretedOption != null && message.hasOwnProperty("uninterpretedOption")) {
                    if (!Array.isArray(message.uninterpretedOption))
                        return "uninterpretedOption: array expected";
                    for (let i = 0; i < message.uninterpretedOption.length; ++i) {
                        let error = $root.google.protobuf.UninterpretedOption.verify(message.uninterpretedOption[i]);
                        if (error)
                            return "uninterpretedOption." + error;
                    }
                }
                return null;
            };

            /**
             * Creates an EnumOptions message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.EnumOptions
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.EnumOptions} EnumOptions
             */
            EnumOptions.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.EnumOptions)
                    return object;
                let message = new $root.google.protobuf.EnumOptions();
                if (object.allowAlias != null)
                    message.allowAlias = Boolean(object.allowAlias);
                if (object.deprecated != null)
                    message.deprecated = Boolean(object.deprecated);
                if (object.uninterpretedOption) {
                    if (!Array.isArray(object.uninterpretedOption))
                        throw TypeError(".google.protobuf.EnumOptions.uninterpretedOption: array expected");
                    message.uninterpretedOption = [];
                    for (let i = 0; i < object.uninterpretedOption.length; ++i) {
                        if (typeof object.uninterpretedOption[i] !== "object")
                            throw TypeError(".google.protobuf.EnumOptions.uninterpretedOption: object expected");
                        message.uninterpretedOption[i] = $root.google.protobuf.UninterpretedOption.fromObject(object.uninterpretedOption[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from an EnumOptions message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.EnumOptions
             * @static
             * @param {google.protobuf.EnumOptions} message EnumOptions
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            EnumOptions.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.uninterpretedOption = [];
                if (options.defaults) {
                    object.allowAlias = false;
                    object.deprecated = false;
                }
                if (message.allowAlias != null && message.hasOwnProperty("allowAlias"))
                    object.allowAlias = message.allowAlias;
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    object.deprecated = message.deprecated;
                if (message.uninterpretedOption && message.uninterpretedOption.length) {
                    object.uninterpretedOption = [];
                    for (let j = 0; j < message.uninterpretedOption.length; ++j)
                        object.uninterpretedOption[j] = $root.google.protobuf.UninterpretedOption.toObject(message.uninterpretedOption[j], options);
                }
                return object;
            };

            /**
             * Converts this EnumOptions to JSON.
             * @function toJSON
             * @memberof google.protobuf.EnumOptions
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            EnumOptions.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return EnumOptions;
        })();

        protobuf.EnumValueOptions = (function() {

            /**
             * Properties of an EnumValueOptions.
             * @memberof google.protobuf
             * @interface IEnumValueOptions
             * @property {boolean|null} [deprecated] EnumValueOptions deprecated
             * @property {Array.<google.protobuf.IUninterpretedOption>|null} [uninterpretedOption] EnumValueOptions uninterpretedOption
             */

            /**
             * Constructs a new EnumValueOptions.
             * @memberof google.protobuf
             * @classdesc Represents an EnumValueOptions.
             * @implements IEnumValueOptions
             * @constructor
             * @param {google.protobuf.IEnumValueOptions=} [properties] Properties to set
             */
            function EnumValueOptions(properties) {
                this.uninterpretedOption = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * EnumValueOptions deprecated.
             * @member {boolean} deprecated
             * @memberof google.protobuf.EnumValueOptions
             * @instance
             */
            EnumValueOptions.prototype.deprecated = false;

            /**
             * EnumValueOptions uninterpretedOption.
             * @member {Array.<google.protobuf.IUninterpretedOption>} uninterpretedOption
             * @memberof google.protobuf.EnumValueOptions
             * @instance
             */
            EnumValueOptions.prototype.uninterpretedOption = $util.emptyArray;

            /**
             * Creates a new EnumValueOptions instance using the specified properties.
             * @function create
             * @memberof google.protobuf.EnumValueOptions
             * @static
             * @param {google.protobuf.IEnumValueOptions=} [properties] Properties to set
             * @returns {google.protobuf.EnumValueOptions} EnumValueOptions instance
             */
            EnumValueOptions.create = function create(properties) {
                return new EnumValueOptions(properties);
            };

            /**
             * Encodes the specified EnumValueOptions message. Does not implicitly {@link google.protobuf.EnumValueOptions.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.EnumValueOptions
             * @static
             * @param {google.protobuf.IEnumValueOptions} message EnumValueOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EnumValueOptions.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.deprecated != null && Object.hasOwnProperty.call(message, "deprecated"))
                    writer.uint32(/* id 1, wireType 0 =*/8).bool(message.deprecated);
                if (message.uninterpretedOption != null && message.uninterpretedOption.length)
                    for (let i = 0; i < message.uninterpretedOption.length; ++i)
                        $root.google.protobuf.UninterpretedOption.encode(message.uninterpretedOption[i], writer.uint32(/* id 999, wireType 2 =*/7994).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified EnumValueOptions message, length delimited. Does not implicitly {@link google.protobuf.EnumValueOptions.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.EnumValueOptions
             * @static
             * @param {google.protobuf.IEnumValueOptions} message EnumValueOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            EnumValueOptions.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an EnumValueOptions message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.EnumValueOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.EnumValueOptions} EnumValueOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EnumValueOptions.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.EnumValueOptions();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.deprecated = reader.bool();
                        break;
                    case 999:
                        if (!(message.uninterpretedOption && message.uninterpretedOption.length))
                            message.uninterpretedOption = [];
                        message.uninterpretedOption.push($root.google.protobuf.UninterpretedOption.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an EnumValueOptions message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.EnumValueOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.EnumValueOptions} EnumValueOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            EnumValueOptions.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an EnumValueOptions message.
             * @function verify
             * @memberof google.protobuf.EnumValueOptions
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            EnumValueOptions.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    if (typeof message.deprecated !== "boolean")
                        return "deprecated: boolean expected";
                if (message.uninterpretedOption != null && message.hasOwnProperty("uninterpretedOption")) {
                    if (!Array.isArray(message.uninterpretedOption))
                        return "uninterpretedOption: array expected";
                    for (let i = 0; i < message.uninterpretedOption.length; ++i) {
                        let error = $root.google.protobuf.UninterpretedOption.verify(message.uninterpretedOption[i]);
                        if (error)
                            return "uninterpretedOption." + error;
                    }
                }
                return null;
            };

            /**
             * Creates an EnumValueOptions message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.EnumValueOptions
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.EnumValueOptions} EnumValueOptions
             */
            EnumValueOptions.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.EnumValueOptions)
                    return object;
                let message = new $root.google.protobuf.EnumValueOptions();
                if (object.deprecated != null)
                    message.deprecated = Boolean(object.deprecated);
                if (object.uninterpretedOption) {
                    if (!Array.isArray(object.uninterpretedOption))
                        throw TypeError(".google.protobuf.EnumValueOptions.uninterpretedOption: array expected");
                    message.uninterpretedOption = [];
                    for (let i = 0; i < object.uninterpretedOption.length; ++i) {
                        if (typeof object.uninterpretedOption[i] !== "object")
                            throw TypeError(".google.protobuf.EnumValueOptions.uninterpretedOption: object expected");
                        message.uninterpretedOption[i] = $root.google.protobuf.UninterpretedOption.fromObject(object.uninterpretedOption[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from an EnumValueOptions message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.EnumValueOptions
             * @static
             * @param {google.protobuf.EnumValueOptions} message EnumValueOptions
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            EnumValueOptions.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.uninterpretedOption = [];
                if (options.defaults)
                    object.deprecated = false;
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    object.deprecated = message.deprecated;
                if (message.uninterpretedOption && message.uninterpretedOption.length) {
                    object.uninterpretedOption = [];
                    for (let j = 0; j < message.uninterpretedOption.length; ++j)
                        object.uninterpretedOption[j] = $root.google.protobuf.UninterpretedOption.toObject(message.uninterpretedOption[j], options);
                }
                return object;
            };

            /**
             * Converts this EnumValueOptions to JSON.
             * @function toJSON
             * @memberof google.protobuf.EnumValueOptions
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            EnumValueOptions.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return EnumValueOptions;
        })();

        protobuf.ServiceOptions = (function() {

            /**
             * Properties of a ServiceOptions.
             * @memberof google.protobuf
             * @interface IServiceOptions
             * @property {boolean|null} [deprecated] ServiceOptions deprecated
             * @property {Array.<google.protobuf.IUninterpretedOption>|null} [uninterpretedOption] ServiceOptions uninterpretedOption
             */

            /**
             * Constructs a new ServiceOptions.
             * @memberof google.protobuf
             * @classdesc Represents a ServiceOptions.
             * @implements IServiceOptions
             * @constructor
             * @param {google.protobuf.IServiceOptions=} [properties] Properties to set
             */
            function ServiceOptions(properties) {
                this.uninterpretedOption = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * ServiceOptions deprecated.
             * @member {boolean} deprecated
             * @memberof google.protobuf.ServiceOptions
             * @instance
             */
            ServiceOptions.prototype.deprecated = false;

            /**
             * ServiceOptions uninterpretedOption.
             * @member {Array.<google.protobuf.IUninterpretedOption>} uninterpretedOption
             * @memberof google.protobuf.ServiceOptions
             * @instance
             */
            ServiceOptions.prototype.uninterpretedOption = $util.emptyArray;

            /**
             * Creates a new ServiceOptions instance using the specified properties.
             * @function create
             * @memberof google.protobuf.ServiceOptions
             * @static
             * @param {google.protobuf.IServiceOptions=} [properties] Properties to set
             * @returns {google.protobuf.ServiceOptions} ServiceOptions instance
             */
            ServiceOptions.create = function create(properties) {
                return new ServiceOptions(properties);
            };

            /**
             * Encodes the specified ServiceOptions message. Does not implicitly {@link google.protobuf.ServiceOptions.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.ServiceOptions
             * @static
             * @param {google.protobuf.IServiceOptions} message ServiceOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ServiceOptions.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.deprecated != null && Object.hasOwnProperty.call(message, "deprecated"))
                    writer.uint32(/* id 33, wireType 0 =*/264).bool(message.deprecated);
                if (message.uninterpretedOption != null && message.uninterpretedOption.length)
                    for (let i = 0; i < message.uninterpretedOption.length; ++i)
                        $root.google.protobuf.UninterpretedOption.encode(message.uninterpretedOption[i], writer.uint32(/* id 999, wireType 2 =*/7994).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified ServiceOptions message, length delimited. Does not implicitly {@link google.protobuf.ServiceOptions.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.ServiceOptions
             * @static
             * @param {google.protobuf.IServiceOptions} message ServiceOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ServiceOptions.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a ServiceOptions message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.ServiceOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.ServiceOptions} ServiceOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ServiceOptions.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.ServiceOptions();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 33:
                        message.deprecated = reader.bool();
                        break;
                    case 999:
                        if (!(message.uninterpretedOption && message.uninterpretedOption.length))
                            message.uninterpretedOption = [];
                        message.uninterpretedOption.push($root.google.protobuf.UninterpretedOption.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a ServiceOptions message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.ServiceOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.ServiceOptions} ServiceOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ServiceOptions.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a ServiceOptions message.
             * @function verify
             * @memberof google.protobuf.ServiceOptions
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ServiceOptions.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    if (typeof message.deprecated !== "boolean")
                        return "deprecated: boolean expected";
                if (message.uninterpretedOption != null && message.hasOwnProperty("uninterpretedOption")) {
                    if (!Array.isArray(message.uninterpretedOption))
                        return "uninterpretedOption: array expected";
                    for (let i = 0; i < message.uninterpretedOption.length; ++i) {
                        let error = $root.google.protobuf.UninterpretedOption.verify(message.uninterpretedOption[i]);
                        if (error)
                            return "uninterpretedOption." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a ServiceOptions message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.ServiceOptions
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.ServiceOptions} ServiceOptions
             */
            ServiceOptions.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.ServiceOptions)
                    return object;
                let message = new $root.google.protobuf.ServiceOptions();
                if (object.deprecated != null)
                    message.deprecated = Boolean(object.deprecated);
                if (object.uninterpretedOption) {
                    if (!Array.isArray(object.uninterpretedOption))
                        throw TypeError(".google.protobuf.ServiceOptions.uninterpretedOption: array expected");
                    message.uninterpretedOption = [];
                    for (let i = 0; i < object.uninterpretedOption.length; ++i) {
                        if (typeof object.uninterpretedOption[i] !== "object")
                            throw TypeError(".google.protobuf.ServiceOptions.uninterpretedOption: object expected");
                        message.uninterpretedOption[i] = $root.google.protobuf.UninterpretedOption.fromObject(object.uninterpretedOption[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a ServiceOptions message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.ServiceOptions
             * @static
             * @param {google.protobuf.ServiceOptions} message ServiceOptions
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ServiceOptions.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.uninterpretedOption = [];
                if (options.defaults)
                    object.deprecated = false;
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    object.deprecated = message.deprecated;
                if (message.uninterpretedOption && message.uninterpretedOption.length) {
                    object.uninterpretedOption = [];
                    for (let j = 0; j < message.uninterpretedOption.length; ++j)
                        object.uninterpretedOption[j] = $root.google.protobuf.UninterpretedOption.toObject(message.uninterpretedOption[j], options);
                }
                return object;
            };

            /**
             * Converts this ServiceOptions to JSON.
             * @function toJSON
             * @memberof google.protobuf.ServiceOptions
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ServiceOptions.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return ServiceOptions;
        })();

        protobuf.MethodOptions = (function() {

            /**
             * Properties of a MethodOptions.
             * @memberof google.protobuf
             * @interface IMethodOptions
             * @property {boolean|null} [deprecated] MethodOptions deprecated
             * @property {Array.<google.protobuf.IUninterpretedOption>|null} [uninterpretedOption] MethodOptions uninterpretedOption
             */

            /**
             * Constructs a new MethodOptions.
             * @memberof google.protobuf
             * @classdesc Represents a MethodOptions.
             * @implements IMethodOptions
             * @constructor
             * @param {google.protobuf.IMethodOptions=} [properties] Properties to set
             */
            function MethodOptions(properties) {
                this.uninterpretedOption = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * MethodOptions deprecated.
             * @member {boolean} deprecated
             * @memberof google.protobuf.MethodOptions
             * @instance
             */
            MethodOptions.prototype.deprecated = false;

            /**
             * MethodOptions uninterpretedOption.
             * @member {Array.<google.protobuf.IUninterpretedOption>} uninterpretedOption
             * @memberof google.protobuf.MethodOptions
             * @instance
             */
            MethodOptions.prototype.uninterpretedOption = $util.emptyArray;

            /**
             * Creates a new MethodOptions instance using the specified properties.
             * @function create
             * @memberof google.protobuf.MethodOptions
             * @static
             * @param {google.protobuf.IMethodOptions=} [properties] Properties to set
             * @returns {google.protobuf.MethodOptions} MethodOptions instance
             */
            MethodOptions.create = function create(properties) {
                return new MethodOptions(properties);
            };

            /**
             * Encodes the specified MethodOptions message. Does not implicitly {@link google.protobuf.MethodOptions.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.MethodOptions
             * @static
             * @param {google.protobuf.IMethodOptions} message MethodOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MethodOptions.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.deprecated != null && Object.hasOwnProperty.call(message, "deprecated"))
                    writer.uint32(/* id 33, wireType 0 =*/264).bool(message.deprecated);
                if (message.uninterpretedOption != null && message.uninterpretedOption.length)
                    for (let i = 0; i < message.uninterpretedOption.length; ++i)
                        $root.google.protobuf.UninterpretedOption.encode(message.uninterpretedOption[i], writer.uint32(/* id 999, wireType 2 =*/7994).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified MethodOptions message, length delimited. Does not implicitly {@link google.protobuf.MethodOptions.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.MethodOptions
             * @static
             * @param {google.protobuf.IMethodOptions} message MethodOptions message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            MethodOptions.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a MethodOptions message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.MethodOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.MethodOptions} MethodOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MethodOptions.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.MethodOptions();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 33:
                        message.deprecated = reader.bool();
                        break;
                    case 999:
                        if (!(message.uninterpretedOption && message.uninterpretedOption.length))
                            message.uninterpretedOption = [];
                        message.uninterpretedOption.push($root.google.protobuf.UninterpretedOption.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a MethodOptions message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.MethodOptions
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.MethodOptions} MethodOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            MethodOptions.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a MethodOptions message.
             * @function verify
             * @memberof google.protobuf.MethodOptions
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            MethodOptions.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    if (typeof message.deprecated !== "boolean")
                        return "deprecated: boolean expected";
                if (message.uninterpretedOption != null && message.hasOwnProperty("uninterpretedOption")) {
                    if (!Array.isArray(message.uninterpretedOption))
                        return "uninterpretedOption: array expected";
                    for (let i = 0; i < message.uninterpretedOption.length; ++i) {
                        let error = $root.google.protobuf.UninterpretedOption.verify(message.uninterpretedOption[i]);
                        if (error)
                            return "uninterpretedOption." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a MethodOptions message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.MethodOptions
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.MethodOptions} MethodOptions
             */
            MethodOptions.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.MethodOptions)
                    return object;
                let message = new $root.google.protobuf.MethodOptions();
                if (object.deprecated != null)
                    message.deprecated = Boolean(object.deprecated);
                if (object.uninterpretedOption) {
                    if (!Array.isArray(object.uninterpretedOption))
                        throw TypeError(".google.protobuf.MethodOptions.uninterpretedOption: array expected");
                    message.uninterpretedOption = [];
                    for (let i = 0; i < object.uninterpretedOption.length; ++i) {
                        if (typeof object.uninterpretedOption[i] !== "object")
                            throw TypeError(".google.protobuf.MethodOptions.uninterpretedOption: object expected");
                        message.uninterpretedOption[i] = $root.google.protobuf.UninterpretedOption.fromObject(object.uninterpretedOption[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a MethodOptions message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.MethodOptions
             * @static
             * @param {google.protobuf.MethodOptions} message MethodOptions
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            MethodOptions.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.uninterpretedOption = [];
                if (options.defaults)
                    object.deprecated = false;
                if (message.deprecated != null && message.hasOwnProperty("deprecated"))
                    object.deprecated = message.deprecated;
                if (message.uninterpretedOption && message.uninterpretedOption.length) {
                    object.uninterpretedOption = [];
                    for (let j = 0; j < message.uninterpretedOption.length; ++j)
                        object.uninterpretedOption[j] = $root.google.protobuf.UninterpretedOption.toObject(message.uninterpretedOption[j], options);
                }
                return object;
            };

            /**
             * Converts this MethodOptions to JSON.
             * @function toJSON
             * @memberof google.protobuf.MethodOptions
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            MethodOptions.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            return MethodOptions;
        })();

        protobuf.UninterpretedOption = (function() {

            /**
             * Properties of an UninterpretedOption.
             * @memberof google.protobuf
             * @interface IUninterpretedOption
             * @property {Array.<google.protobuf.UninterpretedOption.INamePart>|null} [name] UninterpretedOption name
             * @property {string|null} [identifierValue] UninterpretedOption identifierValue
             * @property {number|Long|null} [positiveIntValue] UninterpretedOption positiveIntValue
             * @property {number|Long|null} [negativeIntValue] UninterpretedOption negativeIntValue
             * @property {number|null} [doubleValue] UninterpretedOption doubleValue
             * @property {Uint8Array|null} [stringValue] UninterpretedOption stringValue
             * @property {string|null} [aggregateValue] UninterpretedOption aggregateValue
             */

            /**
             * Constructs a new UninterpretedOption.
             * @memberof google.protobuf
             * @classdesc Represents an UninterpretedOption.
             * @implements IUninterpretedOption
             * @constructor
             * @param {google.protobuf.IUninterpretedOption=} [properties] Properties to set
             */
            function UninterpretedOption(properties) {
                this.name = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * UninterpretedOption name.
             * @member {Array.<google.protobuf.UninterpretedOption.INamePart>} name
             * @memberof google.protobuf.UninterpretedOption
             * @instance
             */
            UninterpretedOption.prototype.name = $util.emptyArray;

            /**
             * UninterpretedOption identifierValue.
             * @member {string} identifierValue
             * @memberof google.protobuf.UninterpretedOption
             * @instance
             */
            UninterpretedOption.prototype.identifierValue = "";

            /**
             * UninterpretedOption positiveIntValue.
             * @member {number|Long} positiveIntValue
             * @memberof google.protobuf.UninterpretedOption
             * @instance
             */
            UninterpretedOption.prototype.positiveIntValue = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

            /**
             * UninterpretedOption negativeIntValue.
             * @member {number|Long} negativeIntValue
             * @memberof google.protobuf.UninterpretedOption
             * @instance
             */
            UninterpretedOption.prototype.negativeIntValue = $util.Long ? $util.Long.fromBits(0,0,false) : 0;

            /**
             * UninterpretedOption doubleValue.
             * @member {number} doubleValue
             * @memberof google.protobuf.UninterpretedOption
             * @instance
             */
            UninterpretedOption.prototype.doubleValue = 0;

            /**
             * UninterpretedOption stringValue.
             * @member {Uint8Array} stringValue
             * @memberof google.protobuf.UninterpretedOption
             * @instance
             */
            UninterpretedOption.prototype.stringValue = $util.newBuffer([]);

            /**
             * UninterpretedOption aggregateValue.
             * @member {string} aggregateValue
             * @memberof google.protobuf.UninterpretedOption
             * @instance
             */
            UninterpretedOption.prototype.aggregateValue = "";

            /**
             * Creates a new UninterpretedOption instance using the specified properties.
             * @function create
             * @memberof google.protobuf.UninterpretedOption
             * @static
             * @param {google.protobuf.IUninterpretedOption=} [properties] Properties to set
             * @returns {google.protobuf.UninterpretedOption} UninterpretedOption instance
             */
            UninterpretedOption.create = function create(properties) {
                return new UninterpretedOption(properties);
            };

            /**
             * Encodes the specified UninterpretedOption message. Does not implicitly {@link google.protobuf.UninterpretedOption.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.UninterpretedOption
             * @static
             * @param {google.protobuf.IUninterpretedOption} message UninterpretedOption message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            UninterpretedOption.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.name != null && message.name.length)
                    for (let i = 0; i < message.name.length; ++i)
                        $root.google.protobuf.UninterpretedOption.NamePart.encode(message.name[i], writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.identifierValue != null && Object.hasOwnProperty.call(message, "identifierValue"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.identifierValue);
                if (message.positiveIntValue != null && Object.hasOwnProperty.call(message, "positiveIntValue"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.positiveIntValue);
                if (message.negativeIntValue != null && Object.hasOwnProperty.call(message, "negativeIntValue"))
                    writer.uint32(/* id 5, wireType 0 =*/40).int64(message.negativeIntValue);
                if (message.doubleValue != null && Object.hasOwnProperty.call(message, "doubleValue"))
                    writer.uint32(/* id 6, wireType 1 =*/49).double(message.doubleValue);
                if (message.stringValue != null && Object.hasOwnProperty.call(message, "stringValue"))
                    writer.uint32(/* id 7, wireType 2 =*/58).bytes(message.stringValue);
                if (message.aggregateValue != null && Object.hasOwnProperty.call(message, "aggregateValue"))
                    writer.uint32(/* id 8, wireType 2 =*/66).string(message.aggregateValue);
                return writer;
            };

            /**
             * Encodes the specified UninterpretedOption message, length delimited. Does not implicitly {@link google.protobuf.UninterpretedOption.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.UninterpretedOption
             * @static
             * @param {google.protobuf.IUninterpretedOption} message UninterpretedOption message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            UninterpretedOption.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes an UninterpretedOption message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.UninterpretedOption
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.UninterpretedOption} UninterpretedOption
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            UninterpretedOption.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.UninterpretedOption();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 2:
                        if (!(message.name && message.name.length))
                            message.name = [];
                        message.name.push($root.google.protobuf.UninterpretedOption.NamePart.decode(reader, reader.uint32()));
                        break;
                    case 3:
                        message.identifierValue = reader.string();
                        break;
                    case 4:
                        message.positiveIntValue = reader.uint64();
                        break;
                    case 5:
                        message.negativeIntValue = reader.int64();
                        break;
                    case 6:
                        message.doubleValue = reader.double();
                        break;
                    case 7:
                        message.stringValue = reader.bytes();
                        break;
                    case 8:
                        message.aggregateValue = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes an UninterpretedOption message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.UninterpretedOption
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.UninterpretedOption} UninterpretedOption
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            UninterpretedOption.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies an UninterpretedOption message.
             * @function verify
             * @memberof google.protobuf.UninterpretedOption
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            UninterpretedOption.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.name != null && message.hasOwnProperty("name")) {
                    if (!Array.isArray(message.name))
                        return "name: array expected";
                    for (let i = 0; i < message.name.length; ++i) {
                        let error = $root.google.protobuf.UninterpretedOption.NamePart.verify(message.name[i]);
                        if (error)
                            return "name." + error;
                    }
                }
                if (message.identifierValue != null && message.hasOwnProperty("identifierValue"))
                    if (!$util.isString(message.identifierValue))
                        return "identifierValue: string expected";
                if (message.positiveIntValue != null && message.hasOwnProperty("positiveIntValue"))
                    if (!$util.isInteger(message.positiveIntValue) && !(message.positiveIntValue && $util.isInteger(message.positiveIntValue.low) && $util.isInteger(message.positiveIntValue.high)))
                        return "positiveIntValue: integer|Long expected";
                if (message.negativeIntValue != null && message.hasOwnProperty("negativeIntValue"))
                    if (!$util.isInteger(message.negativeIntValue) && !(message.negativeIntValue && $util.isInteger(message.negativeIntValue.low) && $util.isInteger(message.negativeIntValue.high)))
                        return "negativeIntValue: integer|Long expected";
                if (message.doubleValue != null && message.hasOwnProperty("doubleValue"))
                    if (typeof message.doubleValue !== "number")
                        return "doubleValue: number expected";
                if (message.stringValue != null && message.hasOwnProperty("stringValue"))
                    if (!(message.stringValue && typeof message.stringValue.length === "number" || $util.isString(message.stringValue)))
                        return "stringValue: buffer expected";
                if (message.aggregateValue != null && message.hasOwnProperty("aggregateValue"))
                    if (!$util.isString(message.aggregateValue))
                        return "aggregateValue: string expected";
                return null;
            };

            /**
             * Creates an UninterpretedOption message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.UninterpretedOption
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.UninterpretedOption} UninterpretedOption
             */
            UninterpretedOption.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.UninterpretedOption)
                    return object;
                let message = new $root.google.protobuf.UninterpretedOption();
                if (object.name) {
                    if (!Array.isArray(object.name))
                        throw TypeError(".google.protobuf.UninterpretedOption.name: array expected");
                    message.name = [];
                    for (let i = 0; i < object.name.length; ++i) {
                        if (typeof object.name[i] !== "object")
                            throw TypeError(".google.protobuf.UninterpretedOption.name: object expected");
                        message.name[i] = $root.google.protobuf.UninterpretedOption.NamePart.fromObject(object.name[i]);
                    }
                }
                if (object.identifierValue != null)
                    message.identifierValue = String(object.identifierValue);
                if (object.positiveIntValue != null)
                    if ($util.Long)
                        (message.positiveIntValue = $util.Long.fromValue(object.positiveIntValue)).unsigned = true;
                    else if (typeof object.positiveIntValue === "string")
                        message.positiveIntValue = parseInt(object.positiveIntValue, 10);
                    else if (typeof object.positiveIntValue === "number")
                        message.positiveIntValue = object.positiveIntValue;
                    else if (typeof object.positiveIntValue === "object")
                        message.positiveIntValue = new $util.LongBits(object.positiveIntValue.low >>> 0, object.positiveIntValue.high >>> 0).toNumber(true);
                if (object.negativeIntValue != null)
                    if ($util.Long)
                        (message.negativeIntValue = $util.Long.fromValue(object.negativeIntValue)).unsigned = false;
                    else if (typeof object.negativeIntValue === "string")
                        message.negativeIntValue = parseInt(object.negativeIntValue, 10);
                    else if (typeof object.negativeIntValue === "number")
                        message.negativeIntValue = object.negativeIntValue;
                    else if (typeof object.negativeIntValue === "object")
                        message.negativeIntValue = new $util.LongBits(object.negativeIntValue.low >>> 0, object.negativeIntValue.high >>> 0).toNumber();
                if (object.doubleValue != null)
                    message.doubleValue = Number(object.doubleValue);
                if (object.stringValue != null)
                    if (typeof object.stringValue === "string")
                        $util.base64.decode(object.stringValue, message.stringValue = $util.newBuffer($util.base64.length(object.stringValue)), 0);
                    else if (object.stringValue.length)
                        message.stringValue = object.stringValue;
                if (object.aggregateValue != null)
                    message.aggregateValue = String(object.aggregateValue);
                return message;
            };

            /**
             * Creates a plain object from an UninterpretedOption message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.UninterpretedOption
             * @static
             * @param {google.protobuf.UninterpretedOption} message UninterpretedOption
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            UninterpretedOption.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.name = [];
                if (options.defaults) {
                    object.identifierValue = "";
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, true);
                        object.positiveIntValue = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.positiveIntValue = options.longs === String ? "0" : 0;
                    if ($util.Long) {
                        let long = new $util.Long(0, 0, false);
                        object.negativeIntValue = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.negativeIntValue = options.longs === String ? "0" : 0;
                    object.doubleValue = 0;
                    if (options.bytes === String)
                        object.stringValue = "";
                    else {
                        object.stringValue = [];
                        if (options.bytes !== Array)
                            object.stringValue = $util.newBuffer(object.stringValue);
                    }
                    object.aggregateValue = "";
                }
                if (message.name && message.name.length) {
                    object.name = [];
                    for (let j = 0; j < message.name.length; ++j)
                        object.name[j] = $root.google.protobuf.UninterpretedOption.NamePart.toObject(message.name[j], options);
                }
                if (message.identifierValue != null && message.hasOwnProperty("identifierValue"))
                    object.identifierValue = message.identifierValue;
                if (message.positiveIntValue != null && message.hasOwnProperty("positiveIntValue"))
                    if (typeof message.positiveIntValue === "number")
                        object.positiveIntValue = options.longs === String ? String(message.positiveIntValue) : message.positiveIntValue;
                    else
                        object.positiveIntValue = options.longs === String ? $util.Long.prototype.toString.call(message.positiveIntValue) : options.longs === Number ? new $util.LongBits(message.positiveIntValue.low >>> 0, message.positiveIntValue.high >>> 0).toNumber(true) : message.positiveIntValue;
                if (message.negativeIntValue != null && message.hasOwnProperty("negativeIntValue"))
                    if (typeof message.negativeIntValue === "number")
                        object.negativeIntValue = options.longs === String ? String(message.negativeIntValue) : message.negativeIntValue;
                    else
                        object.negativeIntValue = options.longs === String ? $util.Long.prototype.toString.call(message.negativeIntValue) : options.longs === Number ? new $util.LongBits(message.negativeIntValue.low >>> 0, message.negativeIntValue.high >>> 0).toNumber() : message.negativeIntValue;
                if (message.doubleValue != null && message.hasOwnProperty("doubleValue"))
                    object.doubleValue = options.json && !isFinite(message.doubleValue) ? String(message.doubleValue) : message.doubleValue;
                if (message.stringValue != null && message.hasOwnProperty("stringValue"))
                    object.stringValue = options.bytes === String ? $util.base64.encode(message.stringValue, 0, message.stringValue.length) : options.bytes === Array ? Array.prototype.slice.call(message.stringValue) : message.stringValue;
                if (message.aggregateValue != null && message.hasOwnProperty("aggregateValue"))
                    object.aggregateValue = message.aggregateValue;
                return object;
            };

            /**
             * Converts this UninterpretedOption to JSON.
             * @function toJSON
             * @memberof google.protobuf.UninterpretedOption
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            UninterpretedOption.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            UninterpretedOption.NamePart = (function() {

                /**
                 * Properties of a NamePart.
                 * @memberof google.protobuf.UninterpretedOption
                 * @interface INamePart
                 * @property {string} namePart NamePart namePart
                 * @property {boolean} isExtension NamePart isExtension
                 */

                /**
                 * Constructs a new NamePart.
                 * @memberof google.protobuf.UninterpretedOption
                 * @classdesc Represents a NamePart.
                 * @implements INamePart
                 * @constructor
                 * @param {google.protobuf.UninterpretedOption.INamePart=} [properties] Properties to set
                 */
                function NamePart(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * NamePart namePart.
                 * @member {string} namePart
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @instance
                 */
                NamePart.prototype.namePart = "";

                /**
                 * NamePart isExtension.
                 * @member {boolean} isExtension
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @instance
                 */
                NamePart.prototype.isExtension = false;

                /**
                 * Creates a new NamePart instance using the specified properties.
                 * @function create
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @static
                 * @param {google.protobuf.UninterpretedOption.INamePart=} [properties] Properties to set
                 * @returns {google.protobuf.UninterpretedOption.NamePart} NamePart instance
                 */
                NamePart.create = function create(properties) {
                    return new NamePart(properties);
                };

                /**
                 * Encodes the specified NamePart message. Does not implicitly {@link google.protobuf.UninterpretedOption.NamePart.verify|verify} messages.
                 * @function encode
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @static
                 * @param {google.protobuf.UninterpretedOption.INamePart} message NamePart message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                NamePart.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.namePart);
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.isExtension);
                    return writer;
                };

                /**
                 * Encodes the specified NamePart message, length delimited. Does not implicitly {@link google.protobuf.UninterpretedOption.NamePart.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @static
                 * @param {google.protobuf.UninterpretedOption.INamePart} message NamePart message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                NamePart.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a NamePart message from the specified reader or buffer.
                 * @function decode
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {google.protobuf.UninterpretedOption.NamePart} NamePart
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                NamePart.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.UninterpretedOption.NamePart();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1:
                            message.namePart = reader.string();
                            break;
                        case 2:
                            message.isExtension = reader.bool();
                            break;
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    if (!message.hasOwnProperty("namePart"))
                        throw $util.ProtocolError("missing required 'namePart'", { instance: message });
                    if (!message.hasOwnProperty("isExtension"))
                        throw $util.ProtocolError("missing required 'isExtension'", { instance: message });
                    return message;
                };

                /**
                 * Decodes a NamePart message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {google.protobuf.UninterpretedOption.NamePart} NamePart
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                NamePart.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a NamePart message.
                 * @function verify
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                NamePart.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (!$util.isString(message.namePart))
                        return "namePart: string expected";
                    if (typeof message.isExtension !== "boolean")
                        return "isExtension: boolean expected";
                    return null;
                };

                /**
                 * Creates a NamePart message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {google.protobuf.UninterpretedOption.NamePart} NamePart
                 */
                NamePart.fromObject = function fromObject(object) {
                    if (object instanceof $root.google.protobuf.UninterpretedOption.NamePart)
                        return object;
                    let message = new $root.google.protobuf.UninterpretedOption.NamePart();
                    if (object.namePart != null)
                        message.namePart = String(object.namePart);
                    if (object.isExtension != null)
                        message.isExtension = Boolean(object.isExtension);
                    return message;
                };

                /**
                 * Creates a plain object from a NamePart message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @static
                 * @param {google.protobuf.UninterpretedOption.NamePart} message NamePart
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                NamePart.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.namePart = "";
                        object.isExtension = false;
                    }
                    if (message.namePart != null && message.hasOwnProperty("namePart"))
                        object.namePart = message.namePart;
                    if (message.isExtension != null && message.hasOwnProperty("isExtension"))
                        object.isExtension = message.isExtension;
                    return object;
                };

                /**
                 * Converts this NamePart to JSON.
                 * @function toJSON
                 * @memberof google.protobuf.UninterpretedOption.NamePart
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                NamePart.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                return NamePart;
            })();

            return UninterpretedOption;
        })();

        protobuf.SourceCodeInfo = (function() {

            /**
             * Properties of a SourceCodeInfo.
             * @memberof google.protobuf
             * @interface ISourceCodeInfo
             * @property {Array.<google.protobuf.SourceCodeInfo.ILocation>|null} [location] SourceCodeInfo location
             */

            /**
             * Constructs a new SourceCodeInfo.
             * @memberof google.protobuf
             * @classdesc Represents a SourceCodeInfo.
             * @implements ISourceCodeInfo
             * @constructor
             * @param {google.protobuf.ISourceCodeInfo=} [properties] Properties to set
             */
            function SourceCodeInfo(properties) {
                this.location = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * SourceCodeInfo location.
             * @member {Array.<google.protobuf.SourceCodeInfo.ILocation>} location
             * @memberof google.protobuf.SourceCodeInfo
             * @instance
             */
            SourceCodeInfo.prototype.location = $util.emptyArray;

            /**
             * Creates a new SourceCodeInfo instance using the specified properties.
             * @function create
             * @memberof google.protobuf.SourceCodeInfo
             * @static
             * @param {google.protobuf.ISourceCodeInfo=} [properties] Properties to set
             * @returns {google.protobuf.SourceCodeInfo} SourceCodeInfo instance
             */
            SourceCodeInfo.create = function create(properties) {
                return new SourceCodeInfo(properties);
            };

            /**
             * Encodes the specified SourceCodeInfo message. Does not implicitly {@link google.protobuf.SourceCodeInfo.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.SourceCodeInfo
             * @static
             * @param {google.protobuf.ISourceCodeInfo} message SourceCodeInfo message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            SourceCodeInfo.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.location != null && message.location.length)
                    for (let i = 0; i < message.location.length; ++i)
                        $root.google.protobuf.SourceCodeInfo.Location.encode(message.location[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified SourceCodeInfo message, length delimited. Does not implicitly {@link google.protobuf.SourceCodeInfo.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.SourceCodeInfo
             * @static
             * @param {google.protobuf.ISourceCodeInfo} message SourceCodeInfo message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            SourceCodeInfo.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a SourceCodeInfo message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.SourceCodeInfo
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.SourceCodeInfo} SourceCodeInfo
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            SourceCodeInfo.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.SourceCodeInfo();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.location && message.location.length))
                            message.location = [];
                        message.location.push($root.google.protobuf.SourceCodeInfo.Location.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a SourceCodeInfo message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.SourceCodeInfo
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.SourceCodeInfo} SourceCodeInfo
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            SourceCodeInfo.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a SourceCodeInfo message.
             * @function verify
             * @memberof google.protobuf.SourceCodeInfo
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            SourceCodeInfo.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.location != null && message.hasOwnProperty("location")) {
                    if (!Array.isArray(message.location))
                        return "location: array expected";
                    for (let i = 0; i < message.location.length; ++i) {
                        let error = $root.google.protobuf.SourceCodeInfo.Location.verify(message.location[i]);
                        if (error)
                            return "location." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a SourceCodeInfo message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.SourceCodeInfo
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.SourceCodeInfo} SourceCodeInfo
             */
            SourceCodeInfo.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.SourceCodeInfo)
                    return object;
                let message = new $root.google.protobuf.SourceCodeInfo();
                if (object.location) {
                    if (!Array.isArray(object.location))
                        throw TypeError(".google.protobuf.SourceCodeInfo.location: array expected");
                    message.location = [];
                    for (let i = 0; i < object.location.length; ++i) {
                        if (typeof object.location[i] !== "object")
                            throw TypeError(".google.protobuf.SourceCodeInfo.location: object expected");
                        message.location[i] = $root.google.protobuf.SourceCodeInfo.Location.fromObject(object.location[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a SourceCodeInfo message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.SourceCodeInfo
             * @static
             * @param {google.protobuf.SourceCodeInfo} message SourceCodeInfo
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            SourceCodeInfo.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.location = [];
                if (message.location && message.location.length) {
                    object.location = [];
                    for (let j = 0; j < message.location.length; ++j)
                        object.location[j] = $root.google.protobuf.SourceCodeInfo.Location.toObject(message.location[j], options);
                }
                return object;
            };

            /**
             * Converts this SourceCodeInfo to JSON.
             * @function toJSON
             * @memberof google.protobuf.SourceCodeInfo
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            SourceCodeInfo.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            SourceCodeInfo.Location = (function() {

                /**
                 * Properties of a Location.
                 * @memberof google.protobuf.SourceCodeInfo
                 * @interface ILocation
                 * @property {Array.<number>|null} [path] Location path
                 * @property {Array.<number>|null} [span] Location span
                 * @property {string|null} [leadingComments] Location leadingComments
                 * @property {string|null} [trailingComments] Location trailingComments
                 * @property {Array.<string>|null} [leadingDetachedComments] Location leadingDetachedComments
                 */

                /**
                 * Constructs a new Location.
                 * @memberof google.protobuf.SourceCodeInfo
                 * @classdesc Represents a Location.
                 * @implements ILocation
                 * @constructor
                 * @param {google.protobuf.SourceCodeInfo.ILocation=} [properties] Properties to set
                 */
                function Location(properties) {
                    this.path = [];
                    this.span = [];
                    this.leadingDetachedComments = [];
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Location path.
                 * @member {Array.<number>} path
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @instance
                 */
                Location.prototype.path = $util.emptyArray;

                /**
                 * Location span.
                 * @member {Array.<number>} span
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @instance
                 */
                Location.prototype.span = $util.emptyArray;

                /**
                 * Location leadingComments.
                 * @member {string} leadingComments
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @instance
                 */
                Location.prototype.leadingComments = "";

                /**
                 * Location trailingComments.
                 * @member {string} trailingComments
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @instance
                 */
                Location.prototype.trailingComments = "";

                /**
                 * Location leadingDetachedComments.
                 * @member {Array.<string>} leadingDetachedComments
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @instance
                 */
                Location.prototype.leadingDetachedComments = $util.emptyArray;

                /**
                 * Creates a new Location instance using the specified properties.
                 * @function create
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @static
                 * @param {google.protobuf.SourceCodeInfo.ILocation=} [properties] Properties to set
                 * @returns {google.protobuf.SourceCodeInfo.Location} Location instance
                 */
                Location.create = function create(properties) {
                    return new Location(properties);
                };

                /**
                 * Encodes the specified Location message. Does not implicitly {@link google.protobuf.SourceCodeInfo.Location.verify|verify} messages.
                 * @function encode
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @static
                 * @param {google.protobuf.SourceCodeInfo.ILocation} message Location message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Location.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.path != null && message.path.length) {
                        writer.uint32(/* id 1, wireType 2 =*/10).fork();
                        for (let i = 0; i < message.path.length; ++i)
                            writer.int32(message.path[i]);
                        writer.ldelim();
                    }
                    if (message.span != null && message.span.length) {
                        writer.uint32(/* id 2, wireType 2 =*/18).fork();
                        for (let i = 0; i < message.span.length; ++i)
                            writer.int32(message.span[i]);
                        writer.ldelim();
                    }
                    if (message.leadingComments != null && Object.hasOwnProperty.call(message, "leadingComments"))
                        writer.uint32(/* id 3, wireType 2 =*/26).string(message.leadingComments);
                    if (message.trailingComments != null && Object.hasOwnProperty.call(message, "trailingComments"))
                        writer.uint32(/* id 4, wireType 2 =*/34).string(message.trailingComments);
                    if (message.leadingDetachedComments != null && message.leadingDetachedComments.length)
                        for (let i = 0; i < message.leadingDetachedComments.length; ++i)
                            writer.uint32(/* id 6, wireType 2 =*/50).string(message.leadingDetachedComments[i]);
                    return writer;
                };

                /**
                 * Encodes the specified Location message, length delimited. Does not implicitly {@link google.protobuf.SourceCodeInfo.Location.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @static
                 * @param {google.protobuf.SourceCodeInfo.ILocation} message Location message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Location.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a Location message from the specified reader or buffer.
                 * @function decode
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {google.protobuf.SourceCodeInfo.Location} Location
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Location.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.SourceCodeInfo.Location();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1:
                            if (!(message.path && message.path.length))
                                message.path = [];
                            if ((tag & 7) === 2) {
                                let end2 = reader.uint32() + reader.pos;
                                while (reader.pos < end2)
                                    message.path.push(reader.int32());
                            } else
                                message.path.push(reader.int32());
                            break;
                        case 2:
                            if (!(message.span && message.span.length))
                                message.span = [];
                            if ((tag & 7) === 2) {
                                let end2 = reader.uint32() + reader.pos;
                                while (reader.pos < end2)
                                    message.span.push(reader.int32());
                            } else
                                message.span.push(reader.int32());
                            break;
                        case 3:
                            message.leadingComments = reader.string();
                            break;
                        case 4:
                            message.trailingComments = reader.string();
                            break;
                        case 6:
                            if (!(message.leadingDetachedComments && message.leadingDetachedComments.length))
                                message.leadingDetachedComments = [];
                            message.leadingDetachedComments.push(reader.string());
                            break;
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a Location message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {google.protobuf.SourceCodeInfo.Location} Location
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Location.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a Location message.
                 * @function verify
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Location.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.path != null && message.hasOwnProperty("path")) {
                        if (!Array.isArray(message.path))
                            return "path: array expected";
                        for (let i = 0; i < message.path.length; ++i)
                            if (!$util.isInteger(message.path[i]))
                                return "path: integer[] expected";
                    }
                    if (message.span != null && message.hasOwnProperty("span")) {
                        if (!Array.isArray(message.span))
                            return "span: array expected";
                        for (let i = 0; i < message.span.length; ++i)
                            if (!$util.isInteger(message.span[i]))
                                return "span: integer[] expected";
                    }
                    if (message.leadingComments != null && message.hasOwnProperty("leadingComments"))
                        if (!$util.isString(message.leadingComments))
                            return "leadingComments: string expected";
                    if (message.trailingComments != null && message.hasOwnProperty("trailingComments"))
                        if (!$util.isString(message.trailingComments))
                            return "trailingComments: string expected";
                    if (message.leadingDetachedComments != null && message.hasOwnProperty("leadingDetachedComments")) {
                        if (!Array.isArray(message.leadingDetachedComments))
                            return "leadingDetachedComments: array expected";
                        for (let i = 0; i < message.leadingDetachedComments.length; ++i)
                            if (!$util.isString(message.leadingDetachedComments[i]))
                                return "leadingDetachedComments: string[] expected";
                    }
                    return null;
                };

                /**
                 * Creates a Location message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {google.protobuf.SourceCodeInfo.Location} Location
                 */
                Location.fromObject = function fromObject(object) {
                    if (object instanceof $root.google.protobuf.SourceCodeInfo.Location)
                        return object;
                    let message = new $root.google.protobuf.SourceCodeInfo.Location();
                    if (object.path) {
                        if (!Array.isArray(object.path))
                            throw TypeError(".google.protobuf.SourceCodeInfo.Location.path: array expected");
                        message.path = [];
                        for (let i = 0; i < object.path.length; ++i)
                            message.path[i] = object.path[i] | 0;
                    }
                    if (object.span) {
                        if (!Array.isArray(object.span))
                            throw TypeError(".google.protobuf.SourceCodeInfo.Location.span: array expected");
                        message.span = [];
                        for (let i = 0; i < object.span.length; ++i)
                            message.span[i] = object.span[i] | 0;
                    }
                    if (object.leadingComments != null)
                        message.leadingComments = String(object.leadingComments);
                    if (object.trailingComments != null)
                        message.trailingComments = String(object.trailingComments);
                    if (object.leadingDetachedComments) {
                        if (!Array.isArray(object.leadingDetachedComments))
                            throw TypeError(".google.protobuf.SourceCodeInfo.Location.leadingDetachedComments: array expected");
                        message.leadingDetachedComments = [];
                        for (let i = 0; i < object.leadingDetachedComments.length; ++i)
                            message.leadingDetachedComments[i] = String(object.leadingDetachedComments[i]);
                    }
                    return message;
                };

                /**
                 * Creates a plain object from a Location message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @static
                 * @param {google.protobuf.SourceCodeInfo.Location} message Location
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Location.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.arrays || options.defaults) {
                        object.path = [];
                        object.span = [];
                        object.leadingDetachedComments = [];
                    }
                    if (options.defaults) {
                        object.leadingComments = "";
                        object.trailingComments = "";
                    }
                    if (message.path && message.path.length) {
                        object.path = [];
                        for (let j = 0; j < message.path.length; ++j)
                            object.path[j] = message.path[j];
                    }
                    if (message.span && message.span.length) {
                        object.span = [];
                        for (let j = 0; j < message.span.length; ++j)
                            object.span[j] = message.span[j];
                    }
                    if (message.leadingComments != null && message.hasOwnProperty("leadingComments"))
                        object.leadingComments = message.leadingComments;
                    if (message.trailingComments != null && message.hasOwnProperty("trailingComments"))
                        object.trailingComments = message.trailingComments;
                    if (message.leadingDetachedComments && message.leadingDetachedComments.length) {
                        object.leadingDetachedComments = [];
                        for (let j = 0; j < message.leadingDetachedComments.length; ++j)
                            object.leadingDetachedComments[j] = message.leadingDetachedComments[j];
                    }
                    return object;
                };

                /**
                 * Converts this Location to JSON.
                 * @function toJSON
                 * @memberof google.protobuf.SourceCodeInfo.Location
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Location.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                return Location;
            })();

            return SourceCodeInfo;
        })();

        protobuf.GeneratedCodeInfo = (function() {

            /**
             * Properties of a GeneratedCodeInfo.
             * @memberof google.protobuf
             * @interface IGeneratedCodeInfo
             * @property {Array.<google.protobuf.GeneratedCodeInfo.IAnnotation>|null} [annotation] GeneratedCodeInfo annotation
             */

            /**
             * Constructs a new GeneratedCodeInfo.
             * @memberof google.protobuf
             * @classdesc Represents a GeneratedCodeInfo.
             * @implements IGeneratedCodeInfo
             * @constructor
             * @param {google.protobuf.IGeneratedCodeInfo=} [properties] Properties to set
             */
            function GeneratedCodeInfo(properties) {
                this.annotation = [];
                if (properties)
                    for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }

            /**
             * GeneratedCodeInfo annotation.
             * @member {Array.<google.protobuf.GeneratedCodeInfo.IAnnotation>} annotation
             * @memberof google.protobuf.GeneratedCodeInfo
             * @instance
             */
            GeneratedCodeInfo.prototype.annotation = $util.emptyArray;

            /**
             * Creates a new GeneratedCodeInfo instance using the specified properties.
             * @function create
             * @memberof google.protobuf.GeneratedCodeInfo
             * @static
             * @param {google.protobuf.IGeneratedCodeInfo=} [properties] Properties to set
             * @returns {google.protobuf.GeneratedCodeInfo} GeneratedCodeInfo instance
             */
            GeneratedCodeInfo.create = function create(properties) {
                return new GeneratedCodeInfo(properties);
            };

            /**
             * Encodes the specified GeneratedCodeInfo message. Does not implicitly {@link google.protobuf.GeneratedCodeInfo.verify|verify} messages.
             * @function encode
             * @memberof google.protobuf.GeneratedCodeInfo
             * @static
             * @param {google.protobuf.IGeneratedCodeInfo} message GeneratedCodeInfo message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            GeneratedCodeInfo.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.annotation != null && message.annotation.length)
                    for (let i = 0; i < message.annotation.length; ++i)
                        $root.google.protobuf.GeneratedCodeInfo.Annotation.encode(message.annotation[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };

            /**
             * Encodes the specified GeneratedCodeInfo message, length delimited. Does not implicitly {@link google.protobuf.GeneratedCodeInfo.verify|verify} messages.
             * @function encodeDelimited
             * @memberof google.protobuf.GeneratedCodeInfo
             * @static
             * @param {google.protobuf.IGeneratedCodeInfo} message GeneratedCodeInfo message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            GeneratedCodeInfo.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a GeneratedCodeInfo message from the specified reader or buffer.
             * @function decode
             * @memberof google.protobuf.GeneratedCodeInfo
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {google.protobuf.GeneratedCodeInfo} GeneratedCodeInfo
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            GeneratedCodeInfo.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.GeneratedCodeInfo();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.annotation && message.annotation.length))
                            message.annotation = [];
                        message.annotation.push($root.google.protobuf.GeneratedCodeInfo.Annotation.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };

            /**
             * Decodes a GeneratedCodeInfo message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof google.protobuf.GeneratedCodeInfo
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {google.protobuf.GeneratedCodeInfo} GeneratedCodeInfo
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            GeneratedCodeInfo.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };

            /**
             * Verifies a GeneratedCodeInfo message.
             * @function verify
             * @memberof google.protobuf.GeneratedCodeInfo
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            GeneratedCodeInfo.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.annotation != null && message.hasOwnProperty("annotation")) {
                    if (!Array.isArray(message.annotation))
                        return "annotation: array expected";
                    for (let i = 0; i < message.annotation.length; ++i) {
                        let error = $root.google.protobuf.GeneratedCodeInfo.Annotation.verify(message.annotation[i]);
                        if (error)
                            return "annotation." + error;
                    }
                }
                return null;
            };

            /**
             * Creates a GeneratedCodeInfo message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof google.protobuf.GeneratedCodeInfo
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {google.protobuf.GeneratedCodeInfo} GeneratedCodeInfo
             */
            GeneratedCodeInfo.fromObject = function fromObject(object) {
                if (object instanceof $root.google.protobuf.GeneratedCodeInfo)
                    return object;
                let message = new $root.google.protobuf.GeneratedCodeInfo();
                if (object.annotation) {
                    if (!Array.isArray(object.annotation))
                        throw TypeError(".google.protobuf.GeneratedCodeInfo.annotation: array expected");
                    message.annotation = [];
                    for (let i = 0; i < object.annotation.length; ++i) {
                        if (typeof object.annotation[i] !== "object")
                            throw TypeError(".google.protobuf.GeneratedCodeInfo.annotation: object expected");
                        message.annotation[i] = $root.google.protobuf.GeneratedCodeInfo.Annotation.fromObject(object.annotation[i]);
                    }
                }
                return message;
            };

            /**
             * Creates a plain object from a GeneratedCodeInfo message. Also converts values to other types if specified.
             * @function toObject
             * @memberof google.protobuf.GeneratedCodeInfo
             * @static
             * @param {google.protobuf.GeneratedCodeInfo} message GeneratedCodeInfo
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            GeneratedCodeInfo.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                let object = {};
                if (options.arrays || options.defaults)
                    object.annotation = [];
                if (message.annotation && message.annotation.length) {
                    object.annotation = [];
                    for (let j = 0; j < message.annotation.length; ++j)
                        object.annotation[j] = $root.google.protobuf.GeneratedCodeInfo.Annotation.toObject(message.annotation[j], options);
                }
                return object;
            };

            /**
             * Converts this GeneratedCodeInfo to JSON.
             * @function toJSON
             * @memberof google.protobuf.GeneratedCodeInfo
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            GeneratedCodeInfo.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            GeneratedCodeInfo.Annotation = (function() {

                /**
                 * Properties of an Annotation.
                 * @memberof google.protobuf.GeneratedCodeInfo
                 * @interface IAnnotation
                 * @property {Array.<number>|null} [path] Annotation path
                 * @property {string|null} [sourceFile] Annotation sourceFile
                 * @property {number|null} [begin] Annotation begin
                 * @property {number|null} [end] Annotation end
                 */

                /**
                 * Constructs a new Annotation.
                 * @memberof google.protobuf.GeneratedCodeInfo
                 * @classdesc Represents an Annotation.
                 * @implements IAnnotation
                 * @constructor
                 * @param {google.protobuf.GeneratedCodeInfo.IAnnotation=} [properties] Properties to set
                 */
                function Annotation(properties) {
                    this.path = [];
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Annotation path.
                 * @member {Array.<number>} path
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @instance
                 */
                Annotation.prototype.path = $util.emptyArray;

                /**
                 * Annotation sourceFile.
                 * @member {string} sourceFile
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @instance
                 */
                Annotation.prototype.sourceFile = "";

                /**
                 * Annotation begin.
                 * @member {number} begin
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @instance
                 */
                Annotation.prototype.begin = 0;

                /**
                 * Annotation end.
                 * @member {number} end
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @instance
                 */
                Annotation.prototype.end = 0;

                /**
                 * Creates a new Annotation instance using the specified properties.
                 * @function create
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @static
                 * @param {google.protobuf.GeneratedCodeInfo.IAnnotation=} [properties] Properties to set
                 * @returns {google.protobuf.GeneratedCodeInfo.Annotation} Annotation instance
                 */
                Annotation.create = function create(properties) {
                    return new Annotation(properties);
                };

                /**
                 * Encodes the specified Annotation message. Does not implicitly {@link google.protobuf.GeneratedCodeInfo.Annotation.verify|verify} messages.
                 * @function encode
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @static
                 * @param {google.protobuf.GeneratedCodeInfo.IAnnotation} message Annotation message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Annotation.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.path != null && message.path.length) {
                        writer.uint32(/* id 1, wireType 2 =*/10).fork();
                        for (let i = 0; i < message.path.length; ++i)
                            writer.int32(message.path[i]);
                        writer.ldelim();
                    }
                    if (message.sourceFile != null && Object.hasOwnProperty.call(message, "sourceFile"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.sourceFile);
                    if (message.begin != null && Object.hasOwnProperty.call(message, "begin"))
                        writer.uint32(/* id 3, wireType 0 =*/24).int32(message.begin);
                    if (message.end != null && Object.hasOwnProperty.call(message, "end"))
                        writer.uint32(/* id 4, wireType 0 =*/32).int32(message.end);
                    return writer;
                };

                /**
                 * Encodes the specified Annotation message, length delimited. Does not implicitly {@link google.protobuf.GeneratedCodeInfo.Annotation.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @static
                 * @param {google.protobuf.GeneratedCodeInfo.IAnnotation} message Annotation message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Annotation.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes an Annotation message from the specified reader or buffer.
                 * @function decode
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {google.protobuf.GeneratedCodeInfo.Annotation} Annotation
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Annotation.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.google.protobuf.GeneratedCodeInfo.Annotation();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1:
                            if (!(message.path && message.path.length))
                                message.path = [];
                            if ((tag & 7) === 2) {
                                let end2 = reader.uint32() + reader.pos;
                                while (reader.pos < end2)
                                    message.path.push(reader.int32());
                            } else
                                message.path.push(reader.int32());
                            break;
                        case 2:
                            message.sourceFile = reader.string();
                            break;
                        case 3:
                            message.begin = reader.int32();
                            break;
                        case 4:
                            message.end = reader.int32();
                            break;
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes an Annotation message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {google.protobuf.GeneratedCodeInfo.Annotation} Annotation
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Annotation.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies an Annotation message.
                 * @function verify
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Annotation.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.path != null && message.hasOwnProperty("path")) {
                        if (!Array.isArray(message.path))
                            return "path: array expected";
                        for (let i = 0; i < message.path.length; ++i)
                            if (!$util.isInteger(message.path[i]))
                                return "path: integer[] expected";
                    }
                    if (message.sourceFile != null && message.hasOwnProperty("sourceFile"))
                        if (!$util.isString(message.sourceFile))
                            return "sourceFile: string expected";
                    if (message.begin != null && message.hasOwnProperty("begin"))
                        if (!$util.isInteger(message.begin))
                            return "begin: integer expected";
                    if (message.end != null && message.hasOwnProperty("end"))
                        if (!$util.isInteger(message.end))
                            return "end: integer expected";
                    return null;
                };

                /**
                 * Creates an Annotation message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {google.protobuf.GeneratedCodeInfo.Annotation} Annotation
                 */
                Annotation.fromObject = function fromObject(object) {
                    if (object instanceof $root.google.protobuf.GeneratedCodeInfo.Annotation)
                        return object;
                    let message = new $root.google.protobuf.GeneratedCodeInfo.Annotation();
                    if (object.path) {
                        if (!Array.isArray(object.path))
                            throw TypeError(".google.protobuf.GeneratedCodeInfo.Annotation.path: array expected");
                        message.path = [];
                        for (let i = 0; i < object.path.length; ++i)
                            message.path[i] = object.path[i] | 0;
                    }
                    if (object.sourceFile != null)
                        message.sourceFile = String(object.sourceFile);
                    if (object.begin != null)
                        message.begin = object.begin | 0;
                    if (object.end != null)
                        message.end = object.end | 0;
                    return message;
                };

                /**
                 * Creates a plain object from an Annotation message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @static
                 * @param {google.protobuf.GeneratedCodeInfo.Annotation} message Annotation
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Annotation.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.arrays || options.defaults)
                        object.path = [];
                    if (options.defaults) {
                        object.sourceFile = "";
                        object.begin = 0;
                        object.end = 0;
                    }
                    if (message.path && message.path.length) {
                        object.path = [];
                        for (let j = 0; j < message.path.length; ++j)
                            object.path[j] = message.path[j];
                    }
                    if (message.sourceFile != null && message.hasOwnProperty("sourceFile"))
                        object.sourceFile = message.sourceFile;
                    if (message.begin != null && message.hasOwnProperty("begin"))
                        object.begin = message.begin;
                    if (message.end != null && message.hasOwnProperty("end"))
                        object.end = message.end;
                    return object;
                };

                /**
                 * Converts this Annotation to JSON.
                 * @function toJSON
                 * @memberof google.protobuf.GeneratedCodeInfo.Annotation
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Annotation.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                return Annotation;
            })();

            return GeneratedCodeInfo;
        })();

        return protobuf;
    })();

    return google;
})();

export { $root as default };

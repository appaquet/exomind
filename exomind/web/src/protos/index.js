/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars*/
import * as $protobuf from "protobufjs/minimal";

// Common aliases
const $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;

// Exported root namespace
const $root = $protobuf.roots["exomind-root"] || ($protobuf.roots["exomind-root"] = {});

export const exomind = $root.exomind = (() => {

    /**
     * Namespace exomind.
     * @exports exomind
     * @namespace
     */
    const exomind = {};

    exomind.base = (function() {

        /**
         * Namespace base.
         * @memberof exomind
         * @namespace
         */
        const base = {};

        base.v1 = (function() {

            /**
             * Namespace v1.
             * @memberof exomind.base
             * @namespace
             */
            const v1 = {};

            v1.Collection = (function() {

                /**
                 * Properties of a Collection.
                 * @memberof exomind.base.v1
                 * @interface ICollection
                 * @property {string|null} [name] Collection name
                 * @property {string|null} [description] Collection description
                 */

                /**
                 * Constructs a new Collection.
                 * @memberof exomind.base.v1
                 * @classdesc Represents a Collection.
                 * @implements ICollection
                 * @constructor
                 * @param {exomind.base.v1.ICollection=} [properties] Properties to set
                 */
                function Collection(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Collection name.
                 * @member {string} name
                 * @memberof exomind.base.v1.Collection
                 * @instance
                 */
                Collection.prototype.name = "";

                /**
                 * Collection description.
                 * @member {string} description
                 * @memberof exomind.base.v1.Collection
                 * @instance
                 */
                Collection.prototype.description = "";

                /**
                 * Creates a new Collection instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.Collection
                 * @static
                 * @param {exomind.base.v1.ICollection=} [properties] Properties to set
                 * @returns {exomind.base.v1.Collection} Collection instance
                 */
                Collection.create = function create(properties) {
                    return new Collection(properties);
                };

                /**
                 * Encodes the specified Collection message. Does not implicitly {@link exomind.base.v1.Collection.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.Collection
                 * @static
                 * @param {exomind.base.v1.ICollection} message Collection message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Collection.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                    if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.description);
                    return writer;
                };

                /**
                 * Encodes the specified Collection message, length delimited. Does not implicitly {@link exomind.base.v1.Collection.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.Collection
                 * @static
                 * @param {exomind.base.v1.ICollection} message Collection message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Collection.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a Collection message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.Collection
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.Collection} Collection
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Collection.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.Collection();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.name = reader.string();
                                break;
                            }
                        case 2: {
                                message.description = reader.string();
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a Collection message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.Collection
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.Collection} Collection
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Collection.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a Collection message.
                 * @function verify
                 * @memberof exomind.base.v1.Collection
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Collection.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.name != null && message.hasOwnProperty("name"))
                        if (!$util.isString(message.name))
                            return "name: string expected";
                    if (message.description != null && message.hasOwnProperty("description"))
                        if (!$util.isString(message.description))
                            return "description: string expected";
                    return null;
                };

                /**
                 * Creates a Collection message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.Collection
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.Collection} Collection
                 */
                Collection.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.Collection)
                        return object;
                    let message = new $root.exomind.base.v1.Collection();
                    if (object.name != null)
                        message.name = String(object.name);
                    if (object.description != null)
                        message.description = String(object.description);
                    return message;
                };

                /**
                 * Creates a plain object from a Collection message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.Collection
                 * @static
                 * @param {exomind.base.v1.Collection} message Collection
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Collection.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.name = "";
                        object.description = "";
                    }
                    if (message.name != null && message.hasOwnProperty("name"))
                        object.name = message.name;
                    if (message.description != null && message.hasOwnProperty("description"))
                        object.description = message.description;
                    return object;
                };

                /**
                 * Converts this Collection to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.Collection
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Collection.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for Collection
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.Collection
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                Collection.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.Collection";
                };

                return Collection;
            })();

            v1.CollectionChild = (function() {

                /**
                 * Properties of a CollectionChild.
                 * @memberof exomind.base.v1
                 * @interface ICollectionChild
                 * @property {exocore.store.IReference|null} [collection] CollectionChild collection
                 * @property {number|Long|null} [weight] CollectionChild weight
                 */

                /**
                 * Constructs a new CollectionChild.
                 * @memberof exomind.base.v1
                 * @classdesc Represents a CollectionChild.
                 * @implements ICollectionChild
                 * @constructor
                 * @param {exomind.base.v1.ICollectionChild=} [properties] Properties to set
                 */
                function CollectionChild(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * CollectionChild collection.
                 * @member {exocore.store.IReference|null|undefined} collection
                 * @memberof exomind.base.v1.CollectionChild
                 * @instance
                 */
                CollectionChild.prototype.collection = null;

                /**
                 * CollectionChild weight.
                 * @member {number|Long} weight
                 * @memberof exomind.base.v1.CollectionChild
                 * @instance
                 */
                CollectionChild.prototype.weight = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

                /**
                 * Creates a new CollectionChild instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.CollectionChild
                 * @static
                 * @param {exomind.base.v1.ICollectionChild=} [properties] Properties to set
                 * @returns {exomind.base.v1.CollectionChild} CollectionChild instance
                 */
                CollectionChild.create = function create(properties) {
                    return new CollectionChild(properties);
                };

                /**
                 * Encodes the specified CollectionChild message. Does not implicitly {@link exomind.base.v1.CollectionChild.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.CollectionChild
                 * @static
                 * @param {exomind.base.v1.ICollectionChild} message CollectionChild message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                CollectionChild.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.collection != null && Object.hasOwnProperty.call(message, "collection"))
                        $root.exocore.store.Reference.encode(message.collection, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                    if (message.weight != null && Object.hasOwnProperty.call(message, "weight"))
                        writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.weight);
                    return writer;
                };

                /**
                 * Encodes the specified CollectionChild message, length delimited. Does not implicitly {@link exomind.base.v1.CollectionChild.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.CollectionChild
                 * @static
                 * @param {exomind.base.v1.ICollectionChild} message CollectionChild message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                CollectionChild.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a CollectionChild message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.CollectionChild
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.CollectionChild} CollectionChild
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                CollectionChild.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.CollectionChild();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.collection = $root.exocore.store.Reference.decode(reader, reader.uint32());
                                break;
                            }
                        case 2: {
                                message.weight = reader.uint64();
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a CollectionChild message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.CollectionChild
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.CollectionChild} CollectionChild
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                CollectionChild.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a CollectionChild message.
                 * @function verify
                 * @memberof exomind.base.v1.CollectionChild
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                CollectionChild.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.collection != null && message.hasOwnProperty("collection")) {
                        let error = $root.exocore.store.Reference.verify(message.collection);
                        if (error)
                            return "collection." + error;
                    }
                    if (message.weight != null && message.hasOwnProperty("weight"))
                        if (!$util.isInteger(message.weight) && !(message.weight && $util.isInteger(message.weight.low) && $util.isInteger(message.weight.high)))
                            return "weight: integer|Long expected";
                    return null;
                };

                /**
                 * Creates a CollectionChild message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.CollectionChild
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.CollectionChild} CollectionChild
                 */
                CollectionChild.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.CollectionChild)
                        return object;
                    let message = new $root.exomind.base.v1.CollectionChild();
                    if (object.collection != null) {
                        if (typeof object.collection !== "object")
                            throw TypeError(".exomind.base.v1.CollectionChild.collection: object expected");
                        message.collection = $root.exocore.store.Reference.fromObject(object.collection);
                    }
                    if (object.weight != null)
                        if ($util.Long)
                            (message.weight = $util.Long.fromValue(object.weight)).unsigned = true;
                        else if (typeof object.weight === "string")
                            message.weight = parseInt(object.weight, 10);
                        else if (typeof object.weight === "number")
                            message.weight = object.weight;
                        else if (typeof object.weight === "object")
                            message.weight = new $util.LongBits(object.weight.low >>> 0, object.weight.high >>> 0).toNumber(true);
                    return message;
                };

                /**
                 * Creates a plain object from a CollectionChild message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.CollectionChild
                 * @static
                 * @param {exomind.base.v1.CollectionChild} message CollectionChild
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                CollectionChild.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.collection = null;
                        if ($util.Long) {
                            let long = new $util.Long(0, 0, true);
                            object.weight = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                        } else
                            object.weight = options.longs === String ? "0" : 0;
                    }
                    if (message.collection != null && message.hasOwnProperty("collection"))
                        object.collection = $root.exocore.store.Reference.toObject(message.collection, options);
                    if (message.weight != null && message.hasOwnProperty("weight"))
                        if (typeof message.weight === "number")
                            object.weight = options.longs === String ? String(message.weight) : message.weight;
                        else
                            object.weight = options.longs === String ? $util.Long.prototype.toString.call(message.weight) : options.longs === Number ? new $util.LongBits(message.weight.low >>> 0, message.weight.high >>> 0).toNumber(true) : message.weight;
                    return object;
                };

                /**
                 * Converts this CollectionChild to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.CollectionChild
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                CollectionChild.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for CollectionChild
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.CollectionChild
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                CollectionChild.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.CollectionChild";
                };

                return CollectionChild;
            })();

            v1.Snoozed = (function() {

                /**
                 * Properties of a Snoozed.
                 * @memberof exomind.base.v1
                 * @interface ISnoozed
                 * @property {google.protobuf.ITimestamp|null} [untilDate] Snoozed untilDate
                 */

                /**
                 * Constructs a new Snoozed.
                 * @memberof exomind.base.v1
                 * @classdesc Represents a Snoozed.
                 * @implements ISnoozed
                 * @constructor
                 * @param {exomind.base.v1.ISnoozed=} [properties] Properties to set
                 */
                function Snoozed(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Snoozed untilDate.
                 * @member {google.protobuf.ITimestamp|null|undefined} untilDate
                 * @memberof exomind.base.v1.Snoozed
                 * @instance
                 */
                Snoozed.prototype.untilDate = null;

                /**
                 * Creates a new Snoozed instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.Snoozed
                 * @static
                 * @param {exomind.base.v1.ISnoozed=} [properties] Properties to set
                 * @returns {exomind.base.v1.Snoozed} Snoozed instance
                 */
                Snoozed.create = function create(properties) {
                    return new Snoozed(properties);
                };

                /**
                 * Encodes the specified Snoozed message. Does not implicitly {@link exomind.base.v1.Snoozed.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.Snoozed
                 * @static
                 * @param {exomind.base.v1.ISnoozed} message Snoozed message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Snoozed.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.untilDate != null && Object.hasOwnProperty.call(message, "untilDate"))
                        $root.google.protobuf.Timestamp.encode(message.untilDate, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                    return writer;
                };

                /**
                 * Encodes the specified Snoozed message, length delimited. Does not implicitly {@link exomind.base.v1.Snoozed.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.Snoozed
                 * @static
                 * @param {exomind.base.v1.ISnoozed} message Snoozed message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Snoozed.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a Snoozed message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.Snoozed
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.Snoozed} Snoozed
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Snoozed.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.Snoozed();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 2: {
                                message.untilDate = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a Snoozed message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.Snoozed
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.Snoozed} Snoozed
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Snoozed.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a Snoozed message.
                 * @function verify
                 * @memberof exomind.base.v1.Snoozed
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Snoozed.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.untilDate != null && message.hasOwnProperty("untilDate")) {
                        let error = $root.google.protobuf.Timestamp.verify(message.untilDate);
                        if (error)
                            return "untilDate." + error;
                    }
                    return null;
                };

                /**
                 * Creates a Snoozed message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.Snoozed
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.Snoozed} Snoozed
                 */
                Snoozed.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.Snoozed)
                        return object;
                    let message = new $root.exomind.base.v1.Snoozed();
                    if (object.untilDate != null) {
                        if (typeof object.untilDate !== "object")
                            throw TypeError(".exomind.base.v1.Snoozed.untilDate: object expected");
                        message.untilDate = $root.google.protobuf.Timestamp.fromObject(object.untilDate);
                    }
                    return message;
                };

                /**
                 * Creates a plain object from a Snoozed message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.Snoozed
                 * @static
                 * @param {exomind.base.v1.Snoozed} message Snoozed
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Snoozed.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults)
                        object.untilDate = null;
                    if (message.untilDate != null && message.hasOwnProperty("untilDate"))
                        object.untilDate = $root.google.protobuf.Timestamp.toObject(message.untilDate, options);
                    return object;
                };

                /**
                 * Converts this Snoozed to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.Snoozed
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Snoozed.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for Snoozed
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.Snoozed
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                Snoozed.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.Snoozed";
                };

                return Snoozed;
            })();

            v1.Unread = (function() {

                /**
                 * Properties of an Unread.
                 * @memberof exomind.base.v1
                 * @interface IUnread
                 * @property {exocore.store.IReference|null} [entity] Unread entity
                 */

                /**
                 * Constructs a new Unread.
                 * @memberof exomind.base.v1
                 * @classdesc Represents an Unread.
                 * @implements IUnread
                 * @constructor
                 * @param {exomind.base.v1.IUnread=} [properties] Properties to set
                 */
                function Unread(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Unread entity.
                 * @member {exocore.store.IReference|null|undefined} entity
                 * @memberof exomind.base.v1.Unread
                 * @instance
                 */
                Unread.prototype.entity = null;

                /**
                 * Creates a new Unread instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.Unread
                 * @static
                 * @param {exomind.base.v1.IUnread=} [properties] Properties to set
                 * @returns {exomind.base.v1.Unread} Unread instance
                 */
                Unread.create = function create(properties) {
                    return new Unread(properties);
                };

                /**
                 * Encodes the specified Unread message. Does not implicitly {@link exomind.base.v1.Unread.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.Unread
                 * @static
                 * @param {exomind.base.v1.IUnread} message Unread message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Unread.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.entity != null && Object.hasOwnProperty.call(message, "entity"))
                        $root.exocore.store.Reference.encode(message.entity, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                    return writer;
                };

                /**
                 * Encodes the specified Unread message, length delimited. Does not implicitly {@link exomind.base.v1.Unread.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.Unread
                 * @static
                 * @param {exomind.base.v1.IUnread} message Unread message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Unread.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes an Unread message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.Unread
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.Unread} Unread
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Unread.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.Unread();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.entity = $root.exocore.store.Reference.decode(reader, reader.uint32());
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes an Unread message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.Unread
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.Unread} Unread
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Unread.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies an Unread message.
                 * @function verify
                 * @memberof exomind.base.v1.Unread
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Unread.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.entity != null && message.hasOwnProperty("entity")) {
                        let error = $root.exocore.store.Reference.verify(message.entity);
                        if (error)
                            return "entity." + error;
                    }
                    return null;
                };

                /**
                 * Creates an Unread message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.Unread
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.Unread} Unread
                 */
                Unread.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.Unread)
                        return object;
                    let message = new $root.exomind.base.v1.Unread();
                    if (object.entity != null) {
                        if (typeof object.entity !== "object")
                            throw TypeError(".exomind.base.v1.Unread.entity: object expected");
                        message.entity = $root.exocore.store.Reference.fromObject(object.entity);
                    }
                    return message;
                };

                /**
                 * Creates a plain object from an Unread message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.Unread
                 * @static
                 * @param {exomind.base.v1.Unread} message Unread
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Unread.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults)
                        object.entity = null;
                    if (message.entity != null && message.hasOwnProperty("entity"))
                        object.entity = $root.exocore.store.Reference.toObject(message.entity, options);
                    return object;
                };

                /**
                 * Converts this Unread to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.Unread
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Unread.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for Unread
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.Unread
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                Unread.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.Unread";
                };

                return Unread;
            })();

            /**
             * AccountType enum.
             * @name exomind.base.v1.AccountType
             * @enum {number}
             * @property {number} ACCOUNT_TYPE_INVALID=0 ACCOUNT_TYPE_INVALID value
             * @property {number} ACCOUNT_TYPE_GMAIL=1 ACCOUNT_TYPE_GMAIL value
             */
            v1.AccountType = (function() {
                const valuesById = {}, values = Object.create(valuesById);
                values[valuesById[0] = "ACCOUNT_TYPE_INVALID"] = 0;
                values[valuesById[1] = "ACCOUNT_TYPE_GMAIL"] = 1;
                return values;
            })();

            /**
             * AccountScope enum.
             * @name exomind.base.v1.AccountScope
             * @enum {number}
             * @property {number} ACCOUNT_SCOPE_INVALID=0 ACCOUNT_SCOPE_INVALID value
             * @property {number} ACCOUNT_SCOPE_EMAIL=1 ACCOUNT_SCOPE_EMAIL value
             */
            v1.AccountScope = (function() {
                const valuesById = {}, values = Object.create(valuesById);
                values[valuesById[0] = "ACCOUNT_SCOPE_INVALID"] = 0;
                values[valuesById[1] = "ACCOUNT_SCOPE_EMAIL"] = 1;
                return values;
            })();

            v1.Account = (function() {

                /**
                 * Properties of an Account.
                 * @memberof exomind.base.v1
                 * @interface IAccount
                 * @property {string|null} [key] Account key
                 * @property {string|null} [name] Account name
                 * @property {exomind.base.v1.AccountType|null} [type] Account type
                 * @property {Array.<exomind.base.v1.AccountScope>|null} [scopes] Account scopes
                 * @property {Object.<string,string>|null} [data] Account data
                 */

                /**
                 * Constructs a new Account.
                 * @memberof exomind.base.v1
                 * @classdesc Represents an Account.
                 * @implements IAccount
                 * @constructor
                 * @param {exomind.base.v1.IAccount=} [properties] Properties to set
                 */
                function Account(properties) {
                    this.scopes = [];
                    this.data = {};
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Account key.
                 * @member {string} key
                 * @memberof exomind.base.v1.Account
                 * @instance
                 */
                Account.prototype.key = "";

                /**
                 * Account name.
                 * @member {string} name
                 * @memberof exomind.base.v1.Account
                 * @instance
                 */
                Account.prototype.name = "";

                /**
                 * Account type.
                 * @member {exomind.base.v1.AccountType} type
                 * @memberof exomind.base.v1.Account
                 * @instance
                 */
                Account.prototype.type = 0;

                /**
                 * Account scopes.
                 * @member {Array.<exomind.base.v1.AccountScope>} scopes
                 * @memberof exomind.base.v1.Account
                 * @instance
                 */
                Account.prototype.scopes = $util.emptyArray;

                /**
                 * Account data.
                 * @member {Object.<string,string>} data
                 * @memberof exomind.base.v1.Account
                 * @instance
                 */
                Account.prototype.data = $util.emptyObject;

                /**
                 * Creates a new Account instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.Account
                 * @static
                 * @param {exomind.base.v1.IAccount=} [properties] Properties to set
                 * @returns {exomind.base.v1.Account} Account instance
                 */
                Account.create = function create(properties) {
                    return new Account(properties);
                };

                /**
                 * Encodes the specified Account message. Does not implicitly {@link exomind.base.v1.Account.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.Account
                 * @static
                 * @param {exomind.base.v1.IAccount} message Account message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Account.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.key != null && Object.hasOwnProperty.call(message, "key"))
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.key);
                    if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.name);
                    if (message.type != null && Object.hasOwnProperty.call(message, "type"))
                        writer.uint32(/* id 3, wireType 0 =*/24).int32(message.type);
                    if (message.scopes != null && message.scopes.length) {
                        writer.uint32(/* id 4, wireType 2 =*/34).fork();
                        for (let i = 0; i < message.scopes.length; ++i)
                            writer.int32(message.scopes[i]);
                        writer.ldelim();
                    }
                    if (message.data != null && Object.hasOwnProperty.call(message, "data"))
                        for (let keys = Object.keys(message.data), i = 0; i < keys.length; ++i)
                            writer.uint32(/* id 5, wireType 2 =*/42).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 2 =*/18).string(message.data[keys[i]]).ldelim();
                    return writer;
                };

                /**
                 * Encodes the specified Account message, length delimited. Does not implicitly {@link exomind.base.v1.Account.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.Account
                 * @static
                 * @param {exomind.base.v1.IAccount} message Account message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Account.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes an Account message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.Account
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.Account} Account
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Account.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.Account(), key, value;
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.key = reader.string();
                                break;
                            }
                        case 2: {
                                message.name = reader.string();
                                break;
                            }
                        case 3: {
                                message.type = reader.int32();
                                break;
                            }
                        case 4: {
                                if (!(message.scopes && message.scopes.length))
                                    message.scopes = [];
                                if ((tag & 7) === 2) {
                                    let end2 = reader.uint32() + reader.pos;
                                    while (reader.pos < end2)
                                        message.scopes.push(reader.int32());
                                } else
                                    message.scopes.push(reader.int32());
                                break;
                            }
                        case 5: {
                                if (message.data === $util.emptyObject)
                                    message.data = {};
                                let end2 = reader.uint32() + reader.pos;
                                key = "";
                                value = "";
                                while (reader.pos < end2) {
                                    let tag2 = reader.uint32();
                                    switch (tag2 >>> 3) {
                                    case 1:
                                        key = reader.string();
                                        break;
                                    case 2:
                                        value = reader.string();
                                        break;
                                    default:
                                        reader.skipType(tag2 & 7);
                                        break;
                                    }
                                }
                                message.data[key] = value;
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes an Account message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.Account
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.Account} Account
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Account.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies an Account message.
                 * @function verify
                 * @memberof exomind.base.v1.Account
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Account.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.key != null && message.hasOwnProperty("key"))
                        if (!$util.isString(message.key))
                            return "key: string expected";
                    if (message.name != null && message.hasOwnProperty("name"))
                        if (!$util.isString(message.name))
                            return "name: string expected";
                    if (message.type != null && message.hasOwnProperty("type"))
                        switch (message.type) {
                        default:
                            return "type: enum value expected";
                        case 0:
                        case 1:
                            break;
                        }
                    if (message.scopes != null && message.hasOwnProperty("scopes")) {
                        if (!Array.isArray(message.scopes))
                            return "scopes: array expected";
                        for (let i = 0; i < message.scopes.length; ++i)
                            switch (message.scopes[i]) {
                            default:
                                return "scopes: enum value[] expected";
                            case 0:
                            case 1:
                                break;
                            }
                    }
                    if (message.data != null && message.hasOwnProperty("data")) {
                        if (!$util.isObject(message.data))
                            return "data: object expected";
                        let key = Object.keys(message.data);
                        for (let i = 0; i < key.length; ++i)
                            if (!$util.isString(message.data[key[i]]))
                                return "data: string{k:string} expected";
                    }
                    return null;
                };

                /**
                 * Creates an Account message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.Account
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.Account} Account
                 */
                Account.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.Account)
                        return object;
                    let message = new $root.exomind.base.v1.Account();
                    if (object.key != null)
                        message.key = String(object.key);
                    if (object.name != null)
                        message.name = String(object.name);
                    switch (object.type) {
                    default:
                        if (typeof object.type === "number") {
                            message.type = object.type;
                            break;
                        }
                        break;
                    case "ACCOUNT_TYPE_INVALID":
                    case 0:
                        message.type = 0;
                        break;
                    case "ACCOUNT_TYPE_GMAIL":
                    case 1:
                        message.type = 1;
                        break;
                    }
                    if (object.scopes) {
                        if (!Array.isArray(object.scopes))
                            throw TypeError(".exomind.base.v1.Account.scopes: array expected");
                        message.scopes = [];
                        for (let i = 0; i < object.scopes.length; ++i)
                            switch (object.scopes[i]) {
                            default:
                                if (typeof object.scopes[i] === "number") {
                                    message.scopes[i] = object.scopes[i];
                                    break;
                                }
                            case "ACCOUNT_SCOPE_INVALID":
                            case 0:
                                message.scopes[i] = 0;
                                break;
                            case "ACCOUNT_SCOPE_EMAIL":
                            case 1:
                                message.scopes[i] = 1;
                                break;
                            }
                    }
                    if (object.data) {
                        if (typeof object.data !== "object")
                            throw TypeError(".exomind.base.v1.Account.data: object expected");
                        message.data = {};
                        for (let keys = Object.keys(object.data), i = 0; i < keys.length; ++i)
                            message.data[keys[i]] = String(object.data[keys[i]]);
                    }
                    return message;
                };

                /**
                 * Creates a plain object from an Account message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.Account
                 * @static
                 * @param {exomind.base.v1.Account} message Account
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Account.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.arrays || options.defaults)
                        object.scopes = [];
                    if (options.objects || options.defaults)
                        object.data = {};
                    if (options.defaults) {
                        object.key = "";
                        object.name = "";
                        object.type = options.enums === String ? "ACCOUNT_TYPE_INVALID" : 0;
                    }
                    if (message.key != null && message.hasOwnProperty("key"))
                        object.key = message.key;
                    if (message.name != null && message.hasOwnProperty("name"))
                        object.name = message.name;
                    if (message.type != null && message.hasOwnProperty("type"))
                        object.type = options.enums === String ? $root.exomind.base.v1.AccountType[message.type] === undefined ? message.type : $root.exomind.base.v1.AccountType[message.type] : message.type;
                    if (message.scopes && message.scopes.length) {
                        object.scopes = [];
                        for (let j = 0; j < message.scopes.length; ++j)
                            object.scopes[j] = options.enums === String ? $root.exomind.base.v1.AccountScope[message.scopes[j]] === undefined ? message.scopes[j] : $root.exomind.base.v1.AccountScope[message.scopes[j]] : message.scopes[j];
                    }
                    let keys2;
                    if (message.data && (keys2 = Object.keys(message.data)).length) {
                        object.data = {};
                        for (let j = 0; j < keys2.length; ++j)
                            object.data[keys2[j]] = message.data[keys2[j]];
                    }
                    return object;
                };

                /**
                 * Converts this Account to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.Account
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Account.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for Account
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.Account
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                Account.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.Account";
                };

                return Account;
            })();

            v1.EmailThread = (function() {

                /**
                 * Properties of an EmailThread.
                 * @memberof exomind.base.v1
                 * @interface IEmailThread
                 * @property {exocore.store.IReference|null} [account] EmailThread account
                 * @property {string|null} [sourceId] EmailThread sourceId
                 * @property {exomind.base.v1.IContact|null} [from] EmailThread from
                 * @property {string|null} [subject] EmailThread subject
                 * @property {string|null} [snippet] EmailThread snippet
                 * @property {exocore.store.IReference|null} [lastEmail] EmailThread lastEmail
                 * @property {boolean|null} [read] EmailThread read
                 */

                /**
                 * Constructs a new EmailThread.
                 * @memberof exomind.base.v1
                 * @classdesc Represents an EmailThread.
                 * @implements IEmailThread
                 * @constructor
                 * @param {exomind.base.v1.IEmailThread=} [properties] Properties to set
                 */
                function EmailThread(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * EmailThread account.
                 * @member {exocore.store.IReference|null|undefined} account
                 * @memberof exomind.base.v1.EmailThread
                 * @instance
                 */
                EmailThread.prototype.account = null;

                /**
                 * EmailThread sourceId.
                 * @member {string} sourceId
                 * @memberof exomind.base.v1.EmailThread
                 * @instance
                 */
                EmailThread.prototype.sourceId = "";

                /**
                 * EmailThread from.
                 * @member {exomind.base.v1.IContact|null|undefined} from
                 * @memberof exomind.base.v1.EmailThread
                 * @instance
                 */
                EmailThread.prototype.from = null;

                /**
                 * EmailThread subject.
                 * @member {string} subject
                 * @memberof exomind.base.v1.EmailThread
                 * @instance
                 */
                EmailThread.prototype.subject = "";

                /**
                 * EmailThread snippet.
                 * @member {string} snippet
                 * @memberof exomind.base.v1.EmailThread
                 * @instance
                 */
                EmailThread.prototype.snippet = "";

                /**
                 * EmailThread lastEmail.
                 * @member {exocore.store.IReference|null|undefined} lastEmail
                 * @memberof exomind.base.v1.EmailThread
                 * @instance
                 */
                EmailThread.prototype.lastEmail = null;

                /**
                 * EmailThread read.
                 * @member {boolean} read
                 * @memberof exomind.base.v1.EmailThread
                 * @instance
                 */
                EmailThread.prototype.read = false;

                /**
                 * Creates a new EmailThread instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.EmailThread
                 * @static
                 * @param {exomind.base.v1.IEmailThread=} [properties] Properties to set
                 * @returns {exomind.base.v1.EmailThread} EmailThread instance
                 */
                EmailThread.create = function create(properties) {
                    return new EmailThread(properties);
                };

                /**
                 * Encodes the specified EmailThread message. Does not implicitly {@link exomind.base.v1.EmailThread.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.EmailThread
                 * @static
                 * @param {exomind.base.v1.IEmailThread} message EmailThread message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                EmailThread.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.account != null && Object.hasOwnProperty.call(message, "account"))
                        $root.exocore.store.Reference.encode(message.account, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                    if (message.sourceId != null && Object.hasOwnProperty.call(message, "sourceId"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.sourceId);
                    if (message.from != null && Object.hasOwnProperty.call(message, "from"))
                        $root.exomind.base.v1.Contact.encode(message.from, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                    if (message.subject != null && Object.hasOwnProperty.call(message, "subject"))
                        writer.uint32(/* id 4, wireType 2 =*/34).string(message.subject);
                    if (message.snippet != null && Object.hasOwnProperty.call(message, "snippet"))
                        writer.uint32(/* id 5, wireType 2 =*/42).string(message.snippet);
                    if (message.lastEmail != null && Object.hasOwnProperty.call(message, "lastEmail"))
                        $root.exocore.store.Reference.encode(message.lastEmail, writer.uint32(/* id 6, wireType 2 =*/50).fork()).ldelim();
                    if (message.read != null && Object.hasOwnProperty.call(message, "read"))
                        writer.uint32(/* id 7, wireType 0 =*/56).bool(message.read);
                    return writer;
                };

                /**
                 * Encodes the specified EmailThread message, length delimited. Does not implicitly {@link exomind.base.v1.EmailThread.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.EmailThread
                 * @static
                 * @param {exomind.base.v1.IEmailThread} message EmailThread message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                EmailThread.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes an EmailThread message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.EmailThread
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.EmailThread} EmailThread
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                EmailThread.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.EmailThread();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.account = $root.exocore.store.Reference.decode(reader, reader.uint32());
                                break;
                            }
                        case 2: {
                                message.sourceId = reader.string();
                                break;
                            }
                        case 3: {
                                message.from = $root.exomind.base.v1.Contact.decode(reader, reader.uint32());
                                break;
                            }
                        case 4: {
                                message.subject = reader.string();
                                break;
                            }
                        case 5: {
                                message.snippet = reader.string();
                                break;
                            }
                        case 6: {
                                message.lastEmail = $root.exocore.store.Reference.decode(reader, reader.uint32());
                                break;
                            }
                        case 7: {
                                message.read = reader.bool();
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes an EmailThread message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.EmailThread
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.EmailThread} EmailThread
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                EmailThread.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies an EmailThread message.
                 * @function verify
                 * @memberof exomind.base.v1.EmailThread
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                EmailThread.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.account != null && message.hasOwnProperty("account")) {
                        let error = $root.exocore.store.Reference.verify(message.account);
                        if (error)
                            return "account." + error;
                    }
                    if (message.sourceId != null && message.hasOwnProperty("sourceId"))
                        if (!$util.isString(message.sourceId))
                            return "sourceId: string expected";
                    if (message.from != null && message.hasOwnProperty("from")) {
                        let error = $root.exomind.base.v1.Contact.verify(message.from);
                        if (error)
                            return "from." + error;
                    }
                    if (message.subject != null && message.hasOwnProperty("subject"))
                        if (!$util.isString(message.subject))
                            return "subject: string expected";
                    if (message.snippet != null && message.hasOwnProperty("snippet"))
                        if (!$util.isString(message.snippet))
                            return "snippet: string expected";
                    if (message.lastEmail != null && message.hasOwnProperty("lastEmail")) {
                        let error = $root.exocore.store.Reference.verify(message.lastEmail);
                        if (error)
                            return "lastEmail." + error;
                    }
                    if (message.read != null && message.hasOwnProperty("read"))
                        if (typeof message.read !== "boolean")
                            return "read: boolean expected";
                    return null;
                };

                /**
                 * Creates an EmailThread message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.EmailThread
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.EmailThread} EmailThread
                 */
                EmailThread.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.EmailThread)
                        return object;
                    let message = new $root.exomind.base.v1.EmailThread();
                    if (object.account != null) {
                        if (typeof object.account !== "object")
                            throw TypeError(".exomind.base.v1.EmailThread.account: object expected");
                        message.account = $root.exocore.store.Reference.fromObject(object.account);
                    }
                    if (object.sourceId != null)
                        message.sourceId = String(object.sourceId);
                    if (object.from != null) {
                        if (typeof object.from !== "object")
                            throw TypeError(".exomind.base.v1.EmailThread.from: object expected");
                        message.from = $root.exomind.base.v1.Contact.fromObject(object.from);
                    }
                    if (object.subject != null)
                        message.subject = String(object.subject);
                    if (object.snippet != null)
                        message.snippet = String(object.snippet);
                    if (object.lastEmail != null) {
                        if (typeof object.lastEmail !== "object")
                            throw TypeError(".exomind.base.v1.EmailThread.lastEmail: object expected");
                        message.lastEmail = $root.exocore.store.Reference.fromObject(object.lastEmail);
                    }
                    if (object.read != null)
                        message.read = Boolean(object.read);
                    return message;
                };

                /**
                 * Creates a plain object from an EmailThread message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.EmailThread
                 * @static
                 * @param {exomind.base.v1.EmailThread} message EmailThread
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                EmailThread.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.account = null;
                        object.sourceId = "";
                        object.from = null;
                        object.subject = "";
                        object.snippet = "";
                        object.lastEmail = null;
                        object.read = false;
                    }
                    if (message.account != null && message.hasOwnProperty("account"))
                        object.account = $root.exocore.store.Reference.toObject(message.account, options);
                    if (message.sourceId != null && message.hasOwnProperty("sourceId"))
                        object.sourceId = message.sourceId;
                    if (message.from != null && message.hasOwnProperty("from"))
                        object.from = $root.exomind.base.v1.Contact.toObject(message.from, options);
                    if (message.subject != null && message.hasOwnProperty("subject"))
                        object.subject = message.subject;
                    if (message.snippet != null && message.hasOwnProperty("snippet"))
                        object.snippet = message.snippet;
                    if (message.lastEmail != null && message.hasOwnProperty("lastEmail"))
                        object.lastEmail = $root.exocore.store.Reference.toObject(message.lastEmail, options);
                    if (message.read != null && message.hasOwnProperty("read"))
                        object.read = message.read;
                    return object;
                };

                /**
                 * Converts this EmailThread to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.EmailThread
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                EmailThread.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for EmailThread
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.EmailThread
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                EmailThread.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.EmailThread";
                };

                return EmailThread;
            })();

            v1.Email = (function() {

                /**
                 * Properties of an Email.
                 * @memberof exomind.base.v1
                 * @interface IEmail
                 * @property {exocore.store.IReference|null} [account] Email account
                 * @property {string|null} [sourceId] Email sourceId
                 * @property {exomind.base.v1.IContact|null} [from] Email from
                 * @property {google.protobuf.ITimestamp|null} [receivedDate] Email receivedDate
                 * @property {Array.<exomind.base.v1.IContact>|null} [to] Email to
                 * @property {Array.<exomind.base.v1.IContact>|null} [cc] Email cc
                 * @property {Array.<exomind.base.v1.IContact>|null} [bcc] Email bcc
                 * @property {string|null} [subject] Email subject
                 * @property {string|null} [snippet] Email snippet
                 * @property {Array.<exomind.base.v1.IEmailPart>|null} [parts] Email parts
                 * @property {Array.<exomind.base.v1.IEmailAttachment>|null} [attachments] Email attachments
                 * @property {boolean|null} [read] Email read
                 */

                /**
                 * Constructs a new Email.
                 * @memberof exomind.base.v1
                 * @classdesc Represents an Email.
                 * @implements IEmail
                 * @constructor
                 * @param {exomind.base.v1.IEmail=} [properties] Properties to set
                 */
                function Email(properties) {
                    this.to = [];
                    this.cc = [];
                    this.bcc = [];
                    this.parts = [];
                    this.attachments = [];
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Email account.
                 * @member {exocore.store.IReference|null|undefined} account
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.account = null;

                /**
                 * Email sourceId.
                 * @member {string} sourceId
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.sourceId = "";

                /**
                 * Email from.
                 * @member {exomind.base.v1.IContact|null|undefined} from
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.from = null;

                /**
                 * Email receivedDate.
                 * @member {google.protobuf.ITimestamp|null|undefined} receivedDate
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.receivedDate = null;

                /**
                 * Email to.
                 * @member {Array.<exomind.base.v1.IContact>} to
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.to = $util.emptyArray;

                /**
                 * Email cc.
                 * @member {Array.<exomind.base.v1.IContact>} cc
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.cc = $util.emptyArray;

                /**
                 * Email bcc.
                 * @member {Array.<exomind.base.v1.IContact>} bcc
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.bcc = $util.emptyArray;

                /**
                 * Email subject.
                 * @member {string} subject
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.subject = "";

                /**
                 * Email snippet.
                 * @member {string} snippet
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.snippet = "";

                /**
                 * Email parts.
                 * @member {Array.<exomind.base.v1.IEmailPart>} parts
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.parts = $util.emptyArray;

                /**
                 * Email attachments.
                 * @member {Array.<exomind.base.v1.IEmailAttachment>} attachments
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.attachments = $util.emptyArray;

                /**
                 * Email read.
                 * @member {boolean} read
                 * @memberof exomind.base.v1.Email
                 * @instance
                 */
                Email.prototype.read = false;

                /**
                 * Creates a new Email instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.Email
                 * @static
                 * @param {exomind.base.v1.IEmail=} [properties] Properties to set
                 * @returns {exomind.base.v1.Email} Email instance
                 */
                Email.create = function create(properties) {
                    return new Email(properties);
                };

                /**
                 * Encodes the specified Email message. Does not implicitly {@link exomind.base.v1.Email.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.Email
                 * @static
                 * @param {exomind.base.v1.IEmail} message Email message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Email.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.account != null && Object.hasOwnProperty.call(message, "account"))
                        $root.exocore.store.Reference.encode(message.account, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                    if (message.sourceId != null && Object.hasOwnProperty.call(message, "sourceId"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.sourceId);
                    if (message.from != null && Object.hasOwnProperty.call(message, "from"))
                        $root.exomind.base.v1.Contact.encode(message.from, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                    if (message.receivedDate != null && Object.hasOwnProperty.call(message, "receivedDate"))
                        $root.google.protobuf.Timestamp.encode(message.receivedDate, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                    if (message.to != null && message.to.length)
                        for (let i = 0; i < message.to.length; ++i)
                            $root.exomind.base.v1.Contact.encode(message.to[i], writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                    if (message.cc != null && message.cc.length)
                        for (let i = 0; i < message.cc.length; ++i)
                            $root.exomind.base.v1.Contact.encode(message.cc[i], writer.uint32(/* id 6, wireType 2 =*/50).fork()).ldelim();
                    if (message.bcc != null && message.bcc.length)
                        for (let i = 0; i < message.bcc.length; ++i)
                            $root.exomind.base.v1.Contact.encode(message.bcc[i], writer.uint32(/* id 7, wireType 2 =*/58).fork()).ldelim();
                    if (message.subject != null && Object.hasOwnProperty.call(message, "subject"))
                        writer.uint32(/* id 8, wireType 2 =*/66).string(message.subject);
                    if (message.snippet != null && Object.hasOwnProperty.call(message, "snippet"))
                        writer.uint32(/* id 9, wireType 2 =*/74).string(message.snippet);
                    if (message.parts != null && message.parts.length)
                        for (let i = 0; i < message.parts.length; ++i)
                            $root.exomind.base.v1.EmailPart.encode(message.parts[i], writer.uint32(/* id 10, wireType 2 =*/82).fork()).ldelim();
                    if (message.attachments != null && message.attachments.length)
                        for (let i = 0; i < message.attachments.length; ++i)
                            $root.exomind.base.v1.EmailAttachment.encode(message.attachments[i], writer.uint32(/* id 11, wireType 2 =*/90).fork()).ldelim();
                    if (message.read != null && Object.hasOwnProperty.call(message, "read"))
                        writer.uint32(/* id 14, wireType 0 =*/112).bool(message.read);
                    return writer;
                };

                /**
                 * Encodes the specified Email message, length delimited. Does not implicitly {@link exomind.base.v1.Email.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.Email
                 * @static
                 * @param {exomind.base.v1.IEmail} message Email message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Email.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes an Email message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.Email
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.Email} Email
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Email.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.Email();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.account = $root.exocore.store.Reference.decode(reader, reader.uint32());
                                break;
                            }
                        case 2: {
                                message.sourceId = reader.string();
                                break;
                            }
                        case 3: {
                                message.from = $root.exomind.base.v1.Contact.decode(reader, reader.uint32());
                                break;
                            }
                        case 4: {
                                message.receivedDate = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                                break;
                            }
                        case 5: {
                                if (!(message.to && message.to.length))
                                    message.to = [];
                                message.to.push($root.exomind.base.v1.Contact.decode(reader, reader.uint32()));
                                break;
                            }
                        case 6: {
                                if (!(message.cc && message.cc.length))
                                    message.cc = [];
                                message.cc.push($root.exomind.base.v1.Contact.decode(reader, reader.uint32()));
                                break;
                            }
                        case 7: {
                                if (!(message.bcc && message.bcc.length))
                                    message.bcc = [];
                                message.bcc.push($root.exomind.base.v1.Contact.decode(reader, reader.uint32()));
                                break;
                            }
                        case 8: {
                                message.subject = reader.string();
                                break;
                            }
                        case 9: {
                                message.snippet = reader.string();
                                break;
                            }
                        case 10: {
                                if (!(message.parts && message.parts.length))
                                    message.parts = [];
                                message.parts.push($root.exomind.base.v1.EmailPart.decode(reader, reader.uint32()));
                                break;
                            }
                        case 11: {
                                if (!(message.attachments && message.attachments.length))
                                    message.attachments = [];
                                message.attachments.push($root.exomind.base.v1.EmailAttachment.decode(reader, reader.uint32()));
                                break;
                            }
                        case 14: {
                                message.read = reader.bool();
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes an Email message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.Email
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.Email} Email
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Email.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies an Email message.
                 * @function verify
                 * @memberof exomind.base.v1.Email
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Email.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.account != null && message.hasOwnProperty("account")) {
                        let error = $root.exocore.store.Reference.verify(message.account);
                        if (error)
                            return "account." + error;
                    }
                    if (message.sourceId != null && message.hasOwnProperty("sourceId"))
                        if (!$util.isString(message.sourceId))
                            return "sourceId: string expected";
                    if (message.from != null && message.hasOwnProperty("from")) {
                        let error = $root.exomind.base.v1.Contact.verify(message.from);
                        if (error)
                            return "from." + error;
                    }
                    if (message.receivedDate != null && message.hasOwnProperty("receivedDate")) {
                        let error = $root.google.protobuf.Timestamp.verify(message.receivedDate);
                        if (error)
                            return "receivedDate." + error;
                    }
                    if (message.to != null && message.hasOwnProperty("to")) {
                        if (!Array.isArray(message.to))
                            return "to: array expected";
                        for (let i = 0; i < message.to.length; ++i) {
                            let error = $root.exomind.base.v1.Contact.verify(message.to[i]);
                            if (error)
                                return "to." + error;
                        }
                    }
                    if (message.cc != null && message.hasOwnProperty("cc")) {
                        if (!Array.isArray(message.cc))
                            return "cc: array expected";
                        for (let i = 0; i < message.cc.length; ++i) {
                            let error = $root.exomind.base.v1.Contact.verify(message.cc[i]);
                            if (error)
                                return "cc." + error;
                        }
                    }
                    if (message.bcc != null && message.hasOwnProperty("bcc")) {
                        if (!Array.isArray(message.bcc))
                            return "bcc: array expected";
                        for (let i = 0; i < message.bcc.length; ++i) {
                            let error = $root.exomind.base.v1.Contact.verify(message.bcc[i]);
                            if (error)
                                return "bcc." + error;
                        }
                    }
                    if (message.subject != null && message.hasOwnProperty("subject"))
                        if (!$util.isString(message.subject))
                            return "subject: string expected";
                    if (message.snippet != null && message.hasOwnProperty("snippet"))
                        if (!$util.isString(message.snippet))
                            return "snippet: string expected";
                    if (message.parts != null && message.hasOwnProperty("parts")) {
                        if (!Array.isArray(message.parts))
                            return "parts: array expected";
                        for (let i = 0; i < message.parts.length; ++i) {
                            let error = $root.exomind.base.v1.EmailPart.verify(message.parts[i]);
                            if (error)
                                return "parts." + error;
                        }
                    }
                    if (message.attachments != null && message.hasOwnProperty("attachments")) {
                        if (!Array.isArray(message.attachments))
                            return "attachments: array expected";
                        for (let i = 0; i < message.attachments.length; ++i) {
                            let error = $root.exomind.base.v1.EmailAttachment.verify(message.attachments[i]);
                            if (error)
                                return "attachments." + error;
                        }
                    }
                    if (message.read != null && message.hasOwnProperty("read"))
                        if (typeof message.read !== "boolean")
                            return "read: boolean expected";
                    return null;
                };

                /**
                 * Creates an Email message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.Email
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.Email} Email
                 */
                Email.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.Email)
                        return object;
                    let message = new $root.exomind.base.v1.Email();
                    if (object.account != null) {
                        if (typeof object.account !== "object")
                            throw TypeError(".exomind.base.v1.Email.account: object expected");
                        message.account = $root.exocore.store.Reference.fromObject(object.account);
                    }
                    if (object.sourceId != null)
                        message.sourceId = String(object.sourceId);
                    if (object.from != null) {
                        if (typeof object.from !== "object")
                            throw TypeError(".exomind.base.v1.Email.from: object expected");
                        message.from = $root.exomind.base.v1.Contact.fromObject(object.from);
                    }
                    if (object.receivedDate != null) {
                        if (typeof object.receivedDate !== "object")
                            throw TypeError(".exomind.base.v1.Email.receivedDate: object expected");
                        message.receivedDate = $root.google.protobuf.Timestamp.fromObject(object.receivedDate);
                    }
                    if (object.to) {
                        if (!Array.isArray(object.to))
                            throw TypeError(".exomind.base.v1.Email.to: array expected");
                        message.to = [];
                        for (let i = 0; i < object.to.length; ++i) {
                            if (typeof object.to[i] !== "object")
                                throw TypeError(".exomind.base.v1.Email.to: object expected");
                            message.to[i] = $root.exomind.base.v1.Contact.fromObject(object.to[i]);
                        }
                    }
                    if (object.cc) {
                        if (!Array.isArray(object.cc))
                            throw TypeError(".exomind.base.v1.Email.cc: array expected");
                        message.cc = [];
                        for (let i = 0; i < object.cc.length; ++i) {
                            if (typeof object.cc[i] !== "object")
                                throw TypeError(".exomind.base.v1.Email.cc: object expected");
                            message.cc[i] = $root.exomind.base.v1.Contact.fromObject(object.cc[i]);
                        }
                    }
                    if (object.bcc) {
                        if (!Array.isArray(object.bcc))
                            throw TypeError(".exomind.base.v1.Email.bcc: array expected");
                        message.bcc = [];
                        for (let i = 0; i < object.bcc.length; ++i) {
                            if (typeof object.bcc[i] !== "object")
                                throw TypeError(".exomind.base.v1.Email.bcc: object expected");
                            message.bcc[i] = $root.exomind.base.v1.Contact.fromObject(object.bcc[i]);
                        }
                    }
                    if (object.subject != null)
                        message.subject = String(object.subject);
                    if (object.snippet != null)
                        message.snippet = String(object.snippet);
                    if (object.parts) {
                        if (!Array.isArray(object.parts))
                            throw TypeError(".exomind.base.v1.Email.parts: array expected");
                        message.parts = [];
                        for (let i = 0; i < object.parts.length; ++i) {
                            if (typeof object.parts[i] !== "object")
                                throw TypeError(".exomind.base.v1.Email.parts: object expected");
                            message.parts[i] = $root.exomind.base.v1.EmailPart.fromObject(object.parts[i]);
                        }
                    }
                    if (object.attachments) {
                        if (!Array.isArray(object.attachments))
                            throw TypeError(".exomind.base.v1.Email.attachments: array expected");
                        message.attachments = [];
                        for (let i = 0; i < object.attachments.length; ++i) {
                            if (typeof object.attachments[i] !== "object")
                                throw TypeError(".exomind.base.v1.Email.attachments: object expected");
                            message.attachments[i] = $root.exomind.base.v1.EmailAttachment.fromObject(object.attachments[i]);
                        }
                    }
                    if (object.read != null)
                        message.read = Boolean(object.read);
                    return message;
                };

                /**
                 * Creates a plain object from an Email message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.Email
                 * @static
                 * @param {exomind.base.v1.Email} message Email
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Email.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.arrays || options.defaults) {
                        object.to = [];
                        object.cc = [];
                        object.bcc = [];
                        object.parts = [];
                        object.attachments = [];
                    }
                    if (options.defaults) {
                        object.account = null;
                        object.sourceId = "";
                        object.from = null;
                        object.receivedDate = null;
                        object.subject = "";
                        object.snippet = "";
                        object.read = false;
                    }
                    if (message.account != null && message.hasOwnProperty("account"))
                        object.account = $root.exocore.store.Reference.toObject(message.account, options);
                    if (message.sourceId != null && message.hasOwnProperty("sourceId"))
                        object.sourceId = message.sourceId;
                    if (message.from != null && message.hasOwnProperty("from"))
                        object.from = $root.exomind.base.v1.Contact.toObject(message.from, options);
                    if (message.receivedDate != null && message.hasOwnProperty("receivedDate"))
                        object.receivedDate = $root.google.protobuf.Timestamp.toObject(message.receivedDate, options);
                    if (message.to && message.to.length) {
                        object.to = [];
                        for (let j = 0; j < message.to.length; ++j)
                            object.to[j] = $root.exomind.base.v1.Contact.toObject(message.to[j], options);
                    }
                    if (message.cc && message.cc.length) {
                        object.cc = [];
                        for (let j = 0; j < message.cc.length; ++j)
                            object.cc[j] = $root.exomind.base.v1.Contact.toObject(message.cc[j], options);
                    }
                    if (message.bcc && message.bcc.length) {
                        object.bcc = [];
                        for (let j = 0; j < message.bcc.length; ++j)
                            object.bcc[j] = $root.exomind.base.v1.Contact.toObject(message.bcc[j], options);
                    }
                    if (message.subject != null && message.hasOwnProperty("subject"))
                        object.subject = message.subject;
                    if (message.snippet != null && message.hasOwnProperty("snippet"))
                        object.snippet = message.snippet;
                    if (message.parts && message.parts.length) {
                        object.parts = [];
                        for (let j = 0; j < message.parts.length; ++j)
                            object.parts[j] = $root.exomind.base.v1.EmailPart.toObject(message.parts[j], options);
                    }
                    if (message.attachments && message.attachments.length) {
                        object.attachments = [];
                        for (let j = 0; j < message.attachments.length; ++j)
                            object.attachments[j] = $root.exomind.base.v1.EmailAttachment.toObject(message.attachments[j], options);
                    }
                    if (message.read != null && message.hasOwnProperty("read"))
                        object.read = message.read;
                    return object;
                };

                /**
                 * Converts this Email to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.Email
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Email.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for Email
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.Email
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                Email.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.Email";
                };

                return Email;
            })();

            v1.DraftEmail = (function() {

                /**
                 * Properties of a DraftEmail.
                 * @memberof exomind.base.v1
                 * @interface IDraftEmail
                 * @property {exocore.store.IReference|null} [account] DraftEmail account
                 * @property {exocore.store.IReference|null} [inReplyTo] DraftEmail inReplyTo
                 * @property {Array.<exomind.base.v1.IContact>|null} [to] DraftEmail to
                 * @property {Array.<exomind.base.v1.IContact>|null} [cc] DraftEmail cc
                 * @property {Array.<exomind.base.v1.IContact>|null} [bcc] DraftEmail bcc
                 * @property {string|null} [subject] DraftEmail subject
                 * @property {Array.<exomind.base.v1.IEmailPart>|null} [parts] DraftEmail parts
                 * @property {Array.<exomind.base.v1.IEmailAttachment>|null} [attachments] DraftEmail attachments
                 * @property {google.protobuf.ITimestamp|null} [sendingDate] DraftEmail sendingDate
                 * @property {google.protobuf.ITimestamp|null} [sentDate] DraftEmail sentDate
                 */

                /**
                 * Constructs a new DraftEmail.
                 * @memberof exomind.base.v1
                 * @classdesc Represents a DraftEmail.
                 * @implements IDraftEmail
                 * @constructor
                 * @param {exomind.base.v1.IDraftEmail=} [properties] Properties to set
                 */
                function DraftEmail(properties) {
                    this.to = [];
                    this.cc = [];
                    this.bcc = [];
                    this.parts = [];
                    this.attachments = [];
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * DraftEmail account.
                 * @member {exocore.store.IReference|null|undefined} account
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.account = null;

                /**
                 * DraftEmail inReplyTo.
                 * @member {exocore.store.IReference|null|undefined} inReplyTo
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.inReplyTo = null;

                /**
                 * DraftEmail to.
                 * @member {Array.<exomind.base.v1.IContact>} to
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.to = $util.emptyArray;

                /**
                 * DraftEmail cc.
                 * @member {Array.<exomind.base.v1.IContact>} cc
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.cc = $util.emptyArray;

                /**
                 * DraftEmail bcc.
                 * @member {Array.<exomind.base.v1.IContact>} bcc
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.bcc = $util.emptyArray;

                /**
                 * DraftEmail subject.
                 * @member {string} subject
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.subject = "";

                /**
                 * DraftEmail parts.
                 * @member {Array.<exomind.base.v1.IEmailPart>} parts
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.parts = $util.emptyArray;

                /**
                 * DraftEmail attachments.
                 * @member {Array.<exomind.base.v1.IEmailAttachment>} attachments
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.attachments = $util.emptyArray;

                /**
                 * DraftEmail sendingDate.
                 * @member {google.protobuf.ITimestamp|null|undefined} sendingDate
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.sendingDate = null;

                /**
                 * DraftEmail sentDate.
                 * @member {google.protobuf.ITimestamp|null|undefined} sentDate
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 */
                DraftEmail.prototype.sentDate = null;

                /**
                 * Creates a new DraftEmail instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.DraftEmail
                 * @static
                 * @param {exomind.base.v1.IDraftEmail=} [properties] Properties to set
                 * @returns {exomind.base.v1.DraftEmail} DraftEmail instance
                 */
                DraftEmail.create = function create(properties) {
                    return new DraftEmail(properties);
                };

                /**
                 * Encodes the specified DraftEmail message. Does not implicitly {@link exomind.base.v1.DraftEmail.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.DraftEmail
                 * @static
                 * @param {exomind.base.v1.IDraftEmail} message DraftEmail message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                DraftEmail.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.account != null && Object.hasOwnProperty.call(message, "account"))
                        $root.exocore.store.Reference.encode(message.account, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                    if (message.inReplyTo != null && Object.hasOwnProperty.call(message, "inReplyTo"))
                        $root.exocore.store.Reference.encode(message.inReplyTo, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                    if (message.to != null && message.to.length)
                        for (let i = 0; i < message.to.length; ++i)
                            $root.exomind.base.v1.Contact.encode(message.to[i], writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                    if (message.cc != null && message.cc.length)
                        for (let i = 0; i < message.cc.length; ++i)
                            $root.exomind.base.v1.Contact.encode(message.cc[i], writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                    if (message.bcc != null && message.bcc.length)
                        for (let i = 0; i < message.bcc.length; ++i)
                            $root.exomind.base.v1.Contact.encode(message.bcc[i], writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                    if (message.subject != null && Object.hasOwnProperty.call(message, "subject"))
                        writer.uint32(/* id 6, wireType 2 =*/50).string(message.subject);
                    if (message.parts != null && message.parts.length)
                        for (let i = 0; i < message.parts.length; ++i)
                            $root.exomind.base.v1.EmailPart.encode(message.parts[i], writer.uint32(/* id 7, wireType 2 =*/58).fork()).ldelim();
                    if (message.attachments != null && message.attachments.length)
                        for (let i = 0; i < message.attachments.length; ++i)
                            $root.exomind.base.v1.EmailAttachment.encode(message.attachments[i], writer.uint32(/* id 8, wireType 2 =*/66).fork()).ldelim();
                    if (message.sendingDate != null && Object.hasOwnProperty.call(message, "sendingDate"))
                        $root.google.protobuf.Timestamp.encode(message.sendingDate, writer.uint32(/* id 9, wireType 2 =*/74).fork()).ldelim();
                    if (message.sentDate != null && Object.hasOwnProperty.call(message, "sentDate"))
                        $root.google.protobuf.Timestamp.encode(message.sentDate, writer.uint32(/* id 10, wireType 2 =*/82).fork()).ldelim();
                    return writer;
                };

                /**
                 * Encodes the specified DraftEmail message, length delimited. Does not implicitly {@link exomind.base.v1.DraftEmail.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.DraftEmail
                 * @static
                 * @param {exomind.base.v1.IDraftEmail} message DraftEmail message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                DraftEmail.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a DraftEmail message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.DraftEmail
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.DraftEmail} DraftEmail
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                DraftEmail.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.DraftEmail();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.account = $root.exocore.store.Reference.decode(reader, reader.uint32());
                                break;
                            }
                        case 2: {
                                message.inReplyTo = $root.exocore.store.Reference.decode(reader, reader.uint32());
                                break;
                            }
                        case 3: {
                                if (!(message.to && message.to.length))
                                    message.to = [];
                                message.to.push($root.exomind.base.v1.Contact.decode(reader, reader.uint32()));
                                break;
                            }
                        case 4: {
                                if (!(message.cc && message.cc.length))
                                    message.cc = [];
                                message.cc.push($root.exomind.base.v1.Contact.decode(reader, reader.uint32()));
                                break;
                            }
                        case 5: {
                                if (!(message.bcc && message.bcc.length))
                                    message.bcc = [];
                                message.bcc.push($root.exomind.base.v1.Contact.decode(reader, reader.uint32()));
                                break;
                            }
                        case 6: {
                                message.subject = reader.string();
                                break;
                            }
                        case 7: {
                                if (!(message.parts && message.parts.length))
                                    message.parts = [];
                                message.parts.push($root.exomind.base.v1.EmailPart.decode(reader, reader.uint32()));
                                break;
                            }
                        case 8: {
                                if (!(message.attachments && message.attachments.length))
                                    message.attachments = [];
                                message.attachments.push($root.exomind.base.v1.EmailAttachment.decode(reader, reader.uint32()));
                                break;
                            }
                        case 9: {
                                message.sendingDate = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                                break;
                            }
                        case 10: {
                                message.sentDate = $root.google.protobuf.Timestamp.decode(reader, reader.uint32());
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a DraftEmail message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.DraftEmail
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.DraftEmail} DraftEmail
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                DraftEmail.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a DraftEmail message.
                 * @function verify
                 * @memberof exomind.base.v1.DraftEmail
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                DraftEmail.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.account != null && message.hasOwnProperty("account")) {
                        let error = $root.exocore.store.Reference.verify(message.account);
                        if (error)
                            return "account." + error;
                    }
                    if (message.inReplyTo != null && message.hasOwnProperty("inReplyTo")) {
                        let error = $root.exocore.store.Reference.verify(message.inReplyTo);
                        if (error)
                            return "inReplyTo." + error;
                    }
                    if (message.to != null && message.hasOwnProperty("to")) {
                        if (!Array.isArray(message.to))
                            return "to: array expected";
                        for (let i = 0; i < message.to.length; ++i) {
                            let error = $root.exomind.base.v1.Contact.verify(message.to[i]);
                            if (error)
                                return "to." + error;
                        }
                    }
                    if (message.cc != null && message.hasOwnProperty("cc")) {
                        if (!Array.isArray(message.cc))
                            return "cc: array expected";
                        for (let i = 0; i < message.cc.length; ++i) {
                            let error = $root.exomind.base.v1.Contact.verify(message.cc[i]);
                            if (error)
                                return "cc." + error;
                        }
                    }
                    if (message.bcc != null && message.hasOwnProperty("bcc")) {
                        if (!Array.isArray(message.bcc))
                            return "bcc: array expected";
                        for (let i = 0; i < message.bcc.length; ++i) {
                            let error = $root.exomind.base.v1.Contact.verify(message.bcc[i]);
                            if (error)
                                return "bcc." + error;
                        }
                    }
                    if (message.subject != null && message.hasOwnProperty("subject"))
                        if (!$util.isString(message.subject))
                            return "subject: string expected";
                    if (message.parts != null && message.hasOwnProperty("parts")) {
                        if (!Array.isArray(message.parts))
                            return "parts: array expected";
                        for (let i = 0; i < message.parts.length; ++i) {
                            let error = $root.exomind.base.v1.EmailPart.verify(message.parts[i]);
                            if (error)
                                return "parts." + error;
                        }
                    }
                    if (message.attachments != null && message.hasOwnProperty("attachments")) {
                        if (!Array.isArray(message.attachments))
                            return "attachments: array expected";
                        for (let i = 0; i < message.attachments.length; ++i) {
                            let error = $root.exomind.base.v1.EmailAttachment.verify(message.attachments[i]);
                            if (error)
                                return "attachments." + error;
                        }
                    }
                    if (message.sendingDate != null && message.hasOwnProperty("sendingDate")) {
                        let error = $root.google.protobuf.Timestamp.verify(message.sendingDate);
                        if (error)
                            return "sendingDate." + error;
                    }
                    if (message.sentDate != null && message.hasOwnProperty("sentDate")) {
                        let error = $root.google.protobuf.Timestamp.verify(message.sentDate);
                        if (error)
                            return "sentDate." + error;
                    }
                    return null;
                };

                /**
                 * Creates a DraftEmail message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.DraftEmail
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.DraftEmail} DraftEmail
                 */
                DraftEmail.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.DraftEmail)
                        return object;
                    let message = new $root.exomind.base.v1.DraftEmail();
                    if (object.account != null) {
                        if (typeof object.account !== "object")
                            throw TypeError(".exomind.base.v1.DraftEmail.account: object expected");
                        message.account = $root.exocore.store.Reference.fromObject(object.account);
                    }
                    if (object.inReplyTo != null) {
                        if (typeof object.inReplyTo !== "object")
                            throw TypeError(".exomind.base.v1.DraftEmail.inReplyTo: object expected");
                        message.inReplyTo = $root.exocore.store.Reference.fromObject(object.inReplyTo);
                    }
                    if (object.to) {
                        if (!Array.isArray(object.to))
                            throw TypeError(".exomind.base.v1.DraftEmail.to: array expected");
                        message.to = [];
                        for (let i = 0; i < object.to.length; ++i) {
                            if (typeof object.to[i] !== "object")
                                throw TypeError(".exomind.base.v1.DraftEmail.to: object expected");
                            message.to[i] = $root.exomind.base.v1.Contact.fromObject(object.to[i]);
                        }
                    }
                    if (object.cc) {
                        if (!Array.isArray(object.cc))
                            throw TypeError(".exomind.base.v1.DraftEmail.cc: array expected");
                        message.cc = [];
                        for (let i = 0; i < object.cc.length; ++i) {
                            if (typeof object.cc[i] !== "object")
                                throw TypeError(".exomind.base.v1.DraftEmail.cc: object expected");
                            message.cc[i] = $root.exomind.base.v1.Contact.fromObject(object.cc[i]);
                        }
                    }
                    if (object.bcc) {
                        if (!Array.isArray(object.bcc))
                            throw TypeError(".exomind.base.v1.DraftEmail.bcc: array expected");
                        message.bcc = [];
                        for (let i = 0; i < object.bcc.length; ++i) {
                            if (typeof object.bcc[i] !== "object")
                                throw TypeError(".exomind.base.v1.DraftEmail.bcc: object expected");
                            message.bcc[i] = $root.exomind.base.v1.Contact.fromObject(object.bcc[i]);
                        }
                    }
                    if (object.subject != null)
                        message.subject = String(object.subject);
                    if (object.parts) {
                        if (!Array.isArray(object.parts))
                            throw TypeError(".exomind.base.v1.DraftEmail.parts: array expected");
                        message.parts = [];
                        for (let i = 0; i < object.parts.length; ++i) {
                            if (typeof object.parts[i] !== "object")
                                throw TypeError(".exomind.base.v1.DraftEmail.parts: object expected");
                            message.parts[i] = $root.exomind.base.v1.EmailPart.fromObject(object.parts[i]);
                        }
                    }
                    if (object.attachments) {
                        if (!Array.isArray(object.attachments))
                            throw TypeError(".exomind.base.v1.DraftEmail.attachments: array expected");
                        message.attachments = [];
                        for (let i = 0; i < object.attachments.length; ++i) {
                            if (typeof object.attachments[i] !== "object")
                                throw TypeError(".exomind.base.v1.DraftEmail.attachments: object expected");
                            message.attachments[i] = $root.exomind.base.v1.EmailAttachment.fromObject(object.attachments[i]);
                        }
                    }
                    if (object.sendingDate != null) {
                        if (typeof object.sendingDate !== "object")
                            throw TypeError(".exomind.base.v1.DraftEmail.sendingDate: object expected");
                        message.sendingDate = $root.google.protobuf.Timestamp.fromObject(object.sendingDate);
                    }
                    if (object.sentDate != null) {
                        if (typeof object.sentDate !== "object")
                            throw TypeError(".exomind.base.v1.DraftEmail.sentDate: object expected");
                        message.sentDate = $root.google.protobuf.Timestamp.fromObject(object.sentDate);
                    }
                    return message;
                };

                /**
                 * Creates a plain object from a DraftEmail message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.DraftEmail
                 * @static
                 * @param {exomind.base.v1.DraftEmail} message DraftEmail
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                DraftEmail.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.arrays || options.defaults) {
                        object.to = [];
                        object.cc = [];
                        object.bcc = [];
                        object.parts = [];
                        object.attachments = [];
                    }
                    if (options.defaults) {
                        object.account = null;
                        object.inReplyTo = null;
                        object.subject = "";
                        object.sendingDate = null;
                        object.sentDate = null;
                    }
                    if (message.account != null && message.hasOwnProperty("account"))
                        object.account = $root.exocore.store.Reference.toObject(message.account, options);
                    if (message.inReplyTo != null && message.hasOwnProperty("inReplyTo"))
                        object.inReplyTo = $root.exocore.store.Reference.toObject(message.inReplyTo, options);
                    if (message.to && message.to.length) {
                        object.to = [];
                        for (let j = 0; j < message.to.length; ++j)
                            object.to[j] = $root.exomind.base.v1.Contact.toObject(message.to[j], options);
                    }
                    if (message.cc && message.cc.length) {
                        object.cc = [];
                        for (let j = 0; j < message.cc.length; ++j)
                            object.cc[j] = $root.exomind.base.v1.Contact.toObject(message.cc[j], options);
                    }
                    if (message.bcc && message.bcc.length) {
                        object.bcc = [];
                        for (let j = 0; j < message.bcc.length; ++j)
                            object.bcc[j] = $root.exomind.base.v1.Contact.toObject(message.bcc[j], options);
                    }
                    if (message.subject != null && message.hasOwnProperty("subject"))
                        object.subject = message.subject;
                    if (message.parts && message.parts.length) {
                        object.parts = [];
                        for (let j = 0; j < message.parts.length; ++j)
                            object.parts[j] = $root.exomind.base.v1.EmailPart.toObject(message.parts[j], options);
                    }
                    if (message.attachments && message.attachments.length) {
                        object.attachments = [];
                        for (let j = 0; j < message.attachments.length; ++j)
                            object.attachments[j] = $root.exomind.base.v1.EmailAttachment.toObject(message.attachments[j], options);
                    }
                    if (message.sendingDate != null && message.hasOwnProperty("sendingDate"))
                        object.sendingDate = $root.google.protobuf.Timestamp.toObject(message.sendingDate, options);
                    if (message.sentDate != null && message.hasOwnProperty("sentDate"))
                        object.sentDate = $root.google.protobuf.Timestamp.toObject(message.sentDate, options);
                    return object;
                };

                /**
                 * Converts this DraftEmail to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.DraftEmail
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                DraftEmail.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for DraftEmail
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.DraftEmail
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                DraftEmail.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.DraftEmail";
                };

                return DraftEmail;
            })();

            v1.EmailPart = (function() {

                /**
                 * Properties of an EmailPart.
                 * @memberof exomind.base.v1
                 * @interface IEmailPart
                 * @property {string|null} [mimeType] EmailPart mimeType
                 * @property {string|null} [body] EmailPart body
                 */

                /**
                 * Constructs a new EmailPart.
                 * @memberof exomind.base.v1
                 * @classdesc Represents an EmailPart.
                 * @implements IEmailPart
                 * @constructor
                 * @param {exomind.base.v1.IEmailPart=} [properties] Properties to set
                 */
                function EmailPart(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * EmailPart mimeType.
                 * @member {string} mimeType
                 * @memberof exomind.base.v1.EmailPart
                 * @instance
                 */
                EmailPart.prototype.mimeType = "";

                /**
                 * EmailPart body.
                 * @member {string} body
                 * @memberof exomind.base.v1.EmailPart
                 * @instance
                 */
                EmailPart.prototype.body = "";

                /**
                 * Creates a new EmailPart instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.EmailPart
                 * @static
                 * @param {exomind.base.v1.IEmailPart=} [properties] Properties to set
                 * @returns {exomind.base.v1.EmailPart} EmailPart instance
                 */
                EmailPart.create = function create(properties) {
                    return new EmailPart(properties);
                };

                /**
                 * Encodes the specified EmailPart message. Does not implicitly {@link exomind.base.v1.EmailPart.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.EmailPart
                 * @static
                 * @param {exomind.base.v1.IEmailPart} message EmailPart message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                EmailPart.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.mimeType != null && Object.hasOwnProperty.call(message, "mimeType"))
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.mimeType);
                    if (message.body != null && Object.hasOwnProperty.call(message, "body"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.body);
                    return writer;
                };

                /**
                 * Encodes the specified EmailPart message, length delimited. Does not implicitly {@link exomind.base.v1.EmailPart.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.EmailPart
                 * @static
                 * @param {exomind.base.v1.IEmailPart} message EmailPart message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                EmailPart.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes an EmailPart message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.EmailPart
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.EmailPart} EmailPart
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                EmailPart.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.EmailPart();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.mimeType = reader.string();
                                break;
                            }
                        case 2: {
                                message.body = reader.string();
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes an EmailPart message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.EmailPart
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.EmailPart} EmailPart
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                EmailPart.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies an EmailPart message.
                 * @function verify
                 * @memberof exomind.base.v1.EmailPart
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                EmailPart.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.mimeType != null && message.hasOwnProperty("mimeType"))
                        if (!$util.isString(message.mimeType))
                            return "mimeType: string expected";
                    if (message.body != null && message.hasOwnProperty("body"))
                        if (!$util.isString(message.body))
                            return "body: string expected";
                    return null;
                };

                /**
                 * Creates an EmailPart message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.EmailPart
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.EmailPart} EmailPart
                 */
                EmailPart.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.EmailPart)
                        return object;
                    let message = new $root.exomind.base.v1.EmailPart();
                    if (object.mimeType != null)
                        message.mimeType = String(object.mimeType);
                    if (object.body != null)
                        message.body = String(object.body);
                    return message;
                };

                /**
                 * Creates a plain object from an EmailPart message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.EmailPart
                 * @static
                 * @param {exomind.base.v1.EmailPart} message EmailPart
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                EmailPart.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.mimeType = "";
                        object.body = "";
                    }
                    if (message.mimeType != null && message.hasOwnProperty("mimeType"))
                        object.mimeType = message.mimeType;
                    if (message.body != null && message.hasOwnProperty("body"))
                        object.body = message.body;
                    return object;
                };

                /**
                 * Converts this EmailPart to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.EmailPart
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                EmailPart.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for EmailPart
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.EmailPart
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                EmailPart.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.EmailPart";
                };

                return EmailPart;
            })();

            v1.EmailAttachment = (function() {

                /**
                 * Properties of an EmailAttachment.
                 * @memberof exomind.base.v1
                 * @interface IEmailAttachment
                 * @property {string|null} [key] EmailAttachment key
                 * @property {string|null} [name] EmailAttachment name
                 * @property {string|null} [mimeType] EmailAttachment mimeType
                 * @property {number|Long|null} [size] EmailAttachment size
                 * @property {string|null} [inlinePlaceholder] EmailAttachment inlinePlaceholder
                 * @property {Object.<string,string>|null} [data] EmailAttachment data
                 */

                /**
                 * Constructs a new EmailAttachment.
                 * @memberof exomind.base.v1
                 * @classdesc Represents an EmailAttachment.
                 * @implements IEmailAttachment
                 * @constructor
                 * @param {exomind.base.v1.IEmailAttachment=} [properties] Properties to set
                 */
                function EmailAttachment(properties) {
                    this.data = {};
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * EmailAttachment key.
                 * @member {string} key
                 * @memberof exomind.base.v1.EmailAttachment
                 * @instance
                 */
                EmailAttachment.prototype.key = "";

                /**
                 * EmailAttachment name.
                 * @member {string} name
                 * @memberof exomind.base.v1.EmailAttachment
                 * @instance
                 */
                EmailAttachment.prototype.name = "";

                /**
                 * EmailAttachment mimeType.
                 * @member {string} mimeType
                 * @memberof exomind.base.v1.EmailAttachment
                 * @instance
                 */
                EmailAttachment.prototype.mimeType = "";

                /**
                 * EmailAttachment size.
                 * @member {number|Long} size
                 * @memberof exomind.base.v1.EmailAttachment
                 * @instance
                 */
                EmailAttachment.prototype.size = $util.Long ? $util.Long.fromBits(0,0,true) : 0;

                /**
                 * EmailAttachment inlinePlaceholder.
                 * @member {string} inlinePlaceholder
                 * @memberof exomind.base.v1.EmailAttachment
                 * @instance
                 */
                EmailAttachment.prototype.inlinePlaceholder = "";

                /**
                 * EmailAttachment data.
                 * @member {Object.<string,string>} data
                 * @memberof exomind.base.v1.EmailAttachment
                 * @instance
                 */
                EmailAttachment.prototype.data = $util.emptyObject;

                /**
                 * Creates a new EmailAttachment instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.EmailAttachment
                 * @static
                 * @param {exomind.base.v1.IEmailAttachment=} [properties] Properties to set
                 * @returns {exomind.base.v1.EmailAttachment} EmailAttachment instance
                 */
                EmailAttachment.create = function create(properties) {
                    return new EmailAttachment(properties);
                };

                /**
                 * Encodes the specified EmailAttachment message. Does not implicitly {@link exomind.base.v1.EmailAttachment.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.EmailAttachment
                 * @static
                 * @param {exomind.base.v1.IEmailAttachment} message EmailAttachment message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                EmailAttachment.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.key != null && Object.hasOwnProperty.call(message, "key"))
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.key);
                    if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.name);
                    if (message.mimeType != null && Object.hasOwnProperty.call(message, "mimeType"))
                        writer.uint32(/* id 3, wireType 2 =*/26).string(message.mimeType);
                    if (message.size != null && Object.hasOwnProperty.call(message, "size"))
                        writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.size);
                    if (message.inlinePlaceholder != null && Object.hasOwnProperty.call(message, "inlinePlaceholder"))
                        writer.uint32(/* id 5, wireType 2 =*/42).string(message.inlinePlaceholder);
                    if (message.data != null && Object.hasOwnProperty.call(message, "data"))
                        for (let keys = Object.keys(message.data), i = 0; i < keys.length; ++i)
                            writer.uint32(/* id 6, wireType 2 =*/50).fork().uint32(/* id 1, wireType 2 =*/10).string(keys[i]).uint32(/* id 2, wireType 2 =*/18).string(message.data[keys[i]]).ldelim();
                    return writer;
                };

                /**
                 * Encodes the specified EmailAttachment message, length delimited. Does not implicitly {@link exomind.base.v1.EmailAttachment.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.EmailAttachment
                 * @static
                 * @param {exomind.base.v1.IEmailAttachment} message EmailAttachment message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                EmailAttachment.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes an EmailAttachment message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.EmailAttachment
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.EmailAttachment} EmailAttachment
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                EmailAttachment.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.EmailAttachment(), key, value;
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.key = reader.string();
                                break;
                            }
                        case 2: {
                                message.name = reader.string();
                                break;
                            }
                        case 3: {
                                message.mimeType = reader.string();
                                break;
                            }
                        case 4: {
                                message.size = reader.uint64();
                                break;
                            }
                        case 5: {
                                message.inlinePlaceholder = reader.string();
                                break;
                            }
                        case 6: {
                                if (message.data === $util.emptyObject)
                                    message.data = {};
                                let end2 = reader.uint32() + reader.pos;
                                key = "";
                                value = "";
                                while (reader.pos < end2) {
                                    let tag2 = reader.uint32();
                                    switch (tag2 >>> 3) {
                                    case 1:
                                        key = reader.string();
                                        break;
                                    case 2:
                                        value = reader.string();
                                        break;
                                    default:
                                        reader.skipType(tag2 & 7);
                                        break;
                                    }
                                }
                                message.data[key] = value;
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes an EmailAttachment message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.EmailAttachment
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.EmailAttachment} EmailAttachment
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                EmailAttachment.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies an EmailAttachment message.
                 * @function verify
                 * @memberof exomind.base.v1.EmailAttachment
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                EmailAttachment.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.key != null && message.hasOwnProperty("key"))
                        if (!$util.isString(message.key))
                            return "key: string expected";
                    if (message.name != null && message.hasOwnProperty("name"))
                        if (!$util.isString(message.name))
                            return "name: string expected";
                    if (message.mimeType != null && message.hasOwnProperty("mimeType"))
                        if (!$util.isString(message.mimeType))
                            return "mimeType: string expected";
                    if (message.size != null && message.hasOwnProperty("size"))
                        if (!$util.isInteger(message.size) && !(message.size && $util.isInteger(message.size.low) && $util.isInteger(message.size.high)))
                            return "size: integer|Long expected";
                    if (message.inlinePlaceholder != null && message.hasOwnProperty("inlinePlaceholder"))
                        if (!$util.isString(message.inlinePlaceholder))
                            return "inlinePlaceholder: string expected";
                    if (message.data != null && message.hasOwnProperty("data")) {
                        if (!$util.isObject(message.data))
                            return "data: object expected";
                        let key = Object.keys(message.data);
                        for (let i = 0; i < key.length; ++i)
                            if (!$util.isString(message.data[key[i]]))
                                return "data: string{k:string} expected";
                    }
                    return null;
                };

                /**
                 * Creates an EmailAttachment message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.EmailAttachment
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.EmailAttachment} EmailAttachment
                 */
                EmailAttachment.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.EmailAttachment)
                        return object;
                    let message = new $root.exomind.base.v1.EmailAttachment();
                    if (object.key != null)
                        message.key = String(object.key);
                    if (object.name != null)
                        message.name = String(object.name);
                    if (object.mimeType != null)
                        message.mimeType = String(object.mimeType);
                    if (object.size != null)
                        if ($util.Long)
                            (message.size = $util.Long.fromValue(object.size)).unsigned = true;
                        else if (typeof object.size === "string")
                            message.size = parseInt(object.size, 10);
                        else if (typeof object.size === "number")
                            message.size = object.size;
                        else if (typeof object.size === "object")
                            message.size = new $util.LongBits(object.size.low >>> 0, object.size.high >>> 0).toNumber(true);
                    if (object.inlinePlaceholder != null)
                        message.inlinePlaceholder = String(object.inlinePlaceholder);
                    if (object.data) {
                        if (typeof object.data !== "object")
                            throw TypeError(".exomind.base.v1.EmailAttachment.data: object expected");
                        message.data = {};
                        for (let keys = Object.keys(object.data), i = 0; i < keys.length; ++i)
                            message.data[keys[i]] = String(object.data[keys[i]]);
                    }
                    return message;
                };

                /**
                 * Creates a plain object from an EmailAttachment message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.EmailAttachment
                 * @static
                 * @param {exomind.base.v1.EmailAttachment} message EmailAttachment
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                EmailAttachment.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.objects || options.defaults)
                        object.data = {};
                    if (options.defaults) {
                        object.key = "";
                        object.name = "";
                        object.mimeType = "";
                        if ($util.Long) {
                            let long = new $util.Long(0, 0, true);
                            object.size = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                        } else
                            object.size = options.longs === String ? "0" : 0;
                        object.inlinePlaceholder = "";
                    }
                    if (message.key != null && message.hasOwnProperty("key"))
                        object.key = message.key;
                    if (message.name != null && message.hasOwnProperty("name"))
                        object.name = message.name;
                    if (message.mimeType != null && message.hasOwnProperty("mimeType"))
                        object.mimeType = message.mimeType;
                    if (message.size != null && message.hasOwnProperty("size"))
                        if (typeof message.size === "number")
                            object.size = options.longs === String ? String(message.size) : message.size;
                        else
                            object.size = options.longs === String ? $util.Long.prototype.toString.call(message.size) : options.longs === Number ? new $util.LongBits(message.size.low >>> 0, message.size.high >>> 0).toNumber(true) : message.size;
                    if (message.inlinePlaceholder != null && message.hasOwnProperty("inlinePlaceholder"))
                        object.inlinePlaceholder = message.inlinePlaceholder;
                    let keys2;
                    if (message.data && (keys2 = Object.keys(message.data)).length) {
                        object.data = {};
                        for (let j = 0; j < keys2.length; ++j)
                            object.data[keys2[j]] = message.data[keys2[j]];
                    }
                    return object;
                };

                /**
                 * Converts this EmailAttachment to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.EmailAttachment
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                EmailAttachment.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for EmailAttachment
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.EmailAttachment
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                EmailAttachment.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.EmailAttachment";
                };

                return EmailAttachment;
            })();

            v1.Note = (function() {

                /**
                 * Properties of a Note.
                 * @memberof exomind.base.v1
                 * @interface INote
                 * @property {string|null} [title] Note title
                 * @property {string|null} [body] Note body
                 */

                /**
                 * Constructs a new Note.
                 * @memberof exomind.base.v1
                 * @classdesc Represents a Note.
                 * @implements INote
                 * @constructor
                 * @param {exomind.base.v1.INote=} [properties] Properties to set
                 */
                function Note(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Note title.
                 * @member {string} title
                 * @memberof exomind.base.v1.Note
                 * @instance
                 */
                Note.prototype.title = "";

                /**
                 * Note body.
                 * @member {string} body
                 * @memberof exomind.base.v1.Note
                 * @instance
                 */
                Note.prototype.body = "";

                /**
                 * Creates a new Note instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.Note
                 * @static
                 * @param {exomind.base.v1.INote=} [properties] Properties to set
                 * @returns {exomind.base.v1.Note} Note instance
                 */
                Note.create = function create(properties) {
                    return new Note(properties);
                };

                /**
                 * Encodes the specified Note message. Does not implicitly {@link exomind.base.v1.Note.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.Note
                 * @static
                 * @param {exomind.base.v1.INote} message Note message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Note.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.title != null && Object.hasOwnProperty.call(message, "title"))
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.title);
                    if (message.body != null && Object.hasOwnProperty.call(message, "body"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.body);
                    return writer;
                };

                /**
                 * Encodes the specified Note message, length delimited. Does not implicitly {@link exomind.base.v1.Note.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.Note
                 * @static
                 * @param {exomind.base.v1.INote} message Note message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Note.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a Note message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.Note
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.Note} Note
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Note.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.Note();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.title = reader.string();
                                break;
                            }
                        case 2: {
                                message.body = reader.string();
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a Note message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.Note
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.Note} Note
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Note.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a Note message.
                 * @function verify
                 * @memberof exomind.base.v1.Note
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Note.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.title != null && message.hasOwnProperty("title"))
                        if (!$util.isString(message.title))
                            return "title: string expected";
                    if (message.body != null && message.hasOwnProperty("body"))
                        if (!$util.isString(message.body))
                            return "body: string expected";
                    return null;
                };

                /**
                 * Creates a Note message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.Note
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.Note} Note
                 */
                Note.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.Note)
                        return object;
                    let message = new $root.exomind.base.v1.Note();
                    if (object.title != null)
                        message.title = String(object.title);
                    if (object.body != null)
                        message.body = String(object.body);
                    return message;
                };

                /**
                 * Creates a plain object from a Note message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.Note
                 * @static
                 * @param {exomind.base.v1.Note} message Note
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Note.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.title = "";
                        object.body = "";
                    }
                    if (message.title != null && message.hasOwnProperty("title"))
                        object.title = message.title;
                    if (message.body != null && message.hasOwnProperty("body"))
                        object.body = message.body;
                    return object;
                };

                /**
                 * Converts this Note to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.Note
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Note.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for Note
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.Note
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                Note.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.Note";
                };

                return Note;
            })();

            v1.Contact = (function() {

                /**
                 * Properties of a Contact.
                 * @memberof exomind.base.v1
                 * @interface IContact
                 * @property {string|null} [name] Contact name
                 * @property {string|null} [email] Contact email
                 */

                /**
                 * Constructs a new Contact.
                 * @memberof exomind.base.v1
                 * @classdesc Represents a Contact.
                 * @implements IContact
                 * @constructor
                 * @param {exomind.base.v1.IContact=} [properties] Properties to set
                 */
                function Contact(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Contact name.
                 * @member {string} name
                 * @memberof exomind.base.v1.Contact
                 * @instance
                 */
                Contact.prototype.name = "";

                /**
                 * Contact email.
                 * @member {string} email
                 * @memberof exomind.base.v1.Contact
                 * @instance
                 */
                Contact.prototype.email = "";

                /**
                 * Creates a new Contact instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.Contact
                 * @static
                 * @param {exomind.base.v1.IContact=} [properties] Properties to set
                 * @returns {exomind.base.v1.Contact} Contact instance
                 */
                Contact.create = function create(properties) {
                    return new Contact(properties);
                };

                /**
                 * Encodes the specified Contact message. Does not implicitly {@link exomind.base.v1.Contact.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.Contact
                 * @static
                 * @param {exomind.base.v1.IContact} message Contact message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Contact.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.name != null && Object.hasOwnProperty.call(message, "name"))
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.name);
                    if (message.email != null && Object.hasOwnProperty.call(message, "email"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.email);
                    return writer;
                };

                /**
                 * Encodes the specified Contact message, length delimited. Does not implicitly {@link exomind.base.v1.Contact.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.Contact
                 * @static
                 * @param {exomind.base.v1.IContact} message Contact message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Contact.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a Contact message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.Contact
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.Contact} Contact
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Contact.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.Contact();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.name = reader.string();
                                break;
                            }
                        case 2: {
                                message.email = reader.string();
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a Contact message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.Contact
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.Contact} Contact
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Contact.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a Contact message.
                 * @function verify
                 * @memberof exomind.base.v1.Contact
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Contact.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.name != null && message.hasOwnProperty("name"))
                        if (!$util.isString(message.name))
                            return "name: string expected";
                    if (message.email != null && message.hasOwnProperty("email"))
                        if (!$util.isString(message.email))
                            return "email: string expected";
                    return null;
                };

                /**
                 * Creates a Contact message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.Contact
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.Contact} Contact
                 */
                Contact.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.Contact)
                        return object;
                    let message = new $root.exomind.base.v1.Contact();
                    if (object.name != null)
                        message.name = String(object.name);
                    if (object.email != null)
                        message.email = String(object.email);
                    return message;
                };

                /**
                 * Creates a plain object from a Contact message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.Contact
                 * @static
                 * @param {exomind.base.v1.Contact} message Contact
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Contact.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.name = "";
                        object.email = "";
                    }
                    if (message.name != null && message.hasOwnProperty("name"))
                        object.name = message.name;
                    if (message.email != null && message.hasOwnProperty("email"))
                        object.email = message.email;
                    return object;
                };

                /**
                 * Converts this Contact to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.Contact
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Contact.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for Contact
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.Contact
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                Contact.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.Contact";
                };

                return Contact;
            })();

            v1.Task = (function() {

                /**
                 * Properties of a Task.
                 * @memberof exomind.base.v1
                 * @interface ITask
                 * @property {string|null} [title] Task title
                 */

                /**
                 * Constructs a new Task.
                 * @memberof exomind.base.v1
                 * @classdesc Represents a Task.
                 * @implements ITask
                 * @constructor
                 * @param {exomind.base.v1.ITask=} [properties] Properties to set
                 */
                function Task(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Task title.
                 * @member {string} title
                 * @memberof exomind.base.v1.Task
                 * @instance
                 */
                Task.prototype.title = "";

                /**
                 * Creates a new Task instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.Task
                 * @static
                 * @param {exomind.base.v1.ITask=} [properties] Properties to set
                 * @returns {exomind.base.v1.Task} Task instance
                 */
                Task.create = function create(properties) {
                    return new Task(properties);
                };

                /**
                 * Encodes the specified Task message. Does not implicitly {@link exomind.base.v1.Task.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.Task
                 * @static
                 * @param {exomind.base.v1.ITask} message Task message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Task.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.title != null && Object.hasOwnProperty.call(message, "title"))
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.title);
                    return writer;
                };

                /**
                 * Encodes the specified Task message, length delimited. Does not implicitly {@link exomind.base.v1.Task.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.Task
                 * @static
                 * @param {exomind.base.v1.ITask} message Task message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Task.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a Task message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.Task
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.Task} Task
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Task.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.Task();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.title = reader.string();
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a Task message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.Task
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.Task} Task
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Task.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a Task message.
                 * @function verify
                 * @memberof exomind.base.v1.Task
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Task.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.title != null && message.hasOwnProperty("title"))
                        if (!$util.isString(message.title))
                            return "title: string expected";
                    return null;
                };

                /**
                 * Creates a Task message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.Task
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.Task} Task
                 */
                Task.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.Task)
                        return object;
                    let message = new $root.exomind.base.v1.Task();
                    if (object.title != null)
                        message.title = String(object.title);
                    return message;
                };

                /**
                 * Creates a plain object from a Task message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.Task
                 * @static
                 * @param {exomind.base.v1.Task} message Task
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Task.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults)
                        object.title = "";
                    if (message.title != null && message.hasOwnProperty("title"))
                        object.title = message.title;
                    return object;
                };

                /**
                 * Converts this Task to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.Task
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Task.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for Task
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.Task
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                Task.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.Task";
                };

                return Task;
            })();

            v1.Link = (function() {

                /**
                 * Properties of a Link.
                 * @memberof exomind.base.v1
                 * @interface ILink
                 * @property {string|null} [url] Link url
                 * @property {string|null} [title] Link title
                 */

                /**
                 * Constructs a new Link.
                 * @memberof exomind.base.v1
                 * @classdesc Represents a Link.
                 * @implements ILink
                 * @constructor
                 * @param {exomind.base.v1.ILink=} [properties] Properties to set
                 */
                function Link(properties) {
                    if (properties)
                        for (let keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                            if (properties[keys[i]] != null)
                                this[keys[i]] = properties[keys[i]];
                }

                /**
                 * Link url.
                 * @member {string} url
                 * @memberof exomind.base.v1.Link
                 * @instance
                 */
                Link.prototype.url = "";

                /**
                 * Link title.
                 * @member {string} title
                 * @memberof exomind.base.v1.Link
                 * @instance
                 */
                Link.prototype.title = "";

                /**
                 * Creates a new Link instance using the specified properties.
                 * @function create
                 * @memberof exomind.base.v1.Link
                 * @static
                 * @param {exomind.base.v1.ILink=} [properties] Properties to set
                 * @returns {exomind.base.v1.Link} Link instance
                 */
                Link.create = function create(properties) {
                    return new Link(properties);
                };

                /**
                 * Encodes the specified Link message. Does not implicitly {@link exomind.base.v1.Link.verify|verify} messages.
                 * @function encode
                 * @memberof exomind.base.v1.Link
                 * @static
                 * @param {exomind.base.v1.ILink} message Link message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Link.encode = function encode(message, writer) {
                    if (!writer)
                        writer = $Writer.create();
                    if (message.url != null && Object.hasOwnProperty.call(message, "url"))
                        writer.uint32(/* id 1, wireType 2 =*/10).string(message.url);
                    if (message.title != null && Object.hasOwnProperty.call(message, "title"))
                        writer.uint32(/* id 2, wireType 2 =*/18).string(message.title);
                    return writer;
                };

                /**
                 * Encodes the specified Link message, length delimited. Does not implicitly {@link exomind.base.v1.Link.verify|verify} messages.
                 * @function encodeDelimited
                 * @memberof exomind.base.v1.Link
                 * @static
                 * @param {exomind.base.v1.ILink} message Link message or plain object to encode
                 * @param {$protobuf.Writer} [writer] Writer to encode to
                 * @returns {$protobuf.Writer} Writer
                 */
                Link.encodeDelimited = function encodeDelimited(message, writer) {
                    return this.encode(message, writer).ldelim();
                };

                /**
                 * Decodes a Link message from the specified reader or buffer.
                 * @function decode
                 * @memberof exomind.base.v1.Link
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @param {number} [length] Message length if known beforehand
                 * @returns {exomind.base.v1.Link} Link
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Link.decode = function decode(reader, length) {
                    if (!(reader instanceof $Reader))
                        reader = $Reader.create(reader);
                    let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exomind.base.v1.Link();
                    while (reader.pos < end) {
                        let tag = reader.uint32();
                        switch (tag >>> 3) {
                        case 1: {
                                message.url = reader.string();
                                break;
                            }
                        case 2: {
                                message.title = reader.string();
                                break;
                            }
                        default:
                            reader.skipType(tag & 7);
                            break;
                        }
                    }
                    return message;
                };

                /**
                 * Decodes a Link message from the specified reader or buffer, length delimited.
                 * @function decodeDelimited
                 * @memberof exomind.base.v1.Link
                 * @static
                 * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
                 * @returns {exomind.base.v1.Link} Link
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                Link.decodeDelimited = function decodeDelimited(reader) {
                    if (!(reader instanceof $Reader))
                        reader = new $Reader(reader);
                    return this.decode(reader, reader.uint32());
                };

                /**
                 * Verifies a Link message.
                 * @function verify
                 * @memberof exomind.base.v1.Link
                 * @static
                 * @param {Object.<string,*>} message Plain object to verify
                 * @returns {string|null} `null` if valid, otherwise the reason why it is not
                 */
                Link.verify = function verify(message) {
                    if (typeof message !== "object" || message === null)
                        return "object expected";
                    if (message.url != null && message.hasOwnProperty("url"))
                        if (!$util.isString(message.url))
                            return "url: string expected";
                    if (message.title != null && message.hasOwnProperty("title"))
                        if (!$util.isString(message.title))
                            return "title: string expected";
                    return null;
                };

                /**
                 * Creates a Link message from a plain object. Also converts values to their respective internal types.
                 * @function fromObject
                 * @memberof exomind.base.v1.Link
                 * @static
                 * @param {Object.<string,*>} object Plain object
                 * @returns {exomind.base.v1.Link} Link
                 */
                Link.fromObject = function fromObject(object) {
                    if (object instanceof $root.exomind.base.v1.Link)
                        return object;
                    let message = new $root.exomind.base.v1.Link();
                    if (object.url != null)
                        message.url = String(object.url);
                    if (object.title != null)
                        message.title = String(object.title);
                    return message;
                };

                /**
                 * Creates a plain object from a Link message. Also converts values to other types if specified.
                 * @function toObject
                 * @memberof exomind.base.v1.Link
                 * @static
                 * @param {exomind.base.v1.Link} message Link
                 * @param {$protobuf.IConversionOptions} [options] Conversion options
                 * @returns {Object.<string,*>} Plain object
                 */
                Link.toObject = function toObject(message, options) {
                    if (!options)
                        options = {};
                    let object = {};
                    if (options.defaults) {
                        object.url = "";
                        object.title = "";
                    }
                    if (message.url != null && message.hasOwnProperty("url"))
                        object.url = message.url;
                    if (message.title != null && message.hasOwnProperty("title"))
                        object.title = message.title;
                    return object;
                };

                /**
                 * Converts this Link to JSON.
                 * @function toJSON
                 * @memberof exomind.base.v1.Link
                 * @instance
                 * @returns {Object.<string,*>} JSON object
                 */
                Link.prototype.toJSON = function toJSON() {
                    return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
                };

                /**
                 * Gets the default type url for Link
                 * @function getTypeUrl
                 * @memberof exomind.base.v1.Link
                 * @static
                 * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns {string} The default type url
                 */
                Link.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                    if (typeUrlPrefix === undefined) {
                        typeUrlPrefix = "type.googleapis.com";
                    }
                    return typeUrlPrefix + "/exomind.base.v1.Link";
                };

                return Link;
            })();

            return v1;
        })();

        return base;
    })();

    return exomind;
})();

export const exocore = $root.exocore = (() => {

    /**
     * Namespace exocore.
     * @exports exocore
     * @namespace
     */
    const exocore = {};

    exocore.store = (function() {

        /**
         * Namespace store.
         * @memberof exocore
         * @namespace
         */
        const store = {};

        store.Reference = (function() {

            /**
             * Properties of a Reference.
             * @memberof exocore.store
             * @interface IReference
             * @property {string|null} [entityId] Reference entityId
             * @property {string|null} [traitId] Reference traitId
             */

            /**
             * Constructs a new Reference.
             * @memberof exocore.store
             * @classdesc Represents a Reference.
             * @implements IReference
             * @constructor
             * @param {exocore.store.IReference=} [properties] Properties to set
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
             * @memberof exocore.store.Reference
             * @instance
             */
            Reference.prototype.entityId = "";

            /**
             * Reference traitId.
             * @member {string} traitId
             * @memberof exocore.store.Reference
             * @instance
             */
            Reference.prototype.traitId = "";

            /**
             * Creates a new Reference instance using the specified properties.
             * @function create
             * @memberof exocore.store.Reference
             * @static
             * @param {exocore.store.IReference=} [properties] Properties to set
             * @returns {exocore.store.Reference} Reference instance
             */
            Reference.create = function create(properties) {
                return new Reference(properties);
            };

            /**
             * Encodes the specified Reference message. Does not implicitly {@link exocore.store.Reference.verify|verify} messages.
             * @function encode
             * @memberof exocore.store.Reference
             * @static
             * @param {exocore.store.IReference} message Reference message or plain object to encode
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
             * Encodes the specified Reference message, length delimited. Does not implicitly {@link exocore.store.Reference.verify|verify} messages.
             * @function encodeDelimited
             * @memberof exocore.store.Reference
             * @static
             * @param {exocore.store.IReference} message Reference message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Reference.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };

            /**
             * Decodes a Reference message from the specified reader or buffer.
             * @function decode
             * @memberof exocore.store.Reference
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {exocore.store.Reference} Reference
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Reference.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                let end = length === undefined ? reader.len : reader.pos + length, message = new $root.exocore.store.Reference();
                while (reader.pos < end) {
                    let tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1: {
                            message.entityId = reader.string();
                            break;
                        }
                    case 2: {
                            message.traitId = reader.string();
                            break;
                        }
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
             * @memberof exocore.store.Reference
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {exocore.store.Reference} Reference
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
             * @memberof exocore.store.Reference
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
             * @memberof exocore.store.Reference
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {exocore.store.Reference} Reference
             */
            Reference.fromObject = function fromObject(object) {
                if (object instanceof $root.exocore.store.Reference)
                    return object;
                let message = new $root.exocore.store.Reference();
                if (object.entityId != null)
                    message.entityId = String(object.entityId);
                if (object.traitId != null)
                    message.traitId = String(object.traitId);
                return message;
            };

            /**
             * Creates a plain object from a Reference message. Also converts values to other types if specified.
             * @function toObject
             * @memberof exocore.store.Reference
             * @static
             * @param {exocore.store.Reference} message Reference
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
             * @memberof exocore.store.Reference
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Reference.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };

            /**
             * Gets the default type url for Reference
             * @function getTypeUrl
             * @memberof exocore.store.Reference
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            Reference.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/exocore.store.Reference";
            };

            return Reference;
        })();

        return store;
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
                    case 1: {
                            message.seconds = reader.int64();
                            break;
                        }
                    case 2: {
                            message.nanos = reader.int32();
                            break;
                        }
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

            /**
             * Gets the default type url for Timestamp
             * @function getTypeUrl
             * @memberof google.protobuf.Timestamp
             * @static
             * @param {string} [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns {string} The default type url
             */
            Timestamp.getTypeUrl = function getTypeUrl(typeUrlPrefix) {
                if (typeUrlPrefix === undefined) {
                    typeUrlPrefix = "type.googleapis.com";
                }
                return typeUrlPrefix + "/google.protobuf.Timestamp";
            };

            return Timestamp;
        })();

        return protobuf;
    })();

    return google;
})();

export { $root as default };

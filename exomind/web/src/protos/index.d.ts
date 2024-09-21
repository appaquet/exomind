import * as $protobuf from "protobufjs";
import Long = require("long");
/** Namespace exomind. */
export namespace exomind {

    /** Namespace base. */
    namespace base {

        /** Namespace v1. */
        namespace v1 {

            /** Properties of a Collection. */
            interface ICollection {

                /** Collection name */
                name?: (string|null);

                /** Collection description */
                description?: (string|null);
            }

            /** Represents a Collection. */
            class Collection implements ICollection {

                /**
                 * Constructs a new Collection.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.ICollection);

                /** Collection name. */
                public name: string;

                /** Collection description. */
                public description: string;

                /**
                 * Creates a new Collection instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Collection instance
                 */
                public static create(properties?: exomind.base.v1.ICollection): exomind.base.v1.Collection;

                /**
                 * Encodes the specified Collection message. Does not implicitly {@link exomind.base.v1.Collection.verify|verify} messages.
                 * @param message Collection message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.ICollection, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Collection message, length delimited. Does not implicitly {@link exomind.base.v1.Collection.verify|verify} messages.
                 * @param message Collection message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.ICollection, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes a Collection message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Collection
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.Collection;

                /**
                 * Decodes a Collection message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Collection
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.Collection;

                /**
                 * Verifies a Collection message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates a Collection message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Collection
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.Collection;

                /**
                 * Creates a plain object from a Collection message. Also converts values to other types if specified.
                 * @param message Collection
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.Collection, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Collection to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Collection
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of a CollectionChild. */
            interface ICollectionChild {

                /** CollectionChild collection */
                collection?: (exocore.store.IReference|null);

                /** CollectionChild weight */
                weight?: (number|Long|null);
            }

            /** Represents a CollectionChild. */
            class CollectionChild implements ICollectionChild {

                /**
                 * Constructs a new CollectionChild.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.ICollectionChild);

                /** CollectionChild collection. */
                public collection?: (exocore.store.IReference|null);

                /** CollectionChild weight. */
                public weight: (number|Long);

                /**
                 * Creates a new CollectionChild instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns CollectionChild instance
                 */
                public static create(properties?: exomind.base.v1.ICollectionChild): exomind.base.v1.CollectionChild;

                /**
                 * Encodes the specified CollectionChild message. Does not implicitly {@link exomind.base.v1.CollectionChild.verify|verify} messages.
                 * @param message CollectionChild message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.ICollectionChild, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified CollectionChild message, length delimited. Does not implicitly {@link exomind.base.v1.CollectionChild.verify|verify} messages.
                 * @param message CollectionChild message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.ICollectionChild, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes a CollectionChild message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns CollectionChild
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.CollectionChild;

                /**
                 * Decodes a CollectionChild message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns CollectionChild
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.CollectionChild;

                /**
                 * Verifies a CollectionChild message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates a CollectionChild message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns CollectionChild
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.CollectionChild;

                /**
                 * Creates a plain object from a CollectionChild message. Also converts values to other types if specified.
                 * @param message CollectionChild
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.CollectionChild, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this CollectionChild to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for CollectionChild
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of a Snoozed. */
            interface ISnoozed {

                /** Snoozed untilDate */
                untilDate?: (google.protobuf.ITimestamp|null);
            }

            /** Represents a Snoozed. */
            class Snoozed implements ISnoozed {

                /**
                 * Constructs a new Snoozed.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.ISnoozed);

                /** Snoozed untilDate. */
                public untilDate?: (google.protobuf.ITimestamp|null);

                /**
                 * Creates a new Snoozed instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Snoozed instance
                 */
                public static create(properties?: exomind.base.v1.ISnoozed): exomind.base.v1.Snoozed;

                /**
                 * Encodes the specified Snoozed message. Does not implicitly {@link exomind.base.v1.Snoozed.verify|verify} messages.
                 * @param message Snoozed message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.ISnoozed, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Snoozed message, length delimited. Does not implicitly {@link exomind.base.v1.Snoozed.verify|verify} messages.
                 * @param message Snoozed message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.ISnoozed, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes a Snoozed message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Snoozed
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.Snoozed;

                /**
                 * Decodes a Snoozed message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Snoozed
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.Snoozed;

                /**
                 * Verifies a Snoozed message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates a Snoozed message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Snoozed
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.Snoozed;

                /**
                 * Creates a plain object from a Snoozed message. Also converts values to other types if specified.
                 * @param message Snoozed
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.Snoozed, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Snoozed to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Snoozed
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of an Unread. */
            interface IUnread {

                /** Unread entity */
                entity?: (exocore.store.IReference|null);
            }

            /** Represents an Unread. */
            class Unread implements IUnread {

                /**
                 * Constructs a new Unread.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.IUnread);

                /** Unread entity. */
                public entity?: (exocore.store.IReference|null);

                /**
                 * Creates a new Unread instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Unread instance
                 */
                public static create(properties?: exomind.base.v1.IUnread): exomind.base.v1.Unread;

                /**
                 * Encodes the specified Unread message. Does not implicitly {@link exomind.base.v1.Unread.verify|verify} messages.
                 * @param message Unread message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.IUnread, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Unread message, length delimited. Does not implicitly {@link exomind.base.v1.Unread.verify|verify} messages.
                 * @param message Unread message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.IUnread, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes an Unread message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Unread
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.Unread;

                /**
                 * Decodes an Unread message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Unread
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.Unread;

                /**
                 * Verifies an Unread message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates an Unread message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Unread
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.Unread;

                /**
                 * Creates a plain object from an Unread message. Also converts values to other types if specified.
                 * @param message Unread
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.Unread, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Unread to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Unread
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** AccountType enum. */
            enum AccountType {
                ACCOUNT_TYPE_INVALID = 0,
                ACCOUNT_TYPE_GMAIL = 1
            }

            /** AccountScope enum. */
            enum AccountScope {
                ACCOUNT_SCOPE_INVALID = 0,
                ACCOUNT_SCOPE_EMAIL = 1
            }

            /** Properties of an Account. */
            interface IAccount {

                /** Account key */
                key?: (string|null);

                /** Account name */
                name?: (string|null);

                /** Account type */
                type?: (exomind.base.v1.AccountType|null);

                /** Account scopes */
                scopes?: (exomind.base.v1.AccountScope[]|null);

                /** Account data */
                data?: ({ [k: string]: string }|null);
            }

            /** Represents an Account. */
            class Account implements IAccount {

                /**
                 * Constructs a new Account.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.IAccount);

                /** Account key. */
                public key: string;

                /** Account name. */
                public name: string;

                /** Account type. */
                public type: exomind.base.v1.AccountType;

                /** Account scopes. */
                public scopes: exomind.base.v1.AccountScope[];

                /** Account data. */
                public data: { [k: string]: string };

                /**
                 * Creates a new Account instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Account instance
                 */
                public static create(properties?: exomind.base.v1.IAccount): exomind.base.v1.Account;

                /**
                 * Encodes the specified Account message. Does not implicitly {@link exomind.base.v1.Account.verify|verify} messages.
                 * @param message Account message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.IAccount, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Account message, length delimited. Does not implicitly {@link exomind.base.v1.Account.verify|verify} messages.
                 * @param message Account message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.IAccount, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes an Account message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Account
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.Account;

                /**
                 * Decodes an Account message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Account
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.Account;

                /**
                 * Verifies an Account message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates an Account message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Account
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.Account;

                /**
                 * Creates a plain object from an Account message. Also converts values to other types if specified.
                 * @param message Account
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.Account, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Account to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Account
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of an EmailThread. */
            interface IEmailThread {

                /** EmailThread account */
                account?: (exocore.store.IReference|null);

                /** EmailThread sourceId */
                sourceId?: (string|null);

                /** EmailThread from */
                from?: (exomind.base.v1.IContact|null);

                /** EmailThread subject */
                subject?: (string|null);

                /** EmailThread snippet */
                snippet?: (string|null);

                /** EmailThread lastEmail */
                lastEmail?: (exocore.store.IReference|null);

                /** EmailThread read */
                read?: (boolean|null);
            }

            /** Represents an EmailThread. */
            class EmailThread implements IEmailThread {

                /**
                 * Constructs a new EmailThread.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.IEmailThread);

                /** EmailThread account. */
                public account?: (exocore.store.IReference|null);

                /** EmailThread sourceId. */
                public sourceId: string;

                /** EmailThread from. */
                public from?: (exomind.base.v1.IContact|null);

                /** EmailThread subject. */
                public subject: string;

                /** EmailThread snippet. */
                public snippet: string;

                /** EmailThread lastEmail. */
                public lastEmail?: (exocore.store.IReference|null);

                /** EmailThread read. */
                public read: boolean;

                /**
                 * Creates a new EmailThread instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns EmailThread instance
                 */
                public static create(properties?: exomind.base.v1.IEmailThread): exomind.base.v1.EmailThread;

                /**
                 * Encodes the specified EmailThread message. Does not implicitly {@link exomind.base.v1.EmailThread.verify|verify} messages.
                 * @param message EmailThread message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.IEmailThread, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified EmailThread message, length delimited. Does not implicitly {@link exomind.base.v1.EmailThread.verify|verify} messages.
                 * @param message EmailThread message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.IEmailThread, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes an EmailThread message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns EmailThread
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.EmailThread;

                /**
                 * Decodes an EmailThread message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns EmailThread
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.EmailThread;

                /**
                 * Verifies an EmailThread message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates an EmailThread message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns EmailThread
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.EmailThread;

                /**
                 * Creates a plain object from an EmailThread message. Also converts values to other types if specified.
                 * @param message EmailThread
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.EmailThread, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this EmailThread to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for EmailThread
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of an Email. */
            interface IEmail {

                /** Email account */
                account?: (exocore.store.IReference|null);

                /** Email sourceId */
                sourceId?: (string|null);

                /** Email from */
                from?: (exomind.base.v1.IContact|null);

                /** Email receivedDate */
                receivedDate?: (google.protobuf.ITimestamp|null);

                /** Email to */
                to?: (exomind.base.v1.IContact[]|null);

                /** Email cc */
                cc?: (exomind.base.v1.IContact[]|null);

                /** Email bcc */
                bcc?: (exomind.base.v1.IContact[]|null);

                /** Email subject */
                subject?: (string|null);

                /** Email snippet */
                snippet?: (string|null);

                /** Email parts */
                parts?: (exomind.base.v1.IEmailPart[]|null);

                /** Email attachments */
                attachments?: (exomind.base.v1.IEmailAttachment[]|null);

                /** Email read */
                read?: (boolean|null);
            }

            /** Represents an Email. */
            class Email implements IEmail {

                /**
                 * Constructs a new Email.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.IEmail);

                /** Email account. */
                public account?: (exocore.store.IReference|null);

                /** Email sourceId. */
                public sourceId: string;

                /** Email from. */
                public from?: (exomind.base.v1.IContact|null);

                /** Email receivedDate. */
                public receivedDate?: (google.protobuf.ITimestamp|null);

                /** Email to. */
                public to: exomind.base.v1.IContact[];

                /** Email cc. */
                public cc: exomind.base.v1.IContact[];

                /** Email bcc. */
                public bcc: exomind.base.v1.IContact[];

                /** Email subject. */
                public subject: string;

                /** Email snippet. */
                public snippet: string;

                /** Email parts. */
                public parts: exomind.base.v1.IEmailPart[];

                /** Email attachments. */
                public attachments: exomind.base.v1.IEmailAttachment[];

                /** Email read. */
                public read: boolean;

                /**
                 * Creates a new Email instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Email instance
                 */
                public static create(properties?: exomind.base.v1.IEmail): exomind.base.v1.Email;

                /**
                 * Encodes the specified Email message. Does not implicitly {@link exomind.base.v1.Email.verify|verify} messages.
                 * @param message Email message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.IEmail, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Email message, length delimited. Does not implicitly {@link exomind.base.v1.Email.verify|verify} messages.
                 * @param message Email message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.IEmail, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes an Email message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Email
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.Email;

                /**
                 * Decodes an Email message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Email
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.Email;

                /**
                 * Verifies an Email message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates an Email message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Email
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.Email;

                /**
                 * Creates a plain object from an Email message. Also converts values to other types if specified.
                 * @param message Email
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.Email, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Email to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Email
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of a DraftEmail. */
            interface IDraftEmail {

                /** DraftEmail account */
                account?: (exocore.store.IReference|null);

                /** DraftEmail inReplyTo */
                inReplyTo?: (exocore.store.IReference|null);

                /** DraftEmail to */
                to?: (exomind.base.v1.IContact[]|null);

                /** DraftEmail cc */
                cc?: (exomind.base.v1.IContact[]|null);

                /** DraftEmail bcc */
                bcc?: (exomind.base.v1.IContact[]|null);

                /** DraftEmail subject */
                subject?: (string|null);

                /** DraftEmail parts */
                parts?: (exomind.base.v1.IEmailPart[]|null);

                /** DraftEmail attachments */
                attachments?: (exomind.base.v1.IEmailAttachment[]|null);

                /** DraftEmail sendingDate */
                sendingDate?: (google.protobuf.ITimestamp|null);

                /** DraftEmail sentDate */
                sentDate?: (google.protobuf.ITimestamp|null);
            }

            /** Represents a DraftEmail. */
            class DraftEmail implements IDraftEmail {

                /**
                 * Constructs a new DraftEmail.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.IDraftEmail);

                /** DraftEmail account. */
                public account?: (exocore.store.IReference|null);

                /** DraftEmail inReplyTo. */
                public inReplyTo?: (exocore.store.IReference|null);

                /** DraftEmail to. */
                public to: exomind.base.v1.IContact[];

                /** DraftEmail cc. */
                public cc: exomind.base.v1.IContact[];

                /** DraftEmail bcc. */
                public bcc: exomind.base.v1.IContact[];

                /** DraftEmail subject. */
                public subject: string;

                /** DraftEmail parts. */
                public parts: exomind.base.v1.IEmailPart[];

                /** DraftEmail attachments. */
                public attachments: exomind.base.v1.IEmailAttachment[];

                /** DraftEmail sendingDate. */
                public sendingDate?: (google.protobuf.ITimestamp|null);

                /** DraftEmail sentDate. */
                public sentDate?: (google.protobuf.ITimestamp|null);

                /**
                 * Creates a new DraftEmail instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns DraftEmail instance
                 */
                public static create(properties?: exomind.base.v1.IDraftEmail): exomind.base.v1.DraftEmail;

                /**
                 * Encodes the specified DraftEmail message. Does not implicitly {@link exomind.base.v1.DraftEmail.verify|verify} messages.
                 * @param message DraftEmail message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.IDraftEmail, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified DraftEmail message, length delimited. Does not implicitly {@link exomind.base.v1.DraftEmail.verify|verify} messages.
                 * @param message DraftEmail message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.IDraftEmail, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes a DraftEmail message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns DraftEmail
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.DraftEmail;

                /**
                 * Decodes a DraftEmail message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns DraftEmail
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.DraftEmail;

                /**
                 * Verifies a DraftEmail message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates a DraftEmail message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns DraftEmail
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.DraftEmail;

                /**
                 * Creates a plain object from a DraftEmail message. Also converts values to other types if specified.
                 * @param message DraftEmail
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.DraftEmail, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this DraftEmail to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for DraftEmail
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of an EmailPart. */
            interface IEmailPart {

                /** EmailPart mimeType */
                mimeType?: (string|null);

                /** EmailPart body */
                body?: (string|null);
            }

            /** Represents an EmailPart. */
            class EmailPart implements IEmailPart {

                /**
                 * Constructs a new EmailPart.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.IEmailPart);

                /** EmailPart mimeType. */
                public mimeType: string;

                /** EmailPart body. */
                public body: string;

                /**
                 * Creates a new EmailPart instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns EmailPart instance
                 */
                public static create(properties?: exomind.base.v1.IEmailPart): exomind.base.v1.EmailPart;

                /**
                 * Encodes the specified EmailPart message. Does not implicitly {@link exomind.base.v1.EmailPart.verify|verify} messages.
                 * @param message EmailPart message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.IEmailPart, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified EmailPart message, length delimited. Does not implicitly {@link exomind.base.v1.EmailPart.verify|verify} messages.
                 * @param message EmailPart message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.IEmailPart, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes an EmailPart message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns EmailPart
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.EmailPart;

                /**
                 * Decodes an EmailPart message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns EmailPart
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.EmailPart;

                /**
                 * Verifies an EmailPart message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates an EmailPart message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns EmailPart
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.EmailPart;

                /**
                 * Creates a plain object from an EmailPart message. Also converts values to other types if specified.
                 * @param message EmailPart
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.EmailPart, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this EmailPart to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for EmailPart
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of an EmailAttachment. */
            interface IEmailAttachment {

                /** EmailAttachment key */
                key?: (string|null);

                /** EmailAttachment name */
                name?: (string|null);

                /** EmailAttachment mimeType */
                mimeType?: (string|null);

                /** EmailAttachment size */
                size?: (number|Long|null);

                /** EmailAttachment inlinePlaceholder */
                inlinePlaceholder?: (string|null);

                /** EmailAttachment data */
                data?: ({ [k: string]: string }|null);
            }

            /** Represents an EmailAttachment. */
            class EmailAttachment implements IEmailAttachment {

                /**
                 * Constructs a new EmailAttachment.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.IEmailAttachment);

                /** EmailAttachment key. */
                public key: string;

                /** EmailAttachment name. */
                public name: string;

                /** EmailAttachment mimeType. */
                public mimeType: string;

                /** EmailAttachment size. */
                public size: (number|Long);

                /** EmailAttachment inlinePlaceholder. */
                public inlinePlaceholder: string;

                /** EmailAttachment data. */
                public data: { [k: string]: string };

                /**
                 * Creates a new EmailAttachment instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns EmailAttachment instance
                 */
                public static create(properties?: exomind.base.v1.IEmailAttachment): exomind.base.v1.EmailAttachment;

                /**
                 * Encodes the specified EmailAttachment message. Does not implicitly {@link exomind.base.v1.EmailAttachment.verify|verify} messages.
                 * @param message EmailAttachment message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.IEmailAttachment, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified EmailAttachment message, length delimited. Does not implicitly {@link exomind.base.v1.EmailAttachment.verify|verify} messages.
                 * @param message EmailAttachment message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.IEmailAttachment, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes an EmailAttachment message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns EmailAttachment
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.EmailAttachment;

                /**
                 * Decodes an EmailAttachment message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns EmailAttachment
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.EmailAttachment;

                /**
                 * Verifies an EmailAttachment message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates an EmailAttachment message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns EmailAttachment
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.EmailAttachment;

                /**
                 * Creates a plain object from an EmailAttachment message. Also converts values to other types if specified.
                 * @param message EmailAttachment
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.EmailAttachment, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this EmailAttachment to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for EmailAttachment
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of a Note. */
            interface INote {

                /** Note title */
                title?: (string|null);

                /** Note body */
                body?: (string|null);
            }

            /** Represents a Note. */
            class Note implements INote {

                /**
                 * Constructs a new Note.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.INote);

                /** Note title. */
                public title: string;

                /** Note body. */
                public body: string;

                /**
                 * Creates a new Note instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Note instance
                 */
                public static create(properties?: exomind.base.v1.INote): exomind.base.v1.Note;

                /**
                 * Encodes the specified Note message. Does not implicitly {@link exomind.base.v1.Note.verify|verify} messages.
                 * @param message Note message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.INote, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Note message, length delimited. Does not implicitly {@link exomind.base.v1.Note.verify|verify} messages.
                 * @param message Note message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.INote, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes a Note message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Note
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.Note;

                /**
                 * Decodes a Note message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Note
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.Note;

                /**
                 * Verifies a Note message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates a Note message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Note
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.Note;

                /**
                 * Creates a plain object from a Note message. Also converts values to other types if specified.
                 * @param message Note
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.Note, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Note to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Note
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of a Contact. */
            interface IContact {

                /** Contact name */
                name?: (string|null);

                /** Contact email */
                email?: (string|null);
            }

            /** Represents a Contact. */
            class Contact implements IContact {

                /**
                 * Constructs a new Contact.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.IContact);

                /** Contact name. */
                public name: string;

                /** Contact email. */
                public email: string;

                /**
                 * Creates a new Contact instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Contact instance
                 */
                public static create(properties?: exomind.base.v1.IContact): exomind.base.v1.Contact;

                /**
                 * Encodes the specified Contact message. Does not implicitly {@link exomind.base.v1.Contact.verify|verify} messages.
                 * @param message Contact message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.IContact, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Contact message, length delimited. Does not implicitly {@link exomind.base.v1.Contact.verify|verify} messages.
                 * @param message Contact message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.IContact, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes a Contact message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Contact
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.Contact;

                /**
                 * Decodes a Contact message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Contact
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.Contact;

                /**
                 * Verifies a Contact message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates a Contact message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Contact
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.Contact;

                /**
                 * Creates a plain object from a Contact message. Also converts values to other types if specified.
                 * @param message Contact
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.Contact, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Contact to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Contact
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of a Task. */
            interface ITask {

                /** Task title */
                title?: (string|null);
            }

            /** Represents a Task. */
            class Task implements ITask {

                /**
                 * Constructs a new Task.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.ITask);

                /** Task title. */
                public title: string;

                /**
                 * Creates a new Task instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Task instance
                 */
                public static create(properties?: exomind.base.v1.ITask): exomind.base.v1.Task;

                /**
                 * Encodes the specified Task message. Does not implicitly {@link exomind.base.v1.Task.verify|verify} messages.
                 * @param message Task message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.ITask, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Task message, length delimited. Does not implicitly {@link exomind.base.v1.Task.verify|verify} messages.
                 * @param message Task message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.ITask, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes a Task message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Task
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.Task;

                /**
                 * Decodes a Task message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Task
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.Task;

                /**
                 * Verifies a Task message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates a Task message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Task
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.Task;

                /**
                 * Creates a plain object from a Task message. Also converts values to other types if specified.
                 * @param message Task
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.Task, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Task to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Task
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }

            /** Properties of a Link. */
            interface ILink {

                /** Link url */
                url?: (string|null);

                /** Link title */
                title?: (string|null);
            }

            /** Represents a Link. */
            class Link implements ILink {

                /**
                 * Constructs a new Link.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: exomind.base.v1.ILink);

                /** Link url. */
                public url: string;

                /** Link title. */
                public title: string;

                /**
                 * Creates a new Link instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Link instance
                 */
                public static create(properties?: exomind.base.v1.ILink): exomind.base.v1.Link;

                /**
                 * Encodes the specified Link message. Does not implicitly {@link exomind.base.v1.Link.verify|verify} messages.
                 * @param message Link message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: exomind.base.v1.ILink, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Link message, length delimited. Does not implicitly {@link exomind.base.v1.Link.verify|verify} messages.
                 * @param message Link message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: exomind.base.v1.ILink, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes a Link message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Link
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exomind.base.v1.Link;

                /**
                 * Decodes a Link message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Link
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exomind.base.v1.Link;

                /**
                 * Verifies a Link message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates a Link message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Link
                 */
                public static fromObject(object: { [k: string]: any }): exomind.base.v1.Link;

                /**
                 * Creates a plain object from a Link message. Also converts values to other types if specified.
                 * @param message Link
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: exomind.base.v1.Link, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Link to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Link
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }
        }
    }
}

/** Namespace exocore. */
export namespace exocore {

    /** Namespace store. */
    namespace store {

        /** Properties of a Reference. */
        interface IReference {

            /** Reference entityId */
            entityId?: (string|null);

            /** Reference traitId */
            traitId?: (string|null);
        }

        /** Represents a Reference. */
        class Reference implements IReference {

            /**
             * Constructs a new Reference.
             * @param [properties] Properties to set
             */
            constructor(properties?: exocore.store.IReference);

            /** Reference entityId. */
            public entityId: string;

            /** Reference traitId. */
            public traitId: string;

            /**
             * Creates a new Reference instance using the specified properties.
             * @param [properties] Properties to set
             * @returns Reference instance
             */
            public static create(properties?: exocore.store.IReference): exocore.store.Reference;

            /**
             * Encodes the specified Reference message. Does not implicitly {@link exocore.store.Reference.verify|verify} messages.
             * @param message Reference message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: exocore.store.IReference, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified Reference message, length delimited. Does not implicitly {@link exocore.store.Reference.verify|verify} messages.
             * @param message Reference message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: exocore.store.IReference, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a Reference message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns Reference
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): exocore.store.Reference;

            /**
             * Decodes a Reference message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns Reference
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): exocore.store.Reference;

            /**
             * Verifies a Reference message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a Reference message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns Reference
             */
            public static fromObject(object: { [k: string]: any }): exocore.store.Reference;

            /**
             * Creates a plain object from a Reference message. Also converts values to other types if specified.
             * @param message Reference
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: exocore.store.Reference, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this Reference to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for Reference
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }
    }
}

/** Namespace google. */
export namespace google {

    /** Namespace protobuf. */
    namespace protobuf {

        /** Properties of a Timestamp. */
        interface ITimestamp {

            /** Timestamp seconds */
            seconds?: (number|Long|null);

            /** Timestamp nanos */
            nanos?: (number|null);
        }

        /** Represents a Timestamp. */
        class Timestamp implements ITimestamp {

            /**
             * Constructs a new Timestamp.
             * @param [properties] Properties to set
             */
            constructor(properties?: google.protobuf.ITimestamp);

            /** Timestamp seconds. */
            public seconds: (number|Long);

            /** Timestamp nanos. */
            public nanos: number;

            /**
             * Creates a new Timestamp instance using the specified properties.
             * @param [properties] Properties to set
             * @returns Timestamp instance
             */
            public static create(properties?: google.protobuf.ITimestamp): google.protobuf.Timestamp;

            /**
             * Encodes the specified Timestamp message. Does not implicitly {@link google.protobuf.Timestamp.verify|verify} messages.
             * @param message Timestamp message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: google.protobuf.ITimestamp, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified Timestamp message, length delimited. Does not implicitly {@link google.protobuf.Timestamp.verify|verify} messages.
             * @param message Timestamp message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: google.protobuf.ITimestamp, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a Timestamp message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns Timestamp
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): google.protobuf.Timestamp;

            /**
             * Decodes a Timestamp message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns Timestamp
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): google.protobuf.Timestamp;

            /**
             * Verifies a Timestamp message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a Timestamp message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns Timestamp
             */
            public static fromObject(object: { [k: string]: any }): google.protobuf.Timestamp;

            /**
             * Creates a plain object from a Timestamp message. Also converts values to other types if specified.
             * @param message Timestamp
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: google.protobuf.Timestamp, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this Timestamp to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for Timestamp
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }
    }
}

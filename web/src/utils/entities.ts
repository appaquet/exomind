import { Exocore, exocore, fromProtoTimestamp, MutationBuilder } from 'exocore';
import Emojis from '../utils/emojis';
import { exomind } from '../protos';
import _ from 'lodash';

export class EntityTraits {
    public entity: exocore.store.IEntity;

    private typeTraits: { [type: string]: exocore.store.ITrait[] } = {};
    private idTraits: { [id: string]: exocore.store.ITrait } = {};
    private idInstances: { [id: string]: EntityTrait<unknown> } = {};
    private priorityTraitId?: string;

    constructor(entity: exocore.store.IEntity) {
        this.entity = entity;

        // check if it's a special entity (ex: inbox)
        let priorityTrait: [exocore.store.ITrait, ITraitConstants] = null;
        if (this.entity.id in TRAITS_CONSTANTS) {
            const traitsConstants = TRAITS_CONSTANTS[this.entity.id];
            for (const trait of this.entity.traits) {
                if (trait.id == this.entity.id) {
                    priorityTrait = [trait, traitsConstants];
                    break;
                }
            }
        }

        // index traits by ids and types
        for (const trait of this.entity.traits) {
            const msgType = Exocore.registry.canonicalFullName(trait.message.type_url);
            if (!(msgType in this.typeTraits)) {
                this.typeTraits[msgType] = [trait];
            } else {
                this.typeTraits[msgType].push(trait);
            }

            this.idTraits[trait.id] = trait;

            let traitConstants;
            if (this.entity.id == trait.id && this.entity.id in TRAITS_CONSTANTS) {
                // special entity
                traitConstants = TRAITS_CONSTANTS[this.entity.id];
            } else if (msgType in TRAITS_CONSTANTS) {
                traitConstants = TRAITS_CONSTANTS[msgType];
            }

            if (traitConstants && ((priorityTrait == null || traitConstants.order < priorityTrait[1].order))) {
                priorityTrait = [trait, traitConstants];
            }
        }

        if (priorityTrait != null) {
            this.priorityTraitId = priorityTrait[0].id;
        } else if (this.entity.traits.length > 0) {
            this.priorityTraitId = this.entity.traits[0].id;
        }
    }

    get id(): string {
        return this.entity.id;
    }

    traitOfType<T>(msg: unknown): EntityTrait<T> | null {
        const traits = this.traitsOfType<T>(msg);
        if (traits.length > 0) {
            return traits[0];
        } else {
            return null;
        }
    }

    traitsOfType<T>(msg: unknown): EntityTrait<T>[] {
        const msgType = Exocore.registry.messageFullName(msg);
        return this.entity.traits.flatMap((trait: exocore.store.ITrait) => {
            if (trait.message.type_url.endsWith(msgType)) {
                return [this.trait(trait.id)];
            } else {
                return [];
            }
        });
    }

    trait<T>(id: string): EntityTrait<T> | null {
        const trait = this.idTraits[id];
        if (!trait) {
            return null;
        }

        if (!(id in this.idInstances)) {
            const et = new EntityTrait<unknown>(
                this,
                trait,
                Exocore.registry.unpackAny(trait.message),
            );
            this.idInstances[id] = et;

            return et as EntityTrait<T>;
        }

        return this.idInstances[id] as EntityTrait<T>;
    }

    get priorityTrait(): EntityTrait<unknown> | null {
        if (this.priorityTraitId != null) {
            return this.trait(this.priorityTraitId);
        } else {
            return null;
        }
    }

    priorityMatch(matcher: ITraitMatcher): unknown {
        const priorityTrait = this.priorityTrait;
        if (priorityTrait == null) {
            return null;
        }

        return priorityTrait.match(matcher);
    }
}

export class EntityTrait<T> {
    trait: exocore.store.ITrait;
    message: T;
    et: EntityTraits;

    constructor(et: EntityTraits, trait: exocore.store.ITrait, message: T) {
        this.et = et;
        this.trait = trait;
        this.message = message;
    }

    get id(): string {
        return this.trait.id;
    }

    get constants(): ITraitConstants {
        const msgType = Exocore.registry.messageFullName(this.message);
        if (this.trait.id == this.et.id && this.et.id in TRAITS_CONSTANTS) {
            // special entity
            return TRAITS_CONSTANTS[this.et.id];
        } else if (msgType in TRAITS_CONSTANTS) {
            return TRAITS_CONSTANTS[msgType];
        } else {
            return TRAITS_CONSTANTS['unknown'];
        }
    }

    get creationDate(): Date | null {
        if (this.trait.creationDate != null) {
            return fromProtoTimestamp(this.trait.creationDate);
        } else {
            return null;
        }
    }

    get modificationDate(): Date | null {
        if (this.trait.modificationDate != null) {
            return fromProtoTimestamp(this.trait.modificationDate);
        } else {
            return null;
        }
    }

    get displayName(): string {
        if (this.constants.nameFunction) {
            return this.constants.nameFunction(this.message);
        }

        if (this.constants.name) {
            return this.constants.name;
        }

        if (this.constants.nameField) {
            const dict = this.message as unknown as { [p: string]: string; };
            const name = dict[this.constants.nameField];
            return name ?? this.constants.nameDefault ?? '*UNTITLED*';
        }

        return '*UNTITLED*';
    }

    get icon(): TraitIcon {
        return this.constants.icon(this.message);
    }

    match(matcher: ITraitMatcher): unknown {
        if (this.constants.key == 'exomind.base.v1.EmailThread' && matcher.emailThread) {
            return matcher.emailThread(this);
        } else if (this.constants.key == 'exomind.base.v1.Email' && matcher.email) {
            return matcher.email(this);
        } else if (this.constants.key == 'exomind.base.v1.DraftEmail' && matcher.draftEmail) {
            return matcher.draftEmail(this);
        } else if (this.constants.key == 'exomind.base.v1.Collection' || (this.constants?.collectionLike ?? false) && matcher.collection) {
            return matcher.collection(this);
        } else if (this.constants.key == 'exomind.base.v1.Task' && matcher.task) {
            return matcher.task(this);
        } else if (this.constants.key == 'exomind.base.v1.Note' && matcher.note) {
            return matcher.note(this);
        } else if (this.constants.key == 'exomind.base.v1.Link' && matcher.link) {
            return matcher.link(this);
        } else if (matcher.default) {
            return matcher.default();
        }
    }

    get canEditName(): boolean {
        return !!this.constants.rename;
    }

    get editableName(): string {
        if (this.constants.renameValue) {
            return this.constants.renameValue(this.message);
        } else {
            return this.displayName;
        }
    }

    async rename(newName: string): Promise<exocore.store.IMutationResult> {
        if (!this.constants.rename) {
            return;
        }


        const newMsg = _.clone(this.message);
        this.constants.rename(newMsg, newName);

        await Exocore.store.mutate(
            MutationBuilder
                .updateEntity(this.et.id)
                .putTrait(newMsg, this.id)
                .build()
        );
    }
}

export function isEntityTraits(object: unknown): object is EntityTraits {
    return object instanceof EntityTraits;
}

export interface ITraitMatcher {
    emailThread?: (trait: EntityTrait<exomind.base.v1.IEmailThread>) => unknown;
    email?: (trait: EntityTrait<exomind.base.v1.IEmail>) => unknown;
    draftEmail?: (trait: EntityTrait<exomind.base.v1.IDraftEmail>) => unknown;
    collection?: (trait: EntityTrait<exomind.base.v1.ICollection>) => unknown;
    task?: (trait: EntityTrait<exomind.base.v1.ITask>) => unknown;
    note?: (trait: EntityTrait<exomind.base.v1.INote>) => unknown;
    link?: (trait: EntityTrait<exomind.base.v1.ILink>) => unknown;
    default?: () => unknown;
}

export interface ITraitConstants {
    key: string;
    name?: string;
    nameField?: string;
    nameDefault?: string;
    nameFunction?: (trait: unknown) => string;
    icon: (trait: unknown) => TraitIcon;
    color: number;
    order: number;
    collectionLike?: boolean;
    renameValue?: (trait: unknown) => string;
    rename?: (trait: unknown, newName: string) => void;
}

export type TraitIcon = { fa: string } | { emoji: string };

export const TRAITS_CONSTANTS: { [type: string]: ITraitConstants } = {
    'inbox': {
        key: 'inbox',
        name: 'Inbox',
        icon: () => { return { fa: 'inbox' }; },
        collectionLike: true,
        color: 4,
        order: 0
    },
    'favorites': {
        key: 'favorites',
        name: 'Favorites',
        icon: () => { return { fa: 'star' }; },
        collectionLike: true,
        color: 4,
        order: 1
    },
    'exomind.base.v1.EmailThread': {
        key: 'exomind.base.v1.EmailThread',
        nameField: 'subject',
        nameDefault: 'Untitled email',
        icon: () => { return { fa: 'envelope-o' }; },
        color: 1,
        order: 2
    },
    'exomind.base.v1.DraftEmail': {
        key: 'exomind.base.v1.DraftEmail',
        nameField: 'subject',
        nameDefault: 'Untitled email',
        icon: () => { return { fa: 'envelope-o' }; },
        color: 6,
        order: 3
    },
    'exomind.base.v1.Email': {
        key: 'exomind.base.v1.Email',
        nameField: 'subject',
        nameDefault: 'Untitled email',
        icon: () => { return { fa: 'envelope-o' }; },
        color: 6,
        order: 4
    },
    'exomind.base.v1.Collection': {
        key: 'exomind.base.v1.Collection',
        nameFunction: (trait) => {
            const col = trait as exomind.base.v1.ICollection;

            if (col.name) {
                if (Emojis.hasEmojiPrefix(col.name)) {
                    const [, title] = Emojis.extractEmojiPrefix(col.name);
                    return title;
                } else {
                    return col.name;
                }
            } else {
                return 'Untitled collection';
            }
        },
        icon: (trait) => {
            const col = trait as exomind.base.v1.ICollection;
            if (Emojis.hasEmojiPrefix(col.name)) {
                const [emoji] = Emojis.extractEmojiPrefix(col.name);
                return { emoji };
            } else {
                return { fa: 'folder-o' };
            }
        },
        color: 2,
        order: 5,
        collectionLike: true,
        renameValue: (trait: unknown) => {
            const col = trait as exomind.base.v1.ICollection;
            return col.name;
        },
        rename: (trait: unknown, newName: string): void => {
            const collection = trait as exomind.base.v1.ICollection;
            collection.name = newName;
        },
    },
    'exomind.base.v1.Task': {
        key: 'exomind.base.v1.Task',
        nameField: 'title',
        nameDefault: 'Untitled task',
        icon: () => { return { fa: 'check-square-o' }; },
        color: 7,
        order: 6,
        rename: (entity: unknown, newName: string): void => {
            const task = entity as exomind.base.v1.ITask;
            task.title = newName;
        },
    },
    'exomind.base.v1.Note': {
        key: 'exomind.base.v1.Note',
        nameField: 'title',
        nameDefault: 'Untitled note',
        icon: () => { return { fa: 'pencil' }; },
        color: 3,
        order: 7,
        rename: (entity: unknown, newName: string): void => {
            const note = entity as exomind.base.v1.INote;
            note.title = newName;
        },
    },
    'exomind.base.v1.Link': {
        key: 'exomind.base.v1.Link',
        nameField: 'title',
        nameDefault: 'Untitled link',
        icon: () => { return { fa: 'link' }; },
        color: 9,
        order: 8
    },
    'unknown': {
        key: 'unknown',
        nameField: '*UNKNOWN*',
        icon: () => { return { fa: 'question' }; },
        color: 0,
        order: 9
    }
};

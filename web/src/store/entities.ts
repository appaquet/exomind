import { Exocore, exocore, fromProtoTimestamp, MutationBuilder } from 'exocore';
import { exomind } from '../protos';

export class EntityTraits {
    public entity: exocore.index.IEntity;

    private typeTraits: { [type: string]: exocore.index.ITrait[] } = {}
    private traitMessages: { [id: string]: exocore.index.ITrait } = {}
    private traitInstances: { [id: string]: EntityTrait<unknown> } = {}
    private priorityTraitId?: string

    constructor(entity: exocore.index.IEntity) {
        this.entity = entity;

        let priorityTrait: [exocore.index.ITrait, ITraitConstants] = null;
        if (this.entity.id in TRAITS_CONSTANTS) {
            const traitsConsts = TRAITS_CONSTANTS[this.entity.id];
            for (const trait of this.entity.traits) {
                if (trait.id == this.entity.id) {
                    priorityTrait = [trait, traitsConsts];
                    break;
                }
            }
        }

        for (const trait of this.entity.traits) {
            const msgType = Exocore.registry.canonicalFullName(trait.message.type_url);
            if (!(msgType in this.typeTraits)) {
                this.typeTraits[msgType] = [trait];
            } else {
                this.typeTraits[msgType].push(trait);
            }

            this.traitMessages[trait.id] = trait;

            let traitConsts;
            if (this.entity.id == trait.id && this.entity.id in TRAITS_CONSTANTS) {
                // special entity
                traitConsts = TRAITS_CONSTANTS[this.entity.id];
            } else if (msgType in TRAITS_CONSTANTS) {
                traitConsts = TRAITS_CONSTANTS[msgType];
            }

            if (traitConsts && ((priorityTrait == null || traitConsts.order < priorityTrait[1].order))) {
                priorityTrait = [trait, traitConsts];
            }
        }

        if (priorityTrait != null) {
            this.priorityTraitId = priorityTrait[0].id;
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
        return this.entity.traits.flatMap((trait: exocore.index.ITrait) => {
            if (trait.message.type_url.endsWith(msgType)) {
                return [this.trait(trait.id)];
            } else {
                return [];
            }
        });
    }

    trait<T>(id: string): EntityTrait<T> | null {
        const trait = this.traitMessages[id];
        if (!trait) {
            return null;
        }

        if (!(id in this.traitInstances)) {
            const et = new EntityTrait<unknown>(
                this,
                trait,
                Exocore.registry.unpackAny(trait.message),
            );
            this.traitInstances[id] = et;

            return et as EntityTrait<T>;
        }

        return this.traitInstances[id] as EntityTrait<T>;
    }

    traits(): EntityTrait<unknown>[] {
        return this.entity.traits.map((trait: exocore.index.ITrait) => {
            return this.trait(trait.id);
        });
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

    actions(container: string, trait?: EntityTrait<unknown>): EntityActionXYZ[] {
        const finalTrait = trait ?? this.priorityTrait;

        const actions: EntityActionXYZ[] = [];
        if (finalTrait.canRename) {
            actions.push(new EntityActionXYZ('pencil', async () => {
                const newName = prompt('New name', finalTrait.displayName);
                if (newName) {
                    await finalTrait.rename(newName);
                }
                return ActionResult.success();
            }));
        }

        return actions;
    }
}

export class EntityActionXYZ {
    private _trigger: () => Promise<ActionResult>;

    constructor(public icon: string, trigger: () => Promise<ActionResult>) {
        this._trigger = trigger;
    }

    async trigger(): Promise<ActionResult> {
        const result = await this._trigger();
        return result;
    }
}

export class ActionResult {
    public result?: unknown;
    public cancelled = false;
    public removed = false;

    static success(result?: unknown, removed?: boolean): ActionResult {
        const res = new ActionResult();
        res.result = result;
        res.removed = removed ?? false;
        return res;
    }

    static cancelled(): ActionResult {
        const res = new ActionResult();
        res.cancelled = true;
        return res;
    }
}

export class EntityTrait<T> {
    trait: exocore.index.ITrait;
    message: T;
    et: EntityTraits;

    constructor(et: EntityTraits, trait: exocore.index.ITrait, message: T) {
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
        if (this.constants.name) {
            return this.constants.name;
        }

        if (this.constants.name_field) {
            const dict = this.message as unknown as { [p: string]: string; };
            const name = dict[this.constants.name_field];
            return name ?? this.constants.name_default ?? '**UNTITLED**';
        }

        return '*UNTITLED*';
    }

    get icon(): string {
        return this.constants.icon;
    }

    match(matcher: ITraitMatcher): unknown {
        if (this.constants.key == 'exomind.base.EmailThread' && matcher.emailThread) {
            return matcher.emailThread(this);
        } else if (this.constants.key == 'exomind.base.Email' && matcher.email) {
            return matcher.email(this);
        } else if (this.constants.key == 'exomind.base.DraftEmail' && matcher.draftEmail) {
            return matcher.draftEmail(this);
        } else if (this.constants.key == 'exomind.base.Collection' || (this.constants?.collectionLike ?? false) && matcher.collection) {
            return matcher.collection(this);
        } else if (this.constants.key == 'exomind.base.Task' && matcher.task) {
            return matcher.task(this);
        } else if (this.constants.key == 'exomind.base.Note' && matcher.note) {
            return matcher.note(this);
        } else if (this.constants.key == 'exomind.base.Link' && matcher.link) {
            return matcher.link(this);
        } else if (matcher.default) {
            return matcher.default();
        }
    }

    get canRename(): boolean {
        return !!this.constants.rename;
    }

    async rename(newName: string): Promise<exocore.index.IMutationResult> {
        if (!this.constants.rename) {
            return;
        }

        this.constants.rename(this.message, newName);
        await Exocore.store.mutate(
            MutationBuilder
                .updateEntity(this.et.id)
                .putTrait(this.message, this.id)
                .build()
        );
    }
}

export interface ITraitMatcher {
    emailThread?: (trait: EntityTrait<exomind.base.IEmailThread>) => unknown;
    email?: (trait: EntityTrait<exomind.base.IEmail>) => unknown;
    draftEmail?: (trait: EntityTrait<exomind.base.IDraftEmail>) => unknown;
    collection?: (trait: EntityTrait<exomind.base.ICollection>) => unknown;
    task?: (trait: EntityTrait<exomind.base.ITask>) => unknown;
    note?: (trait: EntityTrait<exomind.base.INote>) => unknown;
    link?: (trait: EntityTrait<exomind.base.ILink>) => unknown;
    default?: () => unknown;
}

export interface ITraitConstants {
    key: string;
    name?: string;
    name_field?: string;
    name_default?: string;
    icon: string;
    color: number;
    order: number;
    collectionLike?: boolean;
    rename?: (trait: unknown, newName: string) => void;
}

export const TRAITS_CONSTANTS: { [type: string]: ITraitConstants } = {
    'inbox': {
        key: 'inbox',
        name: 'Inbox',
        icon: 'inbox',
        collectionLike: true,
        color: 4,
        order: 0
    },
    'favorites': {
        key: 'favorites',
        name: 'Favorites',
        icon: 'star',
        collectionLike: true,
        color: 4,
        order: 1
    },
    'exomind.integration': {
        key: 'exomind.integration',
        name_field: 'key',
        icon: 'plug',
        color: 4,
        order: 1
    },
    'exomind.base.EmailThread': {
        key: 'exomind.base.EmailThread',
        name_field: 'subject',
        name_default: 'Untitled email',
        icon: 'envelope-o',
        color: 1,
        order: 2
    },
    'exomind.base.DraftEmail': {
        key: 'exomind.base.DraftEmail',
        name_field: 'subject',
        name_default: 'Untitled email',
        icon: 'envelope-o',
        color: 6,
        order: 3
    },
    'exomind.base.Email': {
        key: 'exomind.base.Email',
        name_field: 'subject',
        name_default: 'Untitled email',
        icon: 'envelope-o',
        color: 6,
        order: 4
    },
    'exomind.base.Collection': {
        key: 'exomind.base.Collection',
        name_field: 'name',
        icon: 'folder-o',
        color: 2,
        order: 5,
        rename: (entity: unknown, newName: string): void => {
            const collection = entity as exomind.base.ICollection;
            collection.name = newName;
        },
    },
    'exomind.base.Task': {
        key: 'exomind.base.Task',
        name_field: 'title',
        icon: 'check-square-o',
        color: 7,
        order: 6,
        rename: (entity: unknown, newName: string): void => {
            const task = entity as exomind.base.ITask;
            task.title = newName;
        },
    },
    'exomind.base.Note': {
        key: 'exomind.base.Note',
        name_field: 'title',
        icon: 'pencil',
        color: 3,
        order: 7,
        rename: (entity: unknown, newName: string): void => {
            const note = entity as exomind.base.INote;
            note.title = newName;
        },
    },
    'exomind.base.Link': {
        key: 'exomind.base.Link',
        name_field: 'title',
        name_default: 'Untitled link',
        icon: 'link',
        color: 9,
        order: 8
    },
    'unknown': {
        key: 'unknown',
        name_field: '*UNKNOWN*',
        icon: 'question',
        color: 0,
        order: 9
    }
};

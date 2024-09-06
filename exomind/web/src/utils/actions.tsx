import { getEntityParentRelation, hasEntityParent, isPinnedInParent } from "../stores/collections";
import { EntityTraits } from "./entities";
import { CancellableEvent } from "./events";
import { Stores } from "../stores/stores";
import { CollectionSelector } from "../components/modals/collection-selector/collection-selector";
import React from "react";
import { Commands, IEntityCreateResult } from "./commands";
import copy from 'clipboard-copy';
import TimeSelector from "../components/modals/time-selector/time-selector";
import { exomind } from '../protos';
import _ from "lodash";
import InputModal from "../components/modals/input-modal/input-modal";
import Navigation from "../navigation";

export type ActionIcon = ActionFaIcon;
export type ActionFaIcon = string
export type SiteSection = 'inbox' | 'recent' | 'search' | 'snoozed';

export interface IActionResult {
    remove?: boolean;
    createResult?: IEntityCreateResult;
}

export interface IAction {
    key: string;
    label: string;
    icon?: string;
    execute: (e: CancellableEvent) => Promise<IActionResult>;
    priority?: number;
    disabled?: boolean;
}

export interface IContext {
    parent?: EntityTraits | string;
    section?: SiteSection;
}

export class Actions {
    static forEntity(entity: EntityTraits, context: IContext | null = null): IAction[] {
        const actions: IAction[] = [];
        const push = (priority: number, action: IAction) => {
            action.priority = priority;
            actions.push(action);
        };

        const parentId = Commands.getEntityId(context?.parent);
        const parentRel = parentId ? getEntityParentRelation(entity, parentId) : null;

        if (parentRel) {
            push(10, this.removeFromParent(entity, context?.parent));
        }

        if (!this.isSpecialEntity(entity.id)) {
            if (context?.section !== 'snoozed') {
                push(18, this.snooze(entity, parentId, parentId === 'inbox'));
            }

            if (parentId !== 'inbox' && !hasEntityParent(entity, 'inbox')) {
                push(13, this.addToInbox(entity));
            }
        }

        if (parentId && !this.isSpecialEntity(parentId)) {
            if (!isPinnedInParent(entity, parentId)) {
                push(30, this.pinInParent(entity, parentId));
            } else {
                push(31, this.unpinInParent(entity, parentId));
            }
        }

        if (parentId) {
            push(32, this.moveTopParent(entity, parentId));
        }

        if (context?.section === 'snoozed') {
            push(10, this.removeSnooze(entity));
        }

        if (this.isNoteEntity(entity)) {
            push(45, this.popOutToWindow(entity));
        }

        push(20, this.selectEntityCollections(entity));

        push(40, this.copyLink(entity));

        push(50, this.delete(entity));

        return _.sortBy(actions, (a) => a.priority);
    }

    static forSelectedEntities(entities: EntityTraits[], context: IContext | null = null): IAction[] {
        const actions: IAction[] = [];
        const push = (priority: number, action: IAction) => {
            action.priority = priority;
            actions.push(action);
        };

        const parentId = Commands.getEntityId(context?.parent);
        if (parentId) {
            push(10, this.removeFromParent(entities, context?.parent));
        }

        push(20, this.snooze(entities, parentId, parentId === 'inbox'));

        push(30, this.selectEntityCollections(entities));

        return _.sortBy(actions, (a) => a.priority);
    }

    static forEntityCreation(parent: EntityTraits | string): IAction[] {
        const parentId = Commands.getEntityId(parent);

        const actions: IAction[] = [];
        const push = (priority: number, action: IAction) => {
            action.priority = priority;
            actions.push(action);
        };

        push(10, this.createNote(parentId));
        push(11, this.createCollection(parentId));
        push(12, this.createLink(parentId));
        push(13, this.createTask(parentId));

        return _.sortBy(actions, (a) => a.priority);
    }

    static removeFromParent(et: EntityTraits | EntityTraits[], parent: EntityTraits | string): IAction {
        return {
            key: 'remove-from-parent',
            label: 'Remove',
            icon: 'check',
            execute: async () => {
                await Commands.removeFromParent(et, parent);
                return {
                    remove: true,
                };
            }
        };
    }

    static selectEntityCollections(et: EntityTraits | EntityTraits[]): IAction {
        return {
            key: 'select-entity-collections',
            label: 'Add to collections...',
            icon: 'folder-open-o',
            execute: async () => {
                Stores.session.showModal(() => {
                    return <CollectionSelector entities={et} />;
                });
                return {};
            },
        };
    }

    static pinInParent(et: EntityTraits, parent: EntityTraits | string): IAction {
        return {
            key: 'pin-in-parent',
            label: 'Pin to top',
            icon: 'thumb-tack',
            execute: async () => {
                await Commands.pinEntityInParent(et, parent);
                return {};
            }
        };
    }

    static unpinInParent(et: EntityTraits, parent: EntityTraits | string): IAction {
        return {
            key: 'unpin-in-parent',
            label: 'Unpin from top',
            icon: 'thumb-tack',
            execute: async () => {
                await Commands.unpinEntityInParent(et, parent);
                return {};
            }
        };
    }

    static moveTopParent(et: EntityTraits | EntityTraits[], parent: EntityTraits | string): IAction {
        return {
            key: 'move-top-parent',
            label: 'Move to top',
            icon: 'arrow-up',
            execute: async () => {
                await Commands.addToParent(et, parent);
                return {};
            }
        };
    }

    static addToInbox(et: EntityTraits): IAction {
        return {
            key: 'add-to-inbox',
            label: 'Move to inbox',
            icon: 'inbox',
            execute: async () => {
                await Commands.addToParent(et, 'inbox');
                return {};
            }
        };
    }

    static copyLink(et: EntityTraits): IAction {
        return {
            key: 'copy-link',
            label: 'Copy link',
            icon: 'link',
            execute: async () => {
                copy(`entity://${et.id}`);
                return {};
            }
        };
    }

    static snooze(et: EntityTraits | EntityTraits[], parent: EntityTraits | string | null = null, removeFromParent = false): IAction {
        return {
            key: 'snooze',
            label: 'Snooze...',
            icon: 'clock-o',
            execute: async () => {
                return new Promise((resolve) => {
                    const handleSnooze = async (date: Date) => {
                        Stores.session.hideModal();
                        await Commands.snooze(et, date, parent, removeFromParent);
                        resolve({
                            remove: removeFromParent,
                        });
                    };

                    Stores.session.showModal(() => {
                        return <TimeSelector onSelectionDone={handleSnooze} />;
                    });
                });
            }
        };
    }

    static removeSnooze(et: EntityTraits): IAction {
        return {
            key: 'remove-snooze',
            label: 'Unsnooze',
            icon: 'clock-o',
            execute: async () => {
                await Commands.removeSnooze(et);
                await Commands.addToParent(et, 'inbox');
                return {};
            }
        };
    }

    static delete(et: EntityTraits): IAction {
        return {
            key: 'delete',
            label: 'Delete',
            icon: 'trash',
            execute: async () => {
                if (confirm('Are you sure you want to delete this entity?')) {
                    await Commands.delete(et);
                }
                return {};
            }
        };
    }

    static createNote(parent: EntityTraits | string): IAction {
        return {
            key: 'create-note',
            label: 'Create note',
            icon: 'pencil',
            execute: async () => {
                const createResult = await Commands.createNote(parent);
                return { createResult };
            }
        };
    }

    static createTask(parent: EntityTraits | string): IAction {
        return {
            key: 'create-task',
            label: 'Create task',
            icon: 'check-square-o',
            execute: async () => {
                const createResult = await Commands.createTask(parent);
                return { createResult };
            }
        };
    }

    static createCollection(parent: EntityTraits | string): IAction {
        return {
            key: 'create-collection',
            label: 'Create collection',
            icon: 'folder-o',
            execute: async () => {
                const createResult = await Commands.createCollection(parent);
                return { createResult };
            }
        };
    }

    static createLink(parent: EntityTraits | string): IAction {
        return {
            key: 'create-link',
            label: 'Create link',
            icon: 'link',
            execute: async () => {
                return new Promise((resolve) => {
                    const handleLink = async (url: string) => {
                        Stores.session.hideModal();

                        if (!url) {
                            return;
                        }

                        const createResult = await Commands.createLink(parent, url, url);
                        resolve({ createResult });
                    };

                    Stores.session.showModal(() => {
                        return <InputModal
                            text="URL of the link"
                            onDone={handleLink} />;
                    });
                });
            }
        };
    }

    static popOutToWindow(et: EntityTraits): IAction {
        return {
            key: 'pop-out',
            label: 'Pop out',
            icon: 'external-link',
            execute: async () => {
                Navigation.navigatePopup(Navigation.pathForFullscreen(et.id));
                return {};
            }
        };
    }

    private static isSpecialEntity(entityId: string): boolean {
        switch (entityId) {
            case 'inbox':
                return true;
            case 'favorites':
                return true;
            default:
                return false;
        }
    }

    private static isNoteEntity(entity: EntityTraits): boolean {
        return !!entity.traitOfType<exomind.base.v1.INote>(exomind.base.v1.Note);
    }
}
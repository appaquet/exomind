import { getEntityParentRelation, isPinnedInParent } from "../stores/collections";
import { EntityTraits } from "./entities";
import { CancellableEvent } from "./events";
import { Stores } from "../stores/stores";
import { CollectionSelector } from "../components/modals/collection-selector/collection-selector";
import React from "react";
import { Commands } from "./commands";
import copy from 'clipboard-copy';
import TimeSelector from "../components/modals/time-selector/time-selector";
import _ from "lodash";

export type ActionIcon = ActionFaIcon;
export type ActionFaIcon = string
export type ActionResult = 'remove' | void;
export type SiteSection = 'inbox' | 'recent' | 'search' | 'snoozed';

export interface IAction {
    label: string;
    icon?: string;
    execute: (e: CancellableEvent, action: IAction) => Promise<ActionResult>;
    priority?: number;
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
            push(20, this.snooze(entity, parentId, parentId === 'inbox'));
        }

        if (parentId && !this.isSpecialEntity(parentId)) {
            if (!isPinnedInParent(entity, parentId)) {
                push(30, this.pinInParent(entity, parentId));
            } else {
                push(31, this.unpinInParent(entity, parentId));
            }
        }

        if (parentId !== 'inbox') {
            push(15, this.addToInbox(entity));
        }

        if (context?.section === 'snoozed') {
            const action = this.removeSnooze(entity);
            action.icon = 'times';
            push(10, action);
        }

        push(16, this.selectEntityCollections(entity));

        push(40, this.copyLink(entity));

        push(50, this.delete(entity));

        return _.sortBy(actions, (a) => a.priority);
    }

    static removeFromParent(et: EntityTraits, parent: EntityTraits | string): IAction {
        return {
            label: 'Remove',
            icon: 'check',
            execute: async () => {
                await Commands.removeFromParent(et, parent);
                return 'remove';
            }
        }
    }

    static selectEntityCollections(et: EntityTraits): IAction {
        return {
            label: 'Add to collections...',
            icon: 'folder-open-o',
            execute: async () => {
                Stores.session.showModal(() => {
                    return <CollectionSelector entity={et} />;
                });
            },
        }
    }

    static pinInParent(et: EntityTraits, parent: EntityTraits | string): IAction {
        return {
            label: 'Pin to top',
            icon: 'thumb-tack',
            execute: async () => {
                await Commands.pinEntityInParent(et, parent);
            }
        }
    }

    static unpinInParent(et: EntityTraits, parent: EntityTraits | string): IAction {
        return {
            label: 'Unpin from top',
            icon: 'thumb-tack',
            execute: async () => {
                await Commands.unpinEntityInParent(et, parent);
            }
        }
    }

    static addToInbox(et: EntityTraits): IAction {
        return {
            label: 'Move to inbox',
            icon: 'inbox',
            execute: async () => {
                await Commands.addToParent(et, 'inbox');
            }
        }
    }

    static copyLink(et: EntityTraits): IAction {
        return {
            label: 'Copy link',
            icon: 'link',
            execute: async () => {
                copy(`entity://${et.id}`);
            }
        }
    }

    static snooze(et: EntityTraits, parent: EntityTraits | string | null = null, removeFromParent = false): IAction {
        return {
            label: 'Snooze...',
            icon: 'clock-o',
            execute: async () => {
                return new Promise((resolve) => {
                    const handleSnooze = async (date: Date) => {
                        Stores.session.hideModal();
                        await Commands.snooze(et, date, parent, removeFromParent);
                        resolve();
                    }

                    Stores.session.showModal(() => {
                        return <TimeSelector onSelectionDone={handleSnooze} />;
                    });
                })
            }
        }
    }

    static removeSnooze(et: EntityTraits): IAction {
        return {
            label: 'Remove snooze',
            icon: 'clock-o',
            execute: async () => {
                await Commands.removeSnooze(et);
            }
        }
    }

    static delete(et: EntityTraits): IAction {
        return {
            label: 'Delete',
            icon: 'times',
            execute: async () => {
                if (confirm('Are you sure you want to delete this entity?')) {
                    await Commands.delete(et);
                }
            }
        }
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
}
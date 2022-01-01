import classNames from "classnames";
import { observer } from "mobx-react";
import React from 'react';
import { ListenerToken, Shortcuts } from "../../../shortcuts";
import { EntityTrait, EntityTraits } from "../../../utils/entities";
import DragAndDrop, { DragData } from "../../interaction/drag-and-drop/drag-and-drop";
import Scrollable from "../../interaction/scrollable/scrollable";
import { ContainerState } from "../container-state";
import { Entity } from './entity';
import { EntityActions } from "./entity-action";
import { SelectedItem, Selection } from "./selection";
import './entity-list.less';

export interface IProps {
    entities: EntityTraits[];
    parentEntity?: EntityTraits;

    onRequireLoadMore?: () => void;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    actionsForEntity?: (entity: EntityTraits) => EntityActions;

    header?: React.ReactNode;

    droppable?: boolean;
    draggable?: boolean;
    onDropIn?: (e: IDroppedItem) => void;

    containerState?: ContainerState,

    renderEntityDate?: (entity: EntityTrait<unknown>) => React.ReactFragment;
}

interface IState {
    activeEntityId?: string;
    activeEntityIndex?: number;
}

export interface IDroppedItem {
    droppedEntity?: EntityTraits;
    fromParentEntity?: EntityTraits;

    previousEntity?: EntityTraits;
    overEntity?: EntityTraits;
    nextEntity?: EntityTraits;

    data: DragData;
}

@observer
export class EntityList extends React.Component<IProps, IState> {
    private static nextListId = 0;
    private listId: number;

    private shortcutToken: ListenerToken;

    constructor(props: IProps) {
        super(props);
        this.listId = EntityList.nextListId++;
        this.shortcutToken = Shortcuts.register([
            {
                key: ['n', 'j'],
                callback: this.handleShortcutNext,
                disabledContexts: ['input', 'modal'],
            },
            {
                key: 'ArrowDown',
                callback: this.handleShortcutNext,
                disabledContexts: ['modal'], // allow focusing out of search bar
            },
            {
                key: ['p', 'k', 'ArrowUp'],
                callback: this.handleShortcutPrevious,
                disabledContexts: ['input', 'modal'],
            },
            {
                key: ['Mod-ArrowUp'],
                callback: this.handleShortcutTop,
                disabledContexts: ['input', 'modal'],
            },
            {
                key: ['Mod-ArrowDown'],
                callback: this.handleShortcutBottom,
                disabledContexts: ['input', 'modal'],
            },
            {
                key: ['Space', 'Enter'],
                callback: () => this.handleShortcutSelect(false),
                disabledContexts: ['input', 'modal'],
            },
            {
                key: 'x',
                callback: () => this.handleShortcutSelect(true),
                disabledContexts: ['input', 'modal'],
            },
            {
                key: 'Escape',
                callback: this.handleShortcutClearSelect,
                disabledContexts: ['input', 'modal'],
            },
        ]);
        this.state = {};
    }

    componentWillUnmount() {
        Shortcuts.unregister(this.shortcutToken);
    }

    render(): React.ReactNode {
        Shortcuts.setListenerEnabled(this.shortcutToken, this.props.containerState?.active ?? false);

        const classes = classNames({
            'entity-list': true,
            'header-control': !!this.props.header,
        });

        const nbItems = this.props.entities.length;
        return (
            <div className={classes}>
                <Scrollable
                    initialTopInset={(this.props.header) ? 30 : 0}
                    loadMoreItems={15}
                    onNeedMore={this.props.onRequireLoadMore}
                    nbItems={nbItems}>

                    {this.props.header}
                    {this.renderCollection()}

                </Scrollable>
            </div>
        );
    }

    private renderCollection(): React.ReactNode {
        if (this.props.entities.length == 0) {
            return this.renderEmptyList();
        }

        const count = this.props.entities.length;
        const items = this.props.entities.map((entity, idx) => {
            const selected = this.props.selection?.contains(SelectedItem.fromEntity(entity)) ?? false;

            const handleDropIn = (data: DragData) => {
                let prevEntity;
                if (idx > 0) {
                    prevEntity = this.props.entities[idx - 1];
                }

                let nextEntity;
                if (idx < count) {
                    nextEntity = this.props.entities[idx + 1];
                }

                this.handleDropIn(this.props.entities[idx], prevEntity, nextEntity, data);
            };

            return (
                <Entity
                    id={this.getEntityElementId(idx)}
                    key={entity.id}
                    entity={entity}
                    parentEntity={this.props.parentEntity}
                    active={this.state.activeEntityId === entity.id}

                    selected={selected}
                    onSelectionChange={this.props.onSelectionChange}
                    onClick={(e) => this.handleItemClick(entity, e)}
                    actionsForEntity={this.props.actionsForEntity}

                    onMouseOver={() => this.handleItemMouseOver(entity.id, idx)}
                    onMouseLeave={() => this.handleItemMouseLeave(entity.id)}

                    draggable={this.props.draggable}
                    droppable={this.props.droppable}
                    onDropIn={handleDropIn}

                    renderEntityDate={this.props.renderEntityDate}
                />
            );
        });

        return (
            <ul className="list">{items}</ul>
        );
    }

    private renderEmptyList(): React.ReactNode {
        return (
            <div className="empty">
                <DragAndDrop
                    parentObject={this.props.parentEntity}
                    onDropIn={(data: DragData) => {
                        return this.handleDropIn(null, null, null, data)
                    }}
                    draggable={false}
                    droppable={this.props.droppable}>

                    This collection is empty
                </DragAndDrop>
            </div>
        );
    }

    private handleItemMouseOver(entityId: string, idx: number): void {
        if (Shortcuts.usedRecently) {
            return;
        }

        this.setState({
            activeEntityId: entityId,
            activeEntityIndex: idx,
        });
    }

    private handleItemMouseLeave(entityId: string): void {
        if (Shortcuts.usedRecently) {
            return;
        }

        if (this.state.activeEntityId == entityId) {
            this.setState({
                activeEntityId: undefined,
                activeEntityIndex: undefined,
            });
        }
    }

    private handleShortcutPrevious = (): boolean => {
        let idx = this.state.activeEntityIndex ?? 0;
        idx--;

        if (idx < 0) {
            idx = 0;
        }

        this.hoverEntityIndex(idx);

        return true;
    }

    private handleShortcutNext = (): boolean => {
        let idx = this.state.activeEntityIndex ?? -1;
        idx++;

        if (idx > this.props.entities.length - 1) {
            idx = this.props.entities.length - 1;
        }
        
        this.hoverEntityIndex(idx);

        return true;
    }

    private handleShortcutTop = (): boolean => {
        this.hoverEntityIndex(0);
        return true;
    }

    private handleShortcutBottom = (): boolean => {
        this.hoverEntityIndex(this.props.entities.length - 1);
        return true;
    }

    private hoverEntityIndex(idx: number): void {
        const entity = this.props.entities[idx];
        if (!entity) {
            return;
        }

        const elId = this.getEntityElementId(idx);
        const el = document.getElementById(elId);
        if (el) {
            el.scrollIntoView({ behavior: 'smooth' });
        }

        this.setState({
            activeEntityId: entity.id,
            activeEntityIndex: idx,
        });
    }

    private handleShortcutSelect = (multi: boolean): boolean => {
        if (!this.state.activeEntityId) {
            return false;
        }

        const entity = this.props.entities[this.state.activeEntityIndex];
        if (!entity) {
            return;
        }

        this.selectEntity(entity, multi);
        return true;
    }

    private handleShortcutClearSelect = (): boolean => {
        this.props.onSelectionChange(new Selection());
        return true;
    }

    private handleDropIn(
        overEntity: EntityTraits,
        previousEntity: EntityTraits,
        nextEntity: EntityTraits,
        data: DragData,
    ): void {
        if (this.props.onDropIn != null) {
            this.props.onDropIn({
                fromParentEntity: data.parentObject as EntityTraits,
                droppedEntity: data.object as EntityTraits,
                previousEntity: previousEntity,
                overEntity: overEntity,
                nextEntity: nextEntity,
                data,
            });
        }
    }

    private handleItemClick(entity: EntityTraits, e: React.MouseEvent): void {
        if (this.props.onSelectionChange) {
            const special = e.shiftKey || e.altKey || e.metaKey;
            this.selectEntity(entity, special);
        }
    }

    private selectEntity(entity: EntityTraits, multi = false): void {
        let selection = this.props.selection?.copy() ?? new Selection();
        const item = SelectedItem.fromEntity(entity);

        if (selection.contains(item)) {
            selection = selection.withoutItem(item);
        } else if (multi) {
            selection = selection.withItem(item);
        } else {
            selection = new Selection([item]);
        }

        this.props.onSelectionChange(selection);
    }

    private getEntityElementId(idx: number): string {
        return `et-${this.listId}-${idx}`;
    }
}

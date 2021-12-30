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
import './entity-list.less';
import { SelectedItem, Selection } from "./selection";

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
    private shortcutToken?: ListenerToken;

    constructor(props: IProps) {
        super(props);
        this.state = {};
    }

    componentDidMount() {
        this.shortcutToken = Shortcuts.register([
            {
                key: ['n', 'ArrowDown'],
                callback: this.handleShortcutNext,
                noContext: ['input'],
            },
            {
                key: ['p', 'ArrowUp'],
                callback: this.handleShortcutPrevious,
                noContext: ['input'],
            },
            {
                key: 'Space',
                callback: () => this.handleShortcutSelect(false),
                noContext: ['input'],
            },
            {
                key: 'x',
                callback: () => this.handleShortcutSelect(true),
                noContext: ['input'],
            },
            {
                key: 'Escape',
                callback: this.handleShortcutClearSelect,
                noContext: ['input'],
            },
        ]);
    }

    componentWillUnmount() {
        if (this.shortcutToken != null) {
            Shortcuts.unregister(this.shortcutToken);
        }
    }

    render(): React.ReactNode {
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
        this.setState({
            activeEntityId: entityId,
            activeEntityIndex: idx,
        });
    }

    private handleItemMouseLeave(entityId: string): void {
        if (this.state.activeEntityId == entityId) {
            this.setState({
                activeEntityId: undefined,
                activeEntityIndex: undefined,
            });
        }
    }

    private handleShortcutPrevious = (): boolean => {
        if (!(this.props.containerState?.active ?? false)) {
            return false;
        }

        let idx = this.state.activeEntityIndex ?? this.props.entities.length - 1;
        idx--;

        if (idx < 0) {
            idx = this.props.entities.length - 1;
        }

        this.hoverIndex(idx);

        return true;
    }

    private handleShortcutNext = (): boolean => {
        if (!(this.props.containerState?.active ?? false)) {
            return false;
        }

        let idx = this.state.activeEntityIndex ?? -1;
        idx++;

        if (idx >= this.props.entities.length) {
            idx = 0;
        }

        this.hoverIndex(idx);

        return true;
    }

    private hoverIndex(idx: number): void {
        console.log('hoverIndex', idx);
        const entity = this.props.entities[idx];
        if (!entity) {
            return;
        }

        this.setState({
            activeEntityId: entity.id,
            activeEntityIndex: idx,
        });
    }

    private handleShortcutSelect = (multi: boolean): boolean => {
        if (!(this.props.containerState?.active ?? false)) {
            return false;
        }

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
        if (!(this.props.containerState?.active ?? false)) {
            return false;
        }

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
}

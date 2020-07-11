import classNames from "classnames";
import { exocore } from 'exocore';
import React from 'react';
import { EntityTraits } from "../../../store/entities";
import DragAndDrop from "../../interaction/drag-and-drop/drag-and-drop";
import Scrollable from "../../interaction/scrollable/scrollable";
import { ContainerController } from "../container-controller";
import { Entity } from './entity';
import EntityAction from "./entity-action";
import './entity-list.less';
import { SelectedItem, Selection } from "./selection";

interface IProps {
    entities: exocore.index.IEntity[];
    parentEntity?: exocore.index.IEntity;

    onRequireLoadMore?: () => void;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    actionsForEntity?: (entity: EntityTraits) => EntityAction[];

    headerControl?: React.ReactNode;

    droppable?: boolean;
    draggable?: boolean;
    onDropIn?: (e: IDroppedItem) => void;

    containerController?: ContainerController,
}

export interface IDroppedItem {
    droppedEntity?: exocore.index.IEntity;
    fromParentEntity?: exocore.index.IEntity;

    previousEntity?: exocore.index.IEntity;
    overEntity?: exocore.index.IEntity;
    nextEntity?: exocore.index.IEntity;

    effect: DropEffect,
}

export type DropEffect = ('move' | 'copy');

export class EntityList extends React.Component<IProps> {
    constructor(props: IProps) {
        super(props);
    }

    render(): React.ReactNode {
        const classes = classNames({
            'entity-list': true,
            'header-control': !!this.props.headerControl,
        });

        const nbItems = this.props.entities.length;
        return (
            <div className={classes}>
                <Scrollable
                    initialTopInset={(this.props.headerControl) ? 30 : 0}
                    loadMoreItems={15}
                    onNeedMore={this.props.onRequireLoadMore}
                    nbItems={nbItems}>

                    {this.props.headerControl}
                    {this.renderCollection()}

                </Scrollable>
            </div>
        );
    }

    private renderCollection(): React.ReactNode {
        if (this.props.entities.length > 0) {

            let previousEntity: exocore.index.IEntity = null;
            const items = this.props.entities.map((entity) => {
                const selected = this.props.selection?.contains(SelectedItem.fromEntity(entity)) ?? false;

                const item = <Entity
                    key={entity.id}
                    entity={entity}
                    parentEntity={this.props.parentEntity}

                    selected={selected}
                    onClick={this.handleItemClick.bind(this, entity)}
                    actionsForEntity={this.props.actionsForEntity}

                    draggable={this.props.draggable}
                    droppable={this.props.droppable}
                    onDropIn={this.handleDropIn.bind(this, entity, previousEntity)}
                />;

                previousEntity = entity;

                return item;
            });

            return (
                <ul className="list">{items}</ul>
            );
        } else {
            return this.renderEmptyList();
        }
    }

    private renderEmptyList(): React.ReactNode {
        return (
            <div className="empty">
                <DragAndDrop
                    parentObject={this.props.parentEntity}
                    onDropIn={this.handleDropIn.bind(this, null, null)}
                    draggable={false} droppable={this.props.droppable}>

                    This collection is empty
                </DragAndDrop>
            </div>
        );
    }

    private handleDropIn(overEntity: exocore.index.IEntity, previousEntity: exocore.index.IEntity, entity: exocore.index.IEntity, effect: DropEffect, parentEntity: exocore.index.IEntity): void {
        if (this.props.onDropIn != null) {
            this.props.onDropIn({
                effect: effect,
                fromParentEntity: parentEntity,
                droppedEntity: entity,
                previousEntity: previousEntity,
                overEntity: overEntity,
            });
        }
    }

    private handleItemClick(entity: exocore.index.IEntity, e: MouseEvent): void {
        if (this.props.onSelectionChange) {
            const special = e.shiftKey || e.altKey || e.metaKey;

            let selection = this.props.selection?.copy() ?? new Selection();
            const item = SelectedItem.fromEntity(entity);

            if (selection.contains(item)) {
                selection = selection.withoutItem(item);
            } else if (special) {
                selection = selection.withItem(item);
            } else {
                selection = new Selection([item]);
            }

            this.props.onSelectionChange(selection);
        }
    }
}

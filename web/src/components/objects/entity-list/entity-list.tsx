import classNames from "classnames";
import { exocore } from 'exocore';
import React from 'react';
import { EntityTrait, EntityTraits } from "../../../store/entities";
import DragAndDrop from "../../interaction/drag-and-drop/drag-and-drop";
import Scrollable from "../../interaction/scrollable/scrollable";
import { ContainerController } from "../container-controller";
import { DropEffect, Entity } from './entity';
import { EntityActions } from "./entity-action";
import './entity-list.less';
import { SelectedItem, Selection } from "./selection";

export interface IProps {
    entities: exocore.store.IEntity[];
    parentEntity?: exocore.store.IEntity;

    onRequireLoadMore?: () => void;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    actionsForEntity?: (entity: EntityTraits) => EntityActions;

    header?: React.ReactNode;

    droppable?: boolean;
    draggable?: boolean;
    onDropIn?: (e: IDroppedItem) => void;

    containerController?: ContainerController,

    renderEntityDate?: (entity: EntityTrait<unknown>) => React.ReactFragment;
}

export interface IDroppedItem {
    droppedEntity?: exocore.store.IEntity;
    fromParentEntity?: exocore.store.IEntity;

    previousEntity?: exocore.store.IEntity;
    overEntity?: exocore.store.IEntity;
    nextEntity?: exocore.store.IEntity;

    effect: DropEffect,
}
export class EntityList extends React.Component<IProps> {
    constructor(props: IProps) {
        super(props);
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
        if (this.props.entities.length > 0) {

            let previousEntity: exocore.store.IEntity = null;
            const items = this.props.entities.map((entity) => {
                const selected = this.props.selection?.contains(SelectedItem.fromEntity(entity)) ?? false;

                const entityCopy = entity;
                const previousEntityCopy = previousEntity;
                const item = <Entity
                    key={entity.id}
                    entity={entity}
                    parentEntity={this.props.parentEntity}

                    selected={selected}
                    onClick={(e) => this.handleItemClick(entity, e)}
                    actionsForEntity={this.props.actionsForEntity}

                    draggable={this.props.draggable}
                    droppable={this.props.droppable}
                    onDropIn={(droppedEntity, effect, droppedEntityParent) => {
                        this.handleDropIn(entityCopy, previousEntityCopy, droppedEntity, effect, droppedEntityParent);
                    }}

                    renderEntityDate={this.props.renderEntityDate}
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
                    onDropIn={(droppedEntity: exocore.store.IEntity, effect: DropEffect, parent: exocore.store.IEntity) => {
                        return this.handleDropIn(null, null, droppedEntity, effect, parent)
                    }}
                    draggable={false} droppable={this.props.droppable}>

                    This collection is empty
                </DragAndDrop>
            </div>
        );
    }

    private handleDropIn(
        overEntity: exocore.store.IEntity,
        previousEntity: exocore.store.IEntity,
        droppedEntity: exocore.store.IEntity,
        effect: DropEffect,
        parentEntity: exocore.store.IEntity,
    ): void {
        if (this.props.onDropIn != null) {
            this.props.onDropIn({
                effect: effect,
                fromParentEntity: parentEntity,
                droppedEntity: droppedEntity,
                previousEntity: previousEntity,
                overEntity: overEntity,
            });
        }
    }

    private handleItemClick(entity: exocore.store.IEntity, e: React.MouseEvent): void {
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

import classNames from "classnames";
import React from 'react';
import { EntityTrait, EntityTraits } from "../../../utils/entities";
import DragAndDrop, { DragData } from "../../interaction/drag-and-drop/drag-and-drop";
import Scrollable from "../../interaction/scrollable/scrollable";
import { ContainerController } from "../container-controller";
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

    containerController?: ContainerController,

    renderEntityDate?: (entity: EntityTrait<unknown>) => React.ReactFragment;
}

export interface IDroppedItem {
    droppedEntity?: EntityTraits;
    fromParentEntity?: EntityTraits;

    previousEntity?: EntityTraits;
    overEntity?: EntityTraits;
    nextEntity?: EntityTraits;

    data: DragData;
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

                const item = <Entity
                    key={entity.id}
                    entity={entity}
                    parentEntity={this.props.parentEntity}

                    selected={selected}
                    onSelectionChange={this.props.onSelectionChange}
                    onClick={(e) => this.handleItemClick(entity, e)}
                    actionsForEntity={this.props.actionsForEntity}

                    draggable={this.props.draggable}
                    droppable={this.props.droppable}
                    onDropIn={handleDropIn}

                    renderEntityDate={this.props.renderEntityDate}
                />;

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

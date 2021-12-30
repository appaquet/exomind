import classNames from "classnames";
import { Exocore, exocore, MutationBuilder, QueryBuilder, toProtoTimestamp, TraitQueryBuilder, WatchedQueryWrapper } from 'exocore';
import React from 'react';
import { exomind } from "../../../protos";
import { EntityTraits } from '../../../utils/entities';
import { ExpandableQuery } from "../../../stores/queries";
import { CollectionSelector } from "../../modals/collection-selector/collection-selector";
import TimeSelector from "../../modals/time-selector/time-selector";
import { ActionResult, ButtonAction, EntityActions, InlineAction } from '../entity-list/entity-action';
import { EntityList, IDroppedItem } from "../entity-list/entity-list";
import { ColumnActions } from "./column-actions";
import { SelectedItem, Selection } from "../entity-list/selection";
import { Message } from "../message";
import { IStores, StoresContext } from "../../../stores/stores";
import { getEntityParentRelation, getEntityParentWeight } from "../../../stores/collections";
import { ContainerState } from "../container-state";
import { observer } from "mobx-react";
import './children.less';

const PINNED_WEIGHT = 5000000000000;

interface IProps {
    parent?: EntityTraits;
    parentId?: string;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    onEntityAction?: (action: string, entity: EntityTraits) => void;

    actionsForEntity?: (et: EntityTraits) => string[];

    removeOnPostpone?: boolean;
    containerState?: ContainerState;
}

interface IState {
    parent?: EntityTraits;
    entities?: EntityTraits[],
    hovered: boolean;
    error?: string;
    editedEntity?: EntityTraits;
}

@observer
export class Children extends React.Component<IProps, IState> {
    static contextType = StoresContext;
    declare context: IStores;

    private entityQuery: ExpandableQuery;
    private parentQuery: WatchedQueryWrapper;
    private parentId: string;

    constructor(props: IProps) {
        super(props);

        this.parentId = props.parentId ?? props.parent.id;

        const traitQuery = TraitQueryBuilder.refersTo('collection', this.parentId).build();
        const childrenQuery = QueryBuilder
            .withTrait(exomind.base.v1.CollectionChild, traitQuery)
            .count(30)
            .orderByField('weight', false)
            .project(
                new exocore.store.Projection({
                    fieldGroupIds: [1],
                    package: ["exomind.base"],
                }),
                new exocore.store.Projection({
                    skip: true,
                })
            )
            .build();

        this.entityQuery = new ExpandableQuery(childrenQuery, () => {
            const entities = Array.from(this.entityQuery.results()).map((res) => {
                return new EntityTraits(res.entity);
            });

            this.setState({ entities });
        })

        if (!props.parent) {
            if (!props.parentId) {
                throw 'Children needs a parent or parentId prop';
            }

            const parentQuery = QueryBuilder.withIds(props.parentId).build();
            this.parentQuery = Exocore.store.watchedQuery(parentQuery);
            this.parentQuery.onChange((res) => {
                if (res.entities.length === 0) {
                    this.setState({
                        error: `Couldn't find parent entity '${props.parentId}'`,
                    });
                    return;
                }

                this.setState({
                    parent: new EntityTraits(res.entities[0].entity),
                });
            });
        }

        this.state = {
            parent: props.parent,
            hovered: false,
        };
    }

    componentWillUnmount(): void {
        this.entityQuery.free();

        if (this.parentQuery) {
            this.parentQuery.free();
        }
    }

    render(): React.ReactNode {
        if (this.state.error) {
            return <Message text={this.state.error} error={true} />;
        }

        if (this.entityQuery.hasResults && this.state.parent) {
            const classes = classNames({
                'entity-component': true,
                'children': true,
            });

            const controls = (this.props.containerState?.active ?? false) ?
                <ColumnActions
                    parent={this.state.parent}
                    selection={this.props.selection}
                    onSelectionChange={this.props.onSelectionChange}
                    onCreated={(entity) => this.handleCreatedEntity(entity)}
                    removeOnPostpone={this.props.removeOnPostpone}
                /> : null;

            return (
                <div className={classes}
                    onMouseEnter={this.handleMouseEnter}
                    onMouseLeave={this.handleMouseLeave}>

                    {this.props.children}

                    <EntityList
                        entities={this.state.entities}
                        parentEntity={this.state.parent}

                        onRequireLoadMore={this.handleLoadMore}

                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                        actionsForEntity={this.actionsForEntity}

                        onDropIn={this.handleDropInEntity}
                        containerState={this.props.containerState}
                    />

                    {controls}

                </div>
            );

        } else {
            return <Message text="Loading..." showAfterMs={200} />;
        }
    }

    private handleLoadMore = () => {
        this.entityQuery.expand();
    }

    private handleMouseEnter = () => {
        this.setState({
            hovered: true
        });
    }

    private handleMouseLeave = () => {
        this.setState({
            hovered: false
        });
    }

    private actionsForEntity = (et: EntityTraits): EntityActions => {
        if (!this.props.actionsForEntity) {
            return new EntityActions();
        }

        const actions = this.props.actionsForEntity(et);
        const buttonActions = actions.map((action) => {
            switch (action) {
                case 'done':
                    return new ButtonAction('check', () => { return this.handleEntityDone(et) });
                case 'postpone':
                    return new ButtonAction('clock-o', () => { return this.handleEntityPostpone(et) });
                case 'move':
                    return new ButtonAction('folder-open-o', () => { return this.handleEntityMoveCollection(et) });
                case 'inbox':
                    return new ButtonAction('inbox', () => { return this.handleEntityMoveInbox(et) });
                case 'pin':
                    return new ButtonAction(this.isPinned(et) ? 'caret-down' : 'thumb-tack', () => { return this.handleEntityPin(et) });
                case 'restore': {
                    const icon = (this.props.parentId == 'inbox') ? 'inbox' : 'folder-o';
                    return new ButtonAction(icon, () => { return this.handleEntityRestore(et) });
                }
            }
        });

        // when we just created an entity that require it to be edited right away (ex: task)
        let inlineEdit;
        if (this.state.editedEntity && this.state.editedEntity.id == et.id) {
            inlineEdit = new InlineAction(() => {
                this.setState({
                    editedEntity: null,
                });
            });
        }

        return new EntityActions(buttonActions, inlineEdit);
    }

    private handleEntityDone(et: EntityTraits): ActionResult {
        this.context.collections.removeEntityFromParents([et], this.parentId);

        this.removeFromSelection(et);

        if (this.props.onEntityAction) {
            this.props.onEntityAction('done', et);
        }

        return 'remove';
    }

    private handleEntityPostpone(et: EntityTraits): ActionResult {
        this.context.session.showModal(() => {
            return <TimeSelector onSelectionDone={(date) => this.handleTimeSelectorDone(et, date)} />;
        });
    }

    private handleTimeSelectorDone(et: EntityTraits, date: Date) {
        this.context.session.hideModal();

        let mb = MutationBuilder
            .updateEntity(et.id)
            .putTrait(new exomind.base.v1.Snoozed({
                untilDate: toProtoTimestamp(date),
            }), "snoozed")
            .returnEntities();

        if (this.parentId === 'inbox') {
            const parentRelation = getEntityParentRelation(et, 'inbox');
            if (parentRelation) {
                mb = mb.deleteTrait(parentRelation.id);
            }
        }

        Exocore.store.mutate(mb.build());

        if (this.props.onEntityAction) {
            this.props.onEntityAction('postpone', et);
        }
    }

    private handleEntityMoveCollection(et: EntityTraits) {
        this.context.session.showModal(() => {
            return <CollectionSelector entity={et} />;
        });
    }

    private handleEntityMoveInbox(et: EntityTraits) {
        const mb = MutationBuilder
            .updateEntity(et.id)
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: et.id,
                }),
                weight: new Date().getTime(),
            }), 'child_inbox');
        Exocore.store.mutate(mb.build());

        if (this.props.onEntityAction) {
            this.props.onEntityAction('inbox', et);
        }
    }

    private isPinned(et: EntityTraits): boolean {
        const child = et
            .traitsOfType<exomind.base.v1.ICollectionChild>(exomind.base.v1.CollectionChild)
            .filter((child) => child.message.collection.entityId == this.state.parent?.id)?.[0];
        if (!child) {
            return false;
        }

        return child.message.weight >= PINNED_WEIGHT;
    }

    private handleEntityPin(et: EntityTraits) {
        const child = et
            .traitsOfType<exomind.base.v1.ICollectionChild>(exomind.base.v1.CollectionChild)
            .filter((child) => child.message.collection.entityId == this.state.parent?.id)?.[0];
        if (!child) {
            return;
        }

        if (child.message.weight >= PINNED_WEIGHT) {
            child.message.weight = new Date().getTime();
        } else {
            child.message.weight = new Date().getTime() + PINNED_WEIGHT;
        }

        const mb = MutationBuilder
            .updateEntity(et.id)
            .putTrait(child.message, child.id);
        Exocore.store.mutate(mb.build());
    }

    private handleEntityRestore(et: EntityTraits) {
        // TODO: ExomindDSL.on(entity).relations.addParent(this.state.parentEntity);

        if (this.props.onEntityAction) {
            this.props.onEntityAction('restore', et);
        }
    }

    private handleDropInEntity = (droppedItem: IDroppedItem) => {
        const droppedEntity = droppedItem.droppedEntity;

        // calculate weight by putting it in the middle of the hovered object and the previous object so
        // that the dropped object is inserted right before the hovered object
        let parentId = this.parentId;
        let weight;
        if (droppedItem.overEntity) {
            const overEntityWeight = getEntityParentWeight(droppedItem.overEntity, parentId);

            if (droppedItem.data.position == 'before') {
                if (droppedItem.previousEntity) {
                    const previousEntityWeight = getEntityParentWeight(droppedItem.previousEntity, parentId);
                    weight = (previousEntityWeight + overEntityWeight) / 2;
                } else {
                    weight = new Date().getTime();
                }

            } else if (droppedItem.data.position == 'after') {
                if (droppedItem.nextEntity) {
                    const nextEntityWeight = getEntityParentWeight(droppedItem.nextEntity, parentId);
                    weight = (nextEntityWeight + overEntityWeight) / 2;

                } else {
                    weight = overEntityWeight - 100;
                }
            } else if (droppedItem.data.position == 'into') {
                parentId = droppedItem.overEntity.id;
                weight = new Date().getTime();
            }

        } else {
            weight = new Date().getTime();
        }

        const droppedEntityRelation = getEntityParentRelation(droppedEntity, parentId);
        const relationTraitId = droppedEntityRelation?.id ?? `child_${parentId}`;

        let mb = MutationBuilder
            .updateEntity(droppedEntity.id)
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: parentId
                }),
                weight: weight,
            }), relationTraitId)
            .returnEntities();

        // if it has been moved and it's not inside its own container, then we remove it from old parent
        if (droppedItem.data.effect === 'move' && droppedItem.fromParentEntity && parentId !== droppedItem.fromParentEntity.id) {
            const fromRelation = getEntityParentRelation(droppedEntity, droppedItem.fromParentEntity.id);
            mb = mb.deleteTrait(fromRelation.id);
        }

        Exocore.store.mutate(mb.build());

        if (this.props.onEntityAction) {
            this.props.onEntityAction('drop', droppedEntity);
        }
    }

    private handleCreatedEntity(entity: EntityTraits) {
        if (this.props.onSelectionChange && this.props.selection) {
            if (entity.traitOfType(exomind.base.v1.Task)) {
                this.setState({
                    editedEntity: entity,
                });
            } else {
                this.props.onSelectionChange(new Selection(SelectedItem.fromEntity(entity)));
            }
        }
    }

    private removeFromSelection(entity: EntityTraits) {
        if (this.props.onSelectionChange && this.props.selection) {
            const newSelection = this.props.selection.withoutItem(SelectedItem.fromEntity(entity));
            this.props.onSelectionChange(newSelection);
        }
    }
}

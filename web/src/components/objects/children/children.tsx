import classNames from "classnames";
import { Exocore, exocore, MutationBuilder, QueryBuilder, toProtoTimestamp, TraitQueryBuilder, WatchedQueryWrapper } from 'exocore';
import React from 'react';
import { exomind } from "../../../protos";
import { EntityTraits } from '../../../store/entities';
import { ModalStore } from "../../../store/modal-store";
import { ExpandableQuery } from "../../../store/queries";
import { CollectionSelector } from "../../modals/collection-selector/collection-selector";
import TimeSelector from "../../modals/time-selector/time-selector";
import { ActionResult, ButtonAction, EntityActions, InlineAction } from '../entity-list/entity-action';
import { EntityList, IDroppedItem } from "../entity-list/entity-list";
import { ListActions } from "../entity-list/list-actions";
import { SelectedItem, Selection } from "../entity-list/selection";
import { Message } from "../message";
import Long from "long";
import './children.less';

const PINNED_WEIGHT = 5000000000000;

interface IProps {
    parent?: exocore.store.IEntity;
    parentId?: string;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    onEntityAction?: (action: string, entity: exocore.store.IEntity) => void;

    actionsForEntity?: (et: EntityTraits) => string[];

    removeOnPostpone?: boolean;
}

interface IState {
    parent?: exocore.store.IEntity;
    hovered: boolean;
    error?: string;
    editedEntity?: EntityTraits;
}

export class Children extends React.Component<IProps, IState> {
    private entityQuery: ExpandableQuery;
    private parentQuery: WatchedQueryWrapper;
    private parentId: string;

    constructor(props: IProps) {
        super(props);

        this.parentId = props.parentId ?? props.parent.id;

        const traitQuery = TraitQueryBuilder.refersTo('collection', this.parentId).build();
        const childrenQuery = QueryBuilder
            .withTrait(exomind.base.CollectionChild, traitQuery)
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
            this.setState({});
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
                    parent: res.entities[0].entity,
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

            const entities = Array.from(this.entityQuery.results()).map((res) => {
                return res.entity;
            });

            const controls = (this.state.hovered) ?
                <ListActions
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

                    <EntityList
                        entities={entities}
                        parentEntity={this.state.parent}

                        onRequireLoadMore={this.handleLoadMore}

                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                        actionsForEntity={this.actionsForEntity}

                        onDropIn={this.handleDropInEntity}
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
        const mutationBuilder = MutationBuilder.updateEntity(et.entity.id);
        const colsChildren = et
            .traitsOfType<exomind.base.CollectionChild>(exomind.base.CollectionChild)
            .filter((child) => child.message.collection.entityId == this.parentId);
        if (colsChildren.length > 0) {
            for (const child of colsChildren) {
                mutationBuilder.deleteTrait(child.trait.id);
            }
            Exocore.store.mutate(mutationBuilder.build());
        }

        this.removeFromSelection(et.entity);

        if (this.props.onEntityAction) {
            this.props.onEntityAction('done', et.entity);
        }

        return 'remove';
    }

    private handleEntityPostpone(et: EntityTraits): ActionResult {
        ModalStore.showModal(() => {
            return <TimeSelector onSelectionDone={(date) => this.handleTimeSelectorDone(et, date)} />;
        });
    }

    private handleTimeSelectorDone(et: EntityTraits, date: Date) {
        ModalStore.hideModal();

        let mb = MutationBuilder
            .updateEntity(et.id)
            .putTrait(new exomind.base.Snoozed({
                untilDate: toProtoTimestamp(date),
            }), "snoozed")
            .returnEntities();

        if (this.parentId === 'inbox') {
            const parentRelation = et
                .traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild)
                .find((trt) => trt.message.collection.entityId == 'inbox')
            if (parentRelation) {
                mb = mb.deleteTrait(parentRelation.id);
            }
        }

        Exocore.store.mutate(mb.build());

        if (this.props.onEntityAction) {
            this.props.onEntityAction('postpone', et.entity);
        }
    }

    private handleEntityMoveCollection(et: EntityTraits) {
        ModalStore.showModal(() => {
            return <CollectionSelector entity={et.entity} />;
        });
    }

    private handleEntityMoveInbox(et: EntityTraits) {
        const mb = MutationBuilder
            .updateEntity(et.id)
            .putTrait(new exomind.base.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: et.id,
                }),
                weight: new Date().getTime(),
            }), 'child_inbox');
        Exocore.store.mutate(mb.build());

        if (this.props.onEntityAction) {
            this.props.onEntityAction('inbox', et.entity);
        }
    }


    private isPinned(et: EntityTraits): boolean {
        const child = et
            .traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild)
            .filter((child) => child.message.collection.entityId == this.state.parent?.id)?.[0];
        if (!child) {
            return false;
        }

        return child.message.weight >= PINNED_WEIGHT;
    }

    private handleEntityPin(et: EntityTraits) {
        const child = et
            .traitsOfType<exomind.base.ICollectionChild>(exomind.base.CollectionChild)
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
            this.props.onEntityAction('restore', et.entity);
        }
    }

    private handleDropInEntity = (droppedItem: IDroppedItem) => {
        const getEntityParentRelation = (entity: exocore.store.IEntity, parentId: string) => {
            return new EntityTraits(entity)
                .traitsOfType<exomind.base.CollectionChild>(exomind.base.CollectionChild)
                .filter((e) => e.message.collection.entityId == parentId)
                .shift();
        }

        const getEntityParentWeight = (entity: exocore.store.IEntity): number => {
            const child = getEntityParentRelation(entity, this.parentId)
            const weight = child.message.weight;

            if (Long.isLong(weight)) {
                return weight.toNumber();
            } else {
                return weight;
            }
        }

        const droppedEntity = droppedItem.droppedEntity;

        // calculate weight by putting it in the middle of the hovered object and the previous object so
        // that the dropped object is inserted right before the hovered object
        let weight;
        if (droppedItem.overEntity !== null) {
            const overEntityWeight = getEntityParentWeight(droppedItem.overEntity);

            if (droppedItem.previousEntity !== null) {
                const previousEntityWeight = getEntityParentWeight(droppedItem.previousEntity);
                weight = (previousEntityWeight + overEntityWeight) / 2;
            } else {
                weight = overEntityWeight + 100;
            }
        } else {
            weight = new Date().getTime();
        }

        const droppedEntityRelation = getEntityParentRelation(droppedEntity, this.parentId);
        const relationTraitId = droppedEntityRelation?.id ?? `child_${this.parentId}`;

        let mb = MutationBuilder
            .updateEntity(droppedEntity.id)
            .putTrait(new exomind.base.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: this.parentId
                }),
                weight: weight,
            }), relationTraitId)
            .returnEntities();

        // if it has been moved and it's not inside its own container, then we remove it from old parent
        if (droppedItem.effect === 'move' && droppedItem.fromParentEntity && this.parentId !== droppedItem.fromParentEntity.id) {
            const fromRelation = getEntityParentRelation(droppedEntity, droppedItem.fromParentEntity.id);
            mb = mb.deleteTrait(fromRelation.id);
        }

        Exocore.store.mutate(mb.build());

        if (this.props.onEntityAction) {
            this.props.onEntityAction('drop', droppedEntity);
        }
    }

    private handleCreatedEntity(entity: exocore.store.IEntity) {
        if (this.props.onSelectionChange && this.props.selection) {
            const et = new EntityTraits(entity);
            if (et.traitOfType(exomind.base.Task)) {
                this.setState({
                    editedEntity: et,
                });
            } else {
                const newSelection = this.props.selection.withItem(SelectedItem.fromEntity(entity));
                this.props.onSelectionChange(newSelection);
            }
        }
    }

    private removeFromSelection(entity: exocore.store.IEntity) {
        if (this.props.onSelectionChange && this.props.selection) {
            const newSelection = this.props.selection.withoutItem(SelectedItem.fromEntity(entity));
            this.props.onSelectionChange(newSelection);
        }
    }
}

import classNames from "classnames";
import { Exocore, exocore, MutationBuilder, QueryBuilder, TraitQueryBuilder, WatchedQueryWrapper } from 'exocore';
import React from 'react';
import { exomind } from "../../../protos";
import { EntityTraits } from '../../../utils/entities';
import { ExpandableQuery } from "../../../stores/queries";
import { ListEntityActions, InlineAction } from '../entity-list/actions';
import { EntityList, IDroppedItem } from "../entity-list/entity-list";
import { ColumnActions } from "./column-actions";
import { SelectedItem, Selection } from "../entity-list/selection";
import { Message } from "../message";
import { IStores, StoresContext } from "../../../stores/stores";
import { getEntityParentRelation, getEntityParentWeight } from "../../../stores/collections";
import { ContainerState } from "../container-state";
import { observer } from "mobx-react";
import { Actions, IAction } from "../../../utils/actions";
import './children.less';

interface IProps {
    parent?: EntityTraits;
    parentId?: string;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    onEntityAction?: (action: string, entity: EntityTraits) => void;

    extraActionsForEntity?: (et: EntityTraits) => IAction[];
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
                        editedEntity={this.state.editedEntity}

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

    private actionsForEntity = (et: EntityTraits): ListEntityActions => {
        let actions = Actions.forEntity(et, { parent: this.props.parent || this.props.parentId });

        if (this.props.extraActionsForEntity) {
            actions = actions.concat(this.props.extraActionsForEntity(et));
        }

        const listActions = ListEntityActions.fromActions(actions);

        // when we just created an entity that require it to be edited right away (ex: task)
        if (this.state.editedEntity && this.state.editedEntity.id == et.id) {
            listActions.inlineAction = new InlineAction(() => {
                this.setState({
                    editedEntity: null,
                });
            });
        }

        return listActions;
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

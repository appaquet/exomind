import classNames from "classnames";
import { Exocore, exocore, MutationBuilder, QueryBuilder, TraitQueryBuilder, WatchedQueryWrapper } from 'exocore';
import React from 'react';
import { exomind } from "../../../protos";
import { EntityTraits } from '../../../utils/entities';
import { ManagedQuery } from "../../../stores/queries";
import { ListEntityActions, InlineAction } from '../entity-list/actions';
import { EntityList, IDroppedItem } from "../entity-list/entity-list";
import { SelectedItem, Selection } from "../entity-list/selection";
import { Message } from "../message";
import { IStores, StoresContext } from "../../../stores/stores";
import { getEntityParentRelation, getEntityParentWeight, PINNED_WEIGHT } from "../../../stores/collections";
import { ContainerState } from "../container-state";
import { observer } from "mobx-react";
import { Actions, IAction, IActionResult } from "../../../utils/actions";
import { BottomMenu, BottomMenuItem, IActionShortcut } from "../../interaction/bottom-menu/bottom-menu";
import { IEntityCreateResult } from "../../../utils/commands";
import './children.less';

interface IProps {
    children?: React.ReactNode;

    parent?: EntityTraits;
    parentId?: string;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    emptyIcon?: string;
    emptyText?: string;

    containerState?: ContainerState;
}

interface IState {
    parent?: EntityTraits;
    entities?: EntityTraits[],
    loading?: boolean;
    hovered: boolean;
    error?: string;
    editedEntity?: EntityTraits;
}

@observer
export class Children extends React.Component<IProps, IState> {
    static contextType = StoresContext;
    declare context: IStores;

    private entityQuery: ManagedQuery;
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

        this.entityQuery = new ManagedQuery(childrenQuery, () => {
            const entities = Array.from(this.entityQuery.results()).map((res) => {
                return new EntityTraits(res.entity);
            });

            this.setState({ entities, loading: false });
        });

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
            loading: true,
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

            return (
                <div className={classes}
                    onMouseEnter={this.handleMouseEnter}
                    onMouseLeave={this.handleMouseLeave}>

                    {this.props.children}

                    {!this.state.loading &&
                        <EntityList
                            entities={this.state.entities}
                            parentEntity={this.state.parent}

                            onLoadMore={this.handleLoadMore}

                            selection={this.props.selection}
                            onSelectionChange={this.props.onSelectionChange}

                            actionsForEntity={this.actionsForEntity}
                            editedEntity={this.state.editedEntity}

                            onDropIn={this.handleDropInEntity}
                            containerState={this.props.containerState}
                        />}

                    {!this.state.loading && this.renderBottomMenu()}

                    {!this.state.loading && this.state.entities.length == 0 && this.renderEmpty()}
                </div>
            );

        } else {
            return <Message text="Loading..." showAfterMs={200} />;
        }
    }

    private renderBottomMenu(): React.ReactNode {
        if (!this.state.parent || !(this.props.containerState.active ?? false)) {
            return null;
        }

        let items: BottomMenuItem[] = [];
        let actionShortcuts: IActionShortcut[] = [];

        // Archive / snooze actions
        if (this.props.selection && !this.props.selection.isEmpty) {
            const selectedEntities = this.props.selection.filterSelectedEntities(this.state.entities);
            items = Actions.forSelectedEntities(selectedEntities, { parent: this.state.parent });
            actionShortcuts = [
                {
                    shortcutKey: 'e',
                    disabledContexts: ['input', 'modal'],
                    actionKey: 'remove-from-parent',
                },
                {
                    shortcutKey: 'z',
                    disabledContexts: ['input', 'modal'],
                    actionKey: 'snooze',
                },
                {
                    shortcutKey: 'c',
                    disabledContexts: ['input', 'modal'],
                    actionKey: 'select-entity-collections',
                },
            ];

            items.push('divider');
        }

        // Creation actions
        items = items.concat(Actions.forEntityCreation(this.state.parent));

        const handleExecuted = (action: IAction, result: IActionResult) => {
            if (action.key.startsWith('create-')) {
                this.handleEntityCreated(action, result.createResult);
            } else if (result.remove === true) {
                this.clearSelection();
            }
        };

        return (
            <BottomMenu
                items={items}
                shortcuts={actionShortcuts}
                onExecuted={handleExecuted}
            />
        );
    }

    private renderEmpty(): React.ReactNode {
        if (!this.props.emptyIcon && !this.props.emptyText) {
            return null;
        }

        const iconClass = classNames({
            'fa': true,
            [`fa-${this.props.emptyIcon}`]: true,
        });

        return <div className="empty">
            {this.props.emptyIcon && <div className="icon"><i className={iconClass} /></div>}
            {this.props.emptyText && <div className="text">{this.props.emptyText}</div>}
        </div>;
    }

    private handleLoadMore = () => {
        this.entityQuery.expand();
    };

    private handleMouseEnter = () => {
        this.setState({
            hovered: true
        });
    };

    private handleMouseLeave = () => {
        this.setState({
            hovered: false
        });
    };

    private actionsForEntity = (et: EntityTraits): ListEntityActions => {
        const actions = Actions.forEntity(et, { parent: this.props.parent || this.props.parentId });
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
    };

    private handleDropInEntity = (droppedItem: IDroppedItem) => {
        const droppedEntity = droppedItem.droppedEntity;
        const newTopWeight = new Date().getTime();

        // calculate weight by putting it in the middle of the hovered object and the previous object so
        // that the dropped object is inserted right before the hovered object
        let parentId = this.parentId;
        let weight;
        if (droppedItem.overEntity) {
            const overEntityWeight = getEntityParentWeight(droppedItem.overEntity, parentId);

            if (droppedItem.data.position == 'before') {
                if (droppedItem.previousEntity) {
                    const previousEntityWeight = getEntityParentWeight(droppedItem.previousEntity, parentId);
                    if (previousEntityWeight > PINNED_WEIGHT && overEntityWeight < PINNED_WEIGHT) {
                        weight = newTopWeight;
                    } else {
                        weight = (previousEntityWeight + overEntityWeight) / 2;
                    }
                } else if (overEntityWeight > PINNED_WEIGHT) {
                    weight = overEntityWeight + 100;
                } else {
                    weight = newTopWeight;
                }

            } else if (droppedItem.data.position == 'after') {
                if (droppedItem.nextEntity) {
                    const nextEntityWeight = getEntityParentWeight(droppedItem.nextEntity, parentId);
                    if (nextEntityWeight < PINNED_WEIGHT && overEntityWeight > PINNED_WEIGHT) {
                        weight = newTopWeight;
                    } else {
                        weight = (nextEntityWeight + overEntityWeight) / 2;
                    }

                } else {
                    if (overEntityWeight > PINNED_WEIGHT) {
                        weight = newTopWeight;
                    } else {
                        weight = overEntityWeight - 100;
                    }
                }
            } else if (droppedItem.data.position == 'into') {
                parentId = droppedItem.overEntity.id;
                weight = newTopWeight;
            }

        } else {
            weight = newTopWeight;
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
    };

    private handleEntityCreated(action: IAction, res: IEntityCreateResult) {
        if (!res.entity) {
            alert(`Failed to create entity: ${res.error}`);
            return;
        }

        const entity = res.entity;

        if (entity.traitOfType(exomind.base.v1.Task)) {
            // special case for new tasks that we edit inline
            this.setState({
                editedEntity: entity,
            });

            return; // we don't want to select newly created tasks as we edit them inline
        }

        if (this.props.onSelectionChange) {
            this.props.onSelectionChange(new Selection(SelectedItem.fromEntity(entity)));
        }
    }

    private clearSelection = () => {
        if (this.props.onSelectionChange) {
            this.props.onSelectionChange(new Selection());
        }
    };
}

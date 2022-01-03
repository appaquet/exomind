import classNames from 'classnames';
import { Exocore, exocore, MutationBuilder, QueryBuilder } from 'exocore';
import React from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../utils/entities';
import { ExpandableQuery } from '../../../stores/queries';
import { ContainerState } from '../container-state';
import { ButtonAction, EntityActions } from '../entity-list/entity-action';
import { EntityList } from '../entity-list/entity-list';
import { Selection } from '../entity-list/selection';
import { Message } from '../message';
import './recent.less';
import { runInAction } from 'mobx';

interface IProps {
    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerState?: ContainerState;
}

interface IState {
    entities?: EntityTraits[],
}

export default class Recent extends React.Component<IProps, IState> {
    private entityQuery: ExpandableQuery;

    constructor(props: IProps) {
        super(props);

        const childrenQuery = QueryBuilder
            .all()
            .orderByOperationIds(false)
            .includeDeleted()
            .count(30)
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

        if (props.containerState) {
            runInAction(() => {
                props.containerState.title = 'Recent';
                props.containerState.icon = { fa: 'history' };
            });
        }

        this.state = {};
    }

    componentWillUnmount(): void {
        this.entityQuery.free();
    }

    render(): React.ReactNode {
        if (this.entityQuery.hasResults) {
            const classes = classNames({
                'entity-component': true,
                'recent': true,
            });

            return (
                <div className={classes}>
                    <EntityList
                        entities={this.state.entities}

                        onRequireLoadMore={this.handleLoadMore}

                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                        actionsForEntity={this.actionsForEntity}

                        draggable={false}
                        droppable={false}

                        containerState={this.props.containerState}
                    />
                </div>
            );

        } else {
            return <Message text="Loading..." showAfterMs={200} />;
        }
    }

    private handleLoadMore = () => {
        this.entityQuery.expand();
    }

    private actionsForEntity = (et: EntityTraits): EntityActions => {
        return new EntityActions([
            new ButtonAction('Move to inbox', 'inbox', () => this.handleEntityMoveInbox(et)),
        ]);
    }

    private handleEntityMoveInbox(et: EntityTraits) {
        const mb = MutationBuilder
            .updateEntity(et.id)
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: 'inbox',
                }),
                weight: new Date().getTime(),
            }), 'child_inbox')
            .build();
        Exocore.store.mutate(mb);
    }
}

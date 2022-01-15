import classNames from 'classnames';
import { exocore, QueryBuilder } from 'exocore';
import React from 'react';
import { EntityTraits } from '../../../utils/entities';
import { ManagedQuery } from '../../../stores/queries';
import { ContainerState } from '../container-state';
import { ListEntityActions } from '../entity-list/actions';
import { EntityList } from '../entity-list/entity-list';
import { Selection } from '../entity-list/selection';
import { Message } from '../message';
import { runInAction } from 'mobx';
import { Actions } from '../../../utils/actions';
import './recent.less';

interface IProps {
    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerState?: ContainerState;
}

interface IState {
    entities?: EntityTraits[],
}

export default class Recent extends React.Component<IProps, IState> {
    private entityQuery: ManagedQuery;

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
        this.entityQuery = new ManagedQuery(childrenQuery, () => {
            const entities = Array.from(this.entityQuery.results()).map((res) => {
                return new EntityTraits(res.entity);
            });

            this.setState({ entities });
        });

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

                        onLoadMore={this.handleLoadMore}

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
    };

    private actionsForEntity = (et: EntityTraits): ListEntityActions => {
        const actions = Actions.forEntity(et, { section: 'recent' });
        return ListEntityActions.fromActions(actions);
    };
}

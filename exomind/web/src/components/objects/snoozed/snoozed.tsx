import classNames from 'classnames';
import { exocore, fromProtoTimestamp, QueryBuilder } from 'exocore';
import { exomind } from '../../../protos';
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import { ManagedQuery } from '../../../stores/queries';
import { ContainerState } from '../container-state';
import { ListEntityActions } from '../entity-list/actions';
import { EntityList } from '../entity-list/entity-list';
import { Selection } from '../entity-list/selection';
import { Message } from '../message';
import DateUtil from '../../../utils/dates';
import { runInAction } from 'mobx';
import './snoozed.less';
import { Actions } from '../../../utils/actions';

interface IProps {
    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerState?: ContainerState;
}

interface IState {
    entities?: EntityTraits[],
}

export default class Snoozed extends React.Component<IProps, IState> {
    private entityQuery: ManagedQuery;

    constructor(props: IProps) {
        super(props);

        const childrenQuery = QueryBuilder
            .withTrait(exomind.base.v1.Snoozed)
            .count(30)
            .project(
                new exocore.store.Projection({
                    package: ["exomind.base.v1.Snoozed"],
                }),
                new exocore.store.Projection({
                    fieldGroupIds: [1],
                    package: ["exomind.base"],
                }),
                new exocore.store.Projection({
                    skip: true,
                })
            )
            .orderByField('until_date', true)
            .build();
        this.entityQuery = new ManagedQuery(childrenQuery, () => {
            const entities = Array.from(this.entityQuery.results()).map((res) => {
                return new EntityTraits(res.entity);
            });

            this.setState({ entities });
        });

        if (props.containerState) {
            runInAction(() => {
                props.containerState.title = 'Snoozed';
                props.containerState.icon = { fa: 'clock-o' };
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
                'snoozed': true,
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

                        renderEntityDate={this.renderEntityDate}
                        containerState={this.props.containerState}
                    />
                </div>
            );

        } else {
            return <Message text="Loading..." showAfterMs={200} />;
        }
    }

    private renderEntityDate = (entity: EntityTrait<unknown>): React.ReactNode => {
        const snoozedTrait = entity.et.traitOfType<exomind.base.v1.ISnoozed>(exomind.base.v1.Snoozed);
        if (!snoozedTrait) {
            return 'Invalid';
        }

        let strDate;
        if (snoozedTrait.message.untilDate) {
            const date = fromProtoTimestamp(snoozedTrait.message.untilDate);
            strDate = DateUtil.toShortFormat(date);
        } else {
            strDate = 'unknown';
        }

        return 'Snoozed until ' + strDate;
    };

    private handleLoadMore = () => {
        this.entityQuery.expand();
    };

    private actionsForEntity = (et: EntityTraits): ListEntityActions => {
        const actions = Actions.forEntity(et, { section: 'snoozed' });
        return ListEntityActions.fromActions(actions);
    };
}

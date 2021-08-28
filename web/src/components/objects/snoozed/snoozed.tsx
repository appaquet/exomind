import classNames from 'classnames';
import { Exocore, exocore, fromProtoTimestamp, MutationBuilder, QueryBuilder } from 'exocore';
import { exomind } from '../../../protos';
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import { ExpandableQuery } from '../../../stores/queries';
import { ContainerController } from '../container-controller';
import { ButtonAction, EntityActions } from '../entity-list/entity-action';
import { EntityList } from '../entity-list/entity-list';
import { Selection } from '../entity-list/selection';
import { Message } from '../message';
import './snoozed.less';
import DateUtil from '../../../utils/dates';
import { runInAction } from 'mobx';

interface IProps {
    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerController?: ContainerController;
}

interface IState {
    entities?: EntityTraits[],
}

export default class Snoozed extends React.Component<IProps, IState> {
    private entityQuery: ExpandableQuery;

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
        this.entityQuery = new ExpandableQuery(childrenQuery, () => {
            const entities = Array.from(this.entityQuery.results()).map((res) => {
                return new EntityTraits(res.entity);
            });

            this.setState({ entities });
        })

        if (props.containerController) {
            runInAction(() => {
                props.containerController.title = 'Snoozed';
                props.containerController.icon = { fa: 'clock-o' };
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

                        onRequireLoadMore={this.handleLoadMore.bind(this)}

                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                        actionsForEntity={this.actionsForEntity.bind(this)}

                        draggable={false}
                        droppable={false}

                        renderEntityDate={this.renderEntityDate.bind(this)}
                    />
                </div>
            );

        } else {
            return <Message text="Loading..." showAfterMs={200} />;
        }
    }

    private renderEntityDate(entity: EntityTrait<unknown>): React.ReactFragment {
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
    }

    private handleLoadMore() {
        this.entityQuery.expand();
    }

    private actionsForEntity(et: EntityTraits): EntityActions {
        return new EntityActions([
            new ButtonAction('inbox', this.handleEntityMoveInbox.bind(this, et)),
        ]);
    }

    private handleEntityMoveInbox(et: EntityTraits) {
        const snoozedTrait = et.traitOfType<exomind.base.v1.ISnoozed>(exomind.base.v1.Snoozed);
        if (!snoozedTrait) {
            return;
        }

        const mb = MutationBuilder
            .updateEntity(et.id)
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: 'inbox',
                }),
                weight: new Date().getTime(),
            }), 'child_inbox')
            .deleteTrait(snoozedTrait.id)
            .build();
        Exocore.store.mutate(mb);
    }
}

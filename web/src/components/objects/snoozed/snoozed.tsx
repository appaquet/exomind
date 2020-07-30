import classNames from 'classnames';
import { Exocore, exocore, MutationBuilder, QueryBuilder } from 'exocore';
import { exomind } from '../../../protos';
import React from 'react';
import { EntityTraits } from '../../../store/entities';
import { ExpandableQuery } from '../../../store/queries';
import { ContainerController } from '../container-controller';
import EntityAction from '../entity-list/entity-action';
import { EntityList } from '../entity-list/entity-list';
import { Selection } from '../entity-list/selection';
import { Message } from '../message';
import './snoozed.less';

interface IProps {
    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    onEntityAction?: (action: string, entity: exocore.index.IEntity) => void;

    containerController?: ContainerController;
}

export default class Snoozed extends React.Component<IProps> {
    private entityQuery: ExpandableQuery;

    constructor(props: IProps) {
        super(props);

        const childrenQuery = QueryBuilder
            .withTrait(exomind.base.Postponed)
            .count(30)
            .project(
                new exocore.index.Projection({
                    fieldGroupIds: [1],
                    package: ["exomind.base"],
                }),
                new exocore.index.Projection({
                    skip: true,
                })
            )
            .build();
        this.entityQuery = new ExpandableQuery(childrenQuery, () => {
            this.setState({});
        })

        if (props.containerController) {
            props.containerController.title = 'Snoozed';
            props.containerController.icon = 'clock-o';
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

            const entities = Array.from(this.entityQuery.results()).map((res) => {
                return res.entity;
            });

            return (
                <div className={classes}>
                    <EntityList
                        entities={entities}

                        onRequireLoadMore={this.handleLoadMore.bind(this)}

                        selection={this.props.selection}
                        onSelectionChange={this.props.onSelectionChange}
                        actionsForEntity={this.actionsForEntity.bind(this)}

                        draggable={false}
                        droppable={false}
                    />
                </div>
            );

        } else {
            return <Message text="Loading..." showAfterMs={200} />;
        }
    }

    private handleLoadMore() {
        this.entityQuery.expand();
    }

    private actionsForEntity(et: EntityTraits): EntityAction[] {
        return [
            new EntityAction('inbox', this.handleEntityMoveInbox.bind(this, et)),
        ];
    }

    private handleEntityMoveInbox(et: EntityTraits) {
        const snoozedTrait = et.traitOfType<exomind.base.IPostponed>(exomind.base.Postponed);
        if (!snoozedTrait) {
            return;
        }

        const mb = MutationBuilder
            .updateEntity(et.id)
            .putTrait(new exomind.base.CollectionChild({
                collection: new exocore.index.Reference({
                    entityId: 'inbox',
                }),
                weight: new Date().getTime(),
            }), 'child_inbox')
            .deleteTrait(snoozedTrait.id)
            .build();
        Exocore.store.mutate(mb);

        if (this.props.onEntityAction) {
            this.props.onEntityAction('inbox', et.entity);
        }
    }
}

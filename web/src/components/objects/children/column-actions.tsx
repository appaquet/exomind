import { Exocore, exocore, MutationBuilder, QueryBuilder, toProtoTimestamp } from 'exocore';
import React from 'react';
import { exomind } from '../../../protos';
import { EntityTraits } from '../../../utils/entities';
import InputModal from '../../modals/input-modal/input-modal';
import TimeSelector from '../../modals/time-selector/time-selector';
import { Selection } from "../entity-list/selection";
import { IStores, StoresContext } from '../../../stores/stores';
import { getEntityParentRelation } from '../../../stores/collections';
import { ListenerToken, Shortcuts } from '../../../shortcuts';
import "./column-actions.less";
import { Commands } from '../../../utils/commands';

interface IProps {
    parent: EntityTraits;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    onCreated: (et: EntityTraits) => void;

    removeOnPostpone?: boolean;
}

export class ColumnActions extends React.Component<IProps> {
    private shortcutToken: ListenerToken;

    static contextType = StoresContext;
    declare context: IStores;

    constructor(props: IProps) {
        super(props);

        this.shortcutToken = Shortcuts.register([
            {
                key: 'e',
                callback: () => {
                    if (this.props.selection.length == 0) {
                        return false;
                    }
                    this.handleDoneClick();
                    return true;
                },
                disabledContexts: ['input', 'modal'],
            },
            {
                key: 'z',
                callback: () => {
                    if (this.props.selection.length == 0) {
                        return false;
                    }
                    this.handleSnoozeClick();
                    return true;
                },
                disabledContexts: ['input', 'modal'],
            }
        ]);
    }

    componentWillUnmount() {
        Shortcuts.unregister(this.shortcutToken);
    }

    render(): React.ReactNode {
        if (this.props.selection && this.props.selection.length <= 1) {
            return this.renderCreationActions();
        } else {
            return this.renderSelectionActions();
        }
    }

    private renderCreationActions(): React.ReactNode {
        return <div className="column-bottom-actions">
            <ul>
                <li onClick={this.handleNewNoteClick}><i className="new-note" /></li>
                <li onClick={this.handleNewCollectionClick}><i className="new-collection" /></li>
                <li onClick={this.handleNewTaskClick}><i className="new-task" /></li>
                <li onClick={this.handleNewLinkClick}><i className="new-link" /></li>
            </ul>
        </div>
    }

    private renderSelectionActions(): React.ReactNode {
        return <div className="column-bottom-actions">
            <ul>
                <li onClick={this.handleDoneClick}><i className="done" /></li>
                <li onClick={this.handleSnoozeClick}><i className="postpone" /></li>
            </ul>
        </div>
    }

    private handleNewNoteClick = () => {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Note({
                title: 'New note',
            }))
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewEmailClick = () => {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.DraftEmail())
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewCollectionClick = () => {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Collection({
                name: 'New collection',
            }))
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewTaskClick = () => {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.v1.Task())
            .putTrait(new exomind.base.v1.CollectionChild({
                collection: new exocore.store.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewLinkClick = () => {
        const createLink = (url?: string) => {
            this.context.session.hideModal();

            if (!url) {
                return;
            }

            const mutation = MutationBuilder
                .createEntity()
                .putTrait(new exomind.base.v1.Link({
                    url: url,
                }))
                .putTrait(new exomind.base.v1.CollectionChild({
                    collection: new exocore.store.Reference({
                        entityId: this.props.parent.id,
                    }),
                    weight: new Date().getTime(),
                }), `child_${this.props.parent.id}`)
                .returnEntities()
                .build();

            this.executeNewEntityMutation(mutation);
        }

        this.context.session.showModal(() => {
            return <InputModal
                text="URL of the link"
                onDone={createLink} />;
        });
    }

    private handleDoneClick = async () => {
        const entities = await this.getSelectedEntities();
        await Commands.removeFromParent(entities, this.props.parent.id);

        if (this.props.selection) {
            this.props.onSelectionChange(this.props.selection.cleared());
        }
    }

    private handleSnoozeClick = () => {
        this.context.session.showModal(() => {
            return <TimeSelector onSelectionDone={this.handleTimeSelectorDone} />;
        });
    }

    private handleTimeSelectorDone = async (date: Date) => {
        this.context.session.hideModal();

        const entities = await this.getSelectedEntities();

        for (const entity of entities) {
            const mb = MutationBuilder
                .updateEntity(entity.id)
                .putTrait(new exomind.base.v1.Snoozed({
                    untilDate: toProtoTimestamp(date),
                }), "snoozed");

            const parentRelation = getEntityParentRelation(entity, this.props.parent.id);
            if (parentRelation) {
                mb.deleteTrait(parentRelation.id);
            }

            await Exocore.store.mutate(mb.build());
        }

        if (this.props.selection && this.props.removeOnPostpone) {
            this.props.onSelectionChange(this.props.selection.cleared());
        }
    }

    private async getSelectedEntities() {
        const ids = (this.props.selection?.items ?? [])
            .map((sel) => sel.entityId)
            .filter((et) => !!et);

        const results = await Exocore.store.query(QueryBuilder.withIds(ids).build());
        const entities = results.entities.map((res) => new EntityTraits(res.entity));

        return Array.from(entities);
    }

    private executeNewEntityMutation(mutation: exocore.store.MutationRequest) {
        Exocore.store.mutate(mutation).then(result => {
            if (result.entities.length > 0 && this.props.onCreated) {
                this.props.onCreated(new EntityTraits(result.entities[0]));
            }
        });
    }
}


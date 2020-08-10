import { Exocore, exocore, MutationBuilder } from 'exocore';
import * as _ from 'lodash';
import React from 'react';
import { exomind } from '../../../protos';
import { ModalStore } from '../../../store/modal-store';
import TimeSelector from '../../popups/time-selector/time-selector';
import { Selection } from "./selection";


interface IProps {
    parent: exocore.index.IEntity;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    onCreated: (et: exocore.index.IEntity) => void;

    removeOnPostpone?: boolean;
}

export class ListActions extends React.Component<IProps> {
    render(): React.ReactNode {
        if (_.size(this.props.selection) <= 1) {
            return this.renderCreationActions();
        } else {
            return this.renderSelectionActions();
        }
    }

    private renderCreationActions(): React.ReactNode {
        return <div className="object-actions">
            <ul>
                <li onClick={this.handleNewNoteClick.bind(this)}><i className="new-note" /></li>
                <li onClick={this.handleNewEmailClick.bind(this)}><i className="new-email" /></li>
                <li onClick={this.handleNewCollectionClick.bind(this)}><i className="new-collection" /></li>
                <li onClick={this.handleNewTaskClick.bind(this)}><i className="new-task" /></li>
                <li onClick={this.handleNewLinkClick.bind(this)}><i className="new-link" /></li>
            </ul>
        </div>
    }

    private renderSelectionActions(): React.ReactNode {
        return <div className="object-actions">
            <ul>
                <li onClick={this.handleDoneClick.bind(this)}><i className="done" /></li>
                <li onClick={this.handlePostponeClick.bind(this)}><i className="postpone" /></li>
            </ul>
        </div>
    }

    private handleNewNoteClick() {

        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.Note({
                title: 'New note',
            }))
            .putTrait(new exomind.base.CollectionChild({
                collection: new exocore.index.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewEmailClick() {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.DraftEmail())
            .putTrait(new exomind.base.CollectionChild({
                collection: new exocore.index.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewCollectionClick() {
        const mutation = MutationBuilder
            .createEntity()
            .putTrait(new exomind.base.Collection({
                name: 'New collection',
            }))
            .putTrait(new exomind.base.CollectionChild({
                collection: new exocore.index.Reference({
                    entityId: this.props.parent.id,
                }),
                weight: new Date().getTime(),
            }), `child_${this.props.parent.id}`)
            .returnEntities()
            .build();

        this.executeNewEntityMutation(mutation);
    }

    private handleNewTaskClick() {
        const title = prompt('Name of the task');
        if (!_.isEmpty(title)) {
            const mutation = MutationBuilder
                .createEntity()
                .putTrait(new exomind.base.Task({
                    title: title,
                }))
                .putTrait(new exomind.base.CollectionChild({
                    collection: new exocore.index.Reference({
                        entityId: this.props.parent.id,
                    }),
                    weight: new Date().getTime(),
                }), `child_${this.props.parent.id}`)
                .returnEntities()
                .build();

            this.executeNewEntityMutation(mutation);
        }
    }

    private handleNewLinkClick() {
        const url = prompt('URL');
        if (!_.isEmpty(url)) {
            const mutation = MutationBuilder
                .createEntity()
                .putTrait(new exomind.base.Link({
                    url: url,
                }))
                .putTrait(new exomind.base.CollectionChild({
                    collection: new exocore.index.Reference({
                        entityId: this.props.parent.id,
                    }),
                    weight: new Date().getTime(),
                }), `child_${this.props.parent.id}`)
                .returnEntities()
                .build();

            this.executeNewEntityMutation(mutation);
        }
    }

    private handleDoneClick() {
        // TODO: Done all selected
        // _.forEach(this.props.selection, (entity) => {
        //   ExomindDSL.on(entity).relations.removeParent(this.props.parent);
        // });

        this.props.onSelectionChange(this.props.selection.cleared());
    }

    private handlePostponeClick() {
        ModalStore.showModal(this.showTimeSelector.bind(this));
    }

    private showTimeSelector() {
        return <TimeSelector onSelectionDone={this.handleTimeSelectorDone.bind(this)} />;
    }

    private handleTimeSelectorDone(/*date: Date*/) {
        ModalStore.hideModal();

        // TODO: Postpone all selected
        // _.forEach(this.props.selection, (entity) => {
        //   ExomindDSL.on(entity).relations.postpone(date);
        //   if (this.props.removeOnPostpone) {
        //     ExomindDSL.on(entity).relations.removeParent(this.props.parent);
        //   }
        // });

        if (this.props.removeOnPostpone) {
            this.props.onSelectionChange(this.props.selection.cleared());
        }
    }

    private executeNewEntityMutation(mutation: exocore.index.MutationRequest) {
        Exocore.store.mutate(mutation).then(result => {
            if (result.entities.length > 0 && this.props.onCreated) {
                this.props.onCreated(result.entities[0]);
            }
        });
    }
}


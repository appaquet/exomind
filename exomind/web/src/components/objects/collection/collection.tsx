import { exomind } from "../../../protos";
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import { Children } from '../children/children';
import { Selection } from '../entity-list/selection';
import EditableText from "../../interaction/editable-text/editable-text";
import { ContainerState } from "../container-state";
import { observer } from "mobx-react";
import { Exocore, MutationBuilder } from "exocore";

interface IProps {
    entity: EntityTraits;
    collection: EntityTrait<exomind.base.v1.ICollection>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    sections?: string[];
    section?: string;
    actionsForSection?: (section: string) => string[];

    containerState?: ContainerState;
}

@observer
export default class Collection extends React.Component<IProps> {
    constructor(props: IProps) {
        super(props);

        this.props.containerState?.addDetailsHeaderAction();
    }

    render(): React.ReactNode {
        return (
            <Children
                parent={this.props.entity}

                emptyIcon="folder-o"
                emptyText="This collection is empty"

                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}

                containerState={this.props.containerState}
            >

                {this.props.containerState?.showDetails &&
                    <div className="entity-details">
                        <div className="title field">
                            <span className="field-label">Name</span>
                            <span className="field-content">
                                <EditableText text={this.props.collection.message.name} onChange={this.handleChangeName} />
                            </span>
                        </div>

                        <div className="description field">
                            <span className="field-label">Description</span>
                            <span className="field-content">
                                <EditableText text={this.props.collection.message.description} onChange={this.handleChangeDescription} />
                            </span>
                        </div>
                    </div>
                }

            </Children>
        );
    }

    private handleChangeName = (text: string) => {
        const newCollection = new exomind.base.v1.Collection(this.props.collection.message);
        newCollection.name = text;

        const mutation = MutationBuilder
            .updateEntity(this.props.entity.entity.id)
            .putTrait(newCollection, this.props.collection.id)
            .build();

        Exocore.store.mutate(mutation);
    };

    private handleChangeDescription = (text: string) => {
        const newCollection = new exomind.base.v1.Collection(this.props.collection.message);
        newCollection.description = text;

        const mutation = MutationBuilder
            .updateEntity(this.props.entity.entity.id)
            .putTrait(newCollection, this.props.collection.id)
            .build();

        Exocore.store.mutate(mutation);
    };
}

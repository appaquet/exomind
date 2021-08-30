import { exomind } from "../../../protos";
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import { Children } from '../children/children';
import { Selection } from '../entity-list/selection';
import EditableText from "../../interaction/editable-text/editable-text";
import { ContainerController } from "../container-controller";
import { observer } from "mobx-react";
import { Exocore, MutationBuilder } from "exocore";

interface IProps {
    entity: EntityTraits;
    collection: EntityTrait<exomind.base.v1.ICollection>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    onEntityAction?: (action: string, entity: EntityTraits) => void;

    sections?: string[];
    section?: string;
    actionsForSection?: (section: string) => string[];

    containerController?: ContainerController;
}

@observer
export default class Collection extends React.Component<IProps> {
    constructor(props: IProps) {
        super(props);
    }

    render(): React.ReactNode {
        return (
            <Children
                parent={this.props.entity}

                actionsForEntity={this.actionsForChildrenType}

                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}
            >

                {this.props.containerController?.details &&
                    <div className="object-summary">
                        <div className="title field"><span className="field-label">Name</span>
                            <span className="field-content">
                                <EditableText text={this.props.collection.message.name} onChange={this.handleChangeName} />
                            </span>
                        </div>

                        <div className="title field"><span className="field-label">Description</span>
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
    }

    private handleChangeDescription = (text: string) => {
        const newCollection = new exomind.base.v1.Collection(this.props.collection.message);
        newCollection.description = text;

        const mutation = MutationBuilder
            .updateEntity(this.props.entity.entity.id)
            .putTrait(newCollection, this.props.collection.id)
            .build();

        Exocore.store.mutate(mutation);
    }

    private actionsForChildrenType = (): string[] => {
        return ['done', 'postpone', 'move', 'pin'];
    }
}

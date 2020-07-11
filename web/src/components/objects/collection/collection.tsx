import { exocore } from 'exocore';
import { exomind } from "../../../protos";
import React from 'react';
import { EntityTrait } from '../../../store/entities';
import { Children } from '../children/children';
import { Selection } from '../entity-list/selection';

interface IProps {
    entity: exocore.index.IEntity;
    collection: EntityTrait<exomind.base.ICollection>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    onEntityAction?: (action: string, entity: exocore.index.IEntity) => void;

    sections?: string[];
    section?: string;
    actionsForSection?: (section: string) => string[];
}

export default class Collection extends React.Component<IProps> {
    constructor(props: IProps) {
        super(props);
    }

    render(): React.ReactNode {
        return (
            <Children
                parent={this.props.entity}

                sections={['current', 'old']}
                section={'current'}
                actionsForSection={this.actionsForChildrenType.bind(this)}

                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}
            />
        );
    }

    private actionsForChildrenType(section: string): string[] {
        switch (section) {
            case 'current':
                return ['done', 'postpone', 'move'];
            case 'old':
                return ['restore', 'postpone', 'move'];
        }
    }

    // TODO:
    // queryForChildrenType(childrenType) {
    //     switch (childrenType) {
    //         case 'current':
    //             return Q.Entities.withTrait(Exomind.Child, b => b.refersTo(this.props.entity.id)).withSummary();
    //         case 'old':
    //             return Q.Entities.withTrait(Exomind.OldChild, b => b.refersTo(this.props.entity.id)).withSummary();
    //     }
    // }
    //
    // TODO:
    // handleNameChange(name) {
    //     let newCollection = this.state.currentCollection.clone();
    //     newCollection.name = name;
    //     ExomindDSL.on(this.props.entity).mutate.update(newCollection).execute();
    //
    //     if (this.props.containerController) {
    //         this.props.containerController.title = new ModifiableText(name, this.handleNameChange.bind(this));
    //     }
    // }

}

import { exomind } from "../../../protos";
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import { Children } from '../children/children';
import { Selection } from '../entity-list/selection';

interface IProps {
    entity: EntityTraits;
    collection: EntityTrait<exomind.base.v1.ICollection>;

    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    onEntityAction?: (action: string, entity: EntityTraits) => void;

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

                actionsForEntity={this.actionsForChildrenType}

                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}
            />
        );
    }

    private actionsForChildrenType = (): string[] => {
        return ['done', 'postpone', 'move', 'pin'];
    }
}

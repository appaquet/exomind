import * as React from 'react';
import { Children } from "../children/children";
import { ContainerController } from "../container-controller";
import { Selection } from "../entity-list/selection";

interface IProps {
    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerController?: ContainerController;
}

export class Inbox extends React.Component<IProps> {
    constructor(props: IProps) {
        super(props);

        if (props.containerController) {
            props.containerController.title = 'Inbox';
            props.containerController.icon = 'inbox';
        }
    }

    render(): React.ReactNode {
        return (
            <Children
                parentId="inbox"

                sections={['old', 'current', 'future']}
                section={'current'}
                actionsForSection={this.actionsForChildrenType.bind(this)}

                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}

                removeOnPostpone={true}
            />
        );
    }

    private actionsForChildrenType(section: string): string[] {
        switch (section) {
            case 'current':
                return ['done', 'postpone', 'move'];
            case 'old':
                return ['restore', 'postpone', 'move'];
            case 'future':
                return ['inbox', 'done'];

            default:
                return [];
        }
    }
}
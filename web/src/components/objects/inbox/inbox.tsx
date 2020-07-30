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

                actionsForSection={this.actionsForChildrenType.bind(this)}

                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}

                removeOnPostpone={true}
            />
        );
    }

    private actionsForChildrenType(): string[] {
        return ['done', 'postpone', 'move'];
    }
}
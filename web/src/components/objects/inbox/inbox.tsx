import { runInAction } from 'mobx';
import * as React from 'react';
import { Children } from "../children/children";
import { ContainerState } from "../container-state";
import { Selection } from "../entity-list/selection";

interface IProps {
    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;

    containerState?: ContainerState;
}

export class Inbox extends React.Component<IProps> {
    constructor(props: IProps) {
        super(props);

        if (props.containerState) {
            runInAction(() => {
                props.containerState.title = 'Inbox';
                props.containerState.icon = { fa: 'inbox' };
            });
        }
    }

    render(): React.ReactNode {
        return (
            <Children
                parentId="inbox"

                emptyIcon="sun-o"
                emptyText="All done! It's time for a break!"

                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}

                containerState={this.props.containerState}
            />
        );
    }
}
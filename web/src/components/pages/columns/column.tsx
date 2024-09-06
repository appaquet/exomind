import classNames from 'classnames';
import React from 'react';
import { ContainerState, ModifiableText } from '../../objects/container-state';
import { EntityComponent } from '../../objects/entity-component';
import { Selection } from '../../objects/entity-list/selection';
import { Header, HeaderAction } from '../../objects/header';
import { Inbox } from "../../objects/inbox/inbox";
import Snoozed from "../../objects/snoozed/snoozed";
import Recent from "../../objects/recent/recent";
import { Search } from '../../objects/search/search';
import { ColumnConfig } from './columns-config';
import { Message } from '../../objects/message';
import { observer } from 'mobx-react';
import { observable, runInAction } from 'mobx';
import './column.less';

interface IProps {
    columnConfig: ColumnConfig;
    columnId: number;
    active: boolean;
    onClose: () => void;

    selection?: Selection;
    onSelectionChange: (sel: Selection) => void;
}

interface IState {
    value: string;
}

@observer
export default class Column extends React.Component<IProps, IState> {
    @observable private containerState: ContainerState = new ContainerState();

    constructor(props: IProps) {
        super(props);

        runInAction(() => {
            this.containerState.active = props.active;
        });

        this.state = {
            value: props.columnConfig.first,
        };
    }

    componentDidUpdate(): void {
        runInAction(() => {
            this.containerState.active = this.props.active;
        });
    }

    render(): React.ReactNode {
        if (this.containerState.closed) {
            runInAction(() => {
                this.props.onClose();
            });
        }

        const colKey = `column-${this.props.columnId}`;
        const classes = classNames({
            column: true,
            [colKey]: true,
            active: this.props.active,
        });

        let title, editableTitle, titleRenameHandler;
        if (this.containerState.title instanceof ModifiableText) {
            title = this.containerState.title.value || '';
            titleRenameHandler = this.containerState.title.onChange;
            editableTitle = this.containerState.title.editValue || title;
        } else {
            title = this.containerState.title || '';
            titleRenameHandler = null;
        }

        const headerActions = [];
        if (this.containerState.actions) {
            this.containerState.actions.forEach(action => {
                headerActions.push(action);
            });
        }

        if (this.props.onClose) {
            headerActions.push(new HeaderAction('Close', 'close', this.props.onClose, false, 100));
        }

        return (
            <div className={classes}>
                <Header
                    title={title}
                    editableTitle={editableTitle}
                    onTitleRename={titleRenameHandler}
                    icon={this.containerState.icon}
                    actions={headerActions}
                    active={this.containerState.active}
                />

                <div className="content">
                    {this.renderContent()}
                </div>
            </div>
        );
    }

    private renderContent() {
        if (this.props.columnConfig.isInbox) {
            return <Inbox
                containerState={this.containerState}
                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}
            />;

        } else if (this.props.columnConfig.isSnoozed) {
            return <Snoozed
                containerState={this.containerState}
                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}
            />;

        } else if (this.props.columnConfig.isRecent) {
            return <Recent
                containerState={this.containerState}
                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}
            />;

        } else if (this.props.columnConfig.isSearch) {
            return <Search
                query={this.props.columnConfig.first}
                containerState={this.containerState}
                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}
            />;

        } else if (this.props.columnConfig.isEntity) {
            return <EntityComponent
                entityId={this.props.columnConfig.first}
                containerState={this.containerState}
                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}
            />;

        } else if (this.props.columnConfig.isTrait) {
            return <EntityComponent
                entityId={this.props.columnConfig.first}
                traitId={this.props.columnConfig.second}
                containerState={this.containerState}
                selection={this.props.selection}
                onSelectionChange={this.props.onSelectionChange}
            />;
        } else {
            return <Message text="Unknown column type" />;
        }
    }
}
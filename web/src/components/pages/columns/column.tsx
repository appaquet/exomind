import classNames from 'classnames';
import React from 'react';
import Navigation from '../../../navigation';
import { ContainerController, ModifiableText } from '../../objects/container-controller';
import { EntityComponent } from '../../objects/entity-component';
import { Selection } from '../../objects/entity-list/selection';
import { Header, HeaderAction } from '../../objects/header';
import { Inbox } from "../../objects/inbox/inbox";
import Snoozed from "../../objects/snoozed/snoozed";
import Recent from "../../objects/recent/recent";
import { Search } from '../../objects/search/search';
import './column.less';
import { ColumnConfig } from './columns-config';
import { Message } from '../../objects/message';
import copy from 'clipboard-copy';
import { observer } from 'mobx-react';
import { observable, runInAction } from 'mobx';

interface IProps {
  columnConfig: ColumnConfig;
  columnId: number;

  selection?: Selection;

  onSelectionChange: (sel: Selection) => void;
  onClose: () => void;
}

interface IState {
  value: string;
}

@observer
export default class Column extends React.Component<IProps, IState> {
  @observable private containerController: ContainerController = new ContainerController();

  constructor(props: IProps) {
    super(props);

    this.state = {
      value: props.columnConfig.first,
    }
  }

  render(): React.ReactNode {
    if (this.containerController.closed) {
      runInAction(() => {
        this.props.onClose();
      });
    }

    const colKey = `column-${this.props.columnId}`;
    const classes = classNames({
      column: true,
      [colKey]: true
    });

    let title, editableTitle, titleRenameHandler;
    if (this.containerController.title instanceof ModifiableText) {
      title = this.containerController.title.value || '';
      titleRenameHandler = this.containerController.title.onChange;
      editableTitle = this.containerController.title.editValue || title;
    } else {
      title = this.containerController.title || '';
      titleRenameHandler = null;
    }

    const headerActions = [];

    if (this.containerController.actions) {
      this.containerController.actions.forEach(action => {
        headerActions.push(action);
      });
    }

    if (this.props.columnConfig.isEntity) {
      headerActions.push(new HeaderAction('external-link', this.expandFullscreen));

      headerActions.push(new HeaderAction('copy', () => {
        copy(this.props.columnConfig.first);
      }));
    }

    if (this.props.onClose) {
      headerActions.push(new HeaderAction('close', this.props.onClose));
    }

    return (
      <div className={classes}>
        <Header
          title={title}
          editableTitle={editableTitle}
          onTitleRename={titleRenameHandler}
          icon={this.containerController.icon}
          actions={headerActions}
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
        containerController={this.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isSnoozed) {
      return <Snoozed
        containerController={this.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isRecent) {
      return <Recent
        containerController={this.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isSearch) {
      return <Search
        query={this.props.columnConfig.first}
        containerController={this.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isEntity) {
      return <EntityComponent
        entityId={this.props.columnConfig.first}
        containerController={this.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isTrait) {
      return <EntityComponent
        entityId={this.props.columnConfig.first}
        traitId={this.props.columnConfig.second}
        containerController={this.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;
    } else {
      return <Message text="Unknown column type" />;
    }
  }

  private expandFullscreen = () => {
    const entityId = this.props.columnConfig.first;
    Navigation.navigatePopup(Navigation.pathForFullscreen(entityId));
  }
}
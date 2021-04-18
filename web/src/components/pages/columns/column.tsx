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

interface IProps {
    columnConfig: ColumnConfig;
    columnId: number;

    selection?: Selection;

    onSelectionChange: (sel: Selection) => void;
    onClose: () => void;
}

interface IState {
    value: string;
    containerController: ContainerController;
}

export default class Column extends React.Component<IProps, IState> {
  constructor(props: IProps) {
    super(props);

      const containerController = new ContainerController();
      containerController.onChange((key: string) => {
        if (key === 'closed') {
          props.onClose();
        }
        this.forceUpdate();
      });

      this.state = {
        value: props.columnConfig.value,
        containerController: containerController
      }
  }

  render(): React.ReactNode {
    const colKey = `column-${this.props.columnId}`;
    const classes = classNames({
      column: true,
      [colKey]: true
    });

    let title, editableTitle, titleRenameHandler;
    if (this.state.containerController.title instanceof ModifiableText) {
      title = this.state.containerController.title.value || '';
      titleRenameHandler = this.state.containerController.title.onChange;
      editableTitle = this.state.containerController.title.editValue || title;
    } else {
      title = this.state.containerController.title || '';
      titleRenameHandler = null;
    }

    const headerActions = [];
    if (this.state.containerController.actions) {
      this.state.containerController.actions.forEach(action => {
        headerActions.push(action);
      });
    }

    if (this.props.columnConfig.isEntity) {
      headerActions.push(new HeaderAction('external-link', this.expandFullscreen));

      headerActions.push(new HeaderAction('copy', () => {
        copy(this.props.columnConfig.value);
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
          icon={this.state.containerController.icon}
          actions={headerActions}
        />

        {this.renderContent()}
      </div>
    );
  }

  private renderContent() {
    if (this.props.columnConfig.isInbox) {
      return <Inbox
        containerController={this.state.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isSnoozed) {
      return <Snoozed
        containerController={this.state.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isHistory) {
      return <Recent
        containerController={this.state.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isSearch) {
      return <Search
        query={this.props.columnConfig.value}
        containerController={this.state.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isEntity) {
      return <EntityComponent
        entityId={this.props.columnConfig.value}
        containerController={this.state.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;

    } else if (this.props.columnConfig.isTrait) {
      return <EntityComponent
        entityId={this.props.columnConfig.value}
        traitId={this.props.columnConfig.extra}
        containerController={this.state.containerController}
        selection={this.props.selection}
        onSelectionChange={this.props.onSelectionChange}
      />;
    } else {
      return <Message text="Unknown column type" />;
    }
  }

  private expandFullscreen = () => {
    const entityId = this.props.columnConfig.value;
    Navigation.navigatePopup(Navigation.pathForFullscreen(entityId));
  }
}
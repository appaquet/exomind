import classNames from 'classnames';
import PropTypes from 'prop-types';
import React from 'react';
import Constants from '../../../constants';
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

export default class Column extends React.Component {
  static propTypes = {
    columnConfig: PropTypes.instanceOf(ColumnConfig).isRequired,
    columnId: PropTypes.number.isRequired,

    selection: PropTypes.instanceOf(Selection),
    onSelectionChange: PropTypes.func,
    onClose: PropTypes.func
  };

  constructor(props) {
    super(props);

      let containerController = new ContainerController();
      containerController.onChange((key) => {
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

  render() {
    let colKey = `column-${this.props.columnId}`;
    let classes = classNames({
      column: true,
      [colKey]: true
    });

    let title, titleRenameHandler;
    if (this.state.containerController.title instanceof ModifiableText) {
      title = this.state.containerController.title.value || '';
      titleRenameHandler = this.state.containerController.title.onChange;
    } else {
      title = this.state.containerController.title || '';
      titleRenameHandler = null;
    }

    let headerActions = [];
    if (this.state.containerController.actions) {
      this.state.containerController.actions.forEach(action => {
        headerActions.push(action);
      });
    }

    if (this.props.columnConfig.isEntity) {
      headerActions.push(new HeaderAction('external-link', this.expandFullscreen.bind(this)));
    }

    if (this.props.columnId !== 0) {
      headerActions.push(new HeaderAction('close', this.props.onClose));
    }

    return (
      <div className={classes}>
        <Header
          title={title}
          onTitleRename={titleRenameHandler}
          icon={this.state.containerController.icon}
          actions={headerActions}
        />

        {this.renderContent()}
      </div>
    );
  }

  renderContent() {
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

  expandFullscreen() {
    let entityId = this.props.columnConfig.value;
    Navigation.navigatePopup(Navigation.pathForFullscreen(entityId));
  }
}
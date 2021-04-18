
import React from 'react';
import EditableText from '../interaction/editable-text/editable-text';
import classNames from 'classnames';
import './header.less';
import { TraitIcon } from '../../store/entities.js';
import EntityIcon from './entity-icon';

interface IProps {
  title: string;
  editableTitle?: string; // if title to be edited is different from displayed title (ex: emoji prefix)
  icon?: TraitIcon;
  actions: HeaderAction[];
  onTitleRename: (title: string) => void;
}

export class Header extends React.Component<IProps> {
  render(): React.ReactNode {
    let rightActions;
    if (this.props.actions) {
      rightActions = <div className="right-actions">{this.renderActions()}</div>;
    }

    return (
      <div className="header">
        <div className="icon">
          {this.props.icon && <EntityIcon icon={this.props.icon} />}
        </div>
        <div className="title">{this.renderTitle()}</div>
        {rightActions}
      </div>
    );
  }

  private renderTitle() {
    if (this.props.onTitleRename) {
      return <EditableText
        text={this.props.title}
        editText={this.props.editableTitle}
        doubleClick={true}
        onChange={this.handleTitleRename.bind(this)} />;
    } else {
      return this.props.title;
    }
  }

  private handleTitleRename(newTitle: string) {
    this.props.onTitleRename(newTitle);
  }

  private renderActions() {
    const actions = this.props.actions.map(action => {
      const classes = classNames({
        'fa': true,
        ['fa-' + action.icon]: true
      });
      return <li key={action.icon} onClick={this.handleActionClick.bind(this, action)}><i className={classes} /></li>;
    });

    return <ul className="actions">{actions}</ul>;
  }

  private handleActionClick(action: HeaderAction) {
    if (action.callback) {
      action.callback();
    }
  }
}

export class HeaderAction {
  constructor(public icon: string, public callback: () => void) {
  }
}

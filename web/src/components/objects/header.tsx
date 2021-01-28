
import React from 'react';
import EditableText from '../interaction/editable-text/editable-text.js';
import classNames from 'classnames';
import './header.less';

interface IProps {
  title: string;
  icon?: string;
  actions: HeaderAction[];
  onTitleRename: (title: string) => void;
}

export class Header extends React.Component<IProps> {
  render(): React.ReactNode  {
    let iconClasses;
    if (this.props.icon) {
      iconClasses = classNames({
        'fa': true,
        ['fa-' + this.props.icon]: true
      });
    }


    let rightActions;
    if (this.props.actions) {
      rightActions = <div className="right-actions">{this.renderActions()}</div>;
    }

    return (
      <div className="header">
        <div className="icon"><span className={iconClasses} /></div>
        <div className="title">{this.renderTitle()}</div>
        {rightActions}
      </div>
    );
  }

  private renderTitle() {
    if (this.props.onTitleRename) {
      return <EditableText text={this.props.title} doubleClick={true} onChange={this.handleTitleRename.bind(this)} />;
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

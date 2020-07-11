
import React from 'react';
import EditableText from '../interaction/editable-text/editable-text.js';
import classNames from 'classnames';
import './header.less';
import PropTypes from 'prop-types';

export class Header extends React.Component {
  static propTypes = {
    title: PropTypes.string.isRequired,
    icon: PropTypes.string,
    actions: PropTypes.array,
    onTitleRename: PropTypes.func
  };

  render() {
    var iconClasses;
    if (this.props.icon) {
      iconClasses = classNames({
        'fa': true,
        ['fa-' + this.props.icon]: true
      });
    }


    var rightActions;
    if (!_.isEmpty(this.props.actions)) {
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

  renderTitle() {
    if (this.props.onTitleRename) {
      return <EditableText text={this.props.title} doubleClick={true} onChange={this.handleTitleRename.bind(this)} />;
    } else {
      return this.props.title;
    }
  }

  handleTitleRename(newTitle) {
    this.props.onTitleRename(newTitle);
  }

  renderActions() {
    let actions = this.props.actions.map(action => {
      let classes = classNames({
        'fa': true,
        ['fa-' + action.icon]: true
      });
      return <li key={action.icon} onClick={this.handleActionClick.bind(this, action)}><i className={classes}/></li>;
    });

    return <ul className="actions">{actions}</ul>;
  }

  handleActionClick(action) {
    if (action.callback) {
      action.callback();
    }
  }
}

export class HeaderAction {
  constructor(icon, callback) {
    this.icon = icon;
    this.callback = callback;
  }
}

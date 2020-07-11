
import React from 'react';
import classNames from 'classnames';
import './children-type-switcher.less';
import PropTypes from 'prop-types';

export default class ChildrenTypeSwitcher extends React.Component {
  // TODO: change
  static propTypes = {
    types: PropTypes.array.isRequired,
    selected: PropTypes.string.isRequired,
    onChange: PropTypes.func.isRequired
  };

  render() {
    let list = this.props.types.map(type => {
      let classes = classNames({
        [type]: true,
        active: this.props.selected === type
      });
      return <li className={classes} key={type} onClick={this.handleClick.bind(this, type)}><i/></li>;
    });

    return (
      <ul className="children-type-switcher">
        {list}
      </ul>
    );
  }

  handleClick(key) {
    this.props.onChange(key);
  }
}

import React from 'react';
import './scrollable.less';
import PropTypes from 'prop-types';

export default class Scrollable extends React.Component {
  static propTypes = {
    nbItems: PropTypes.number.isRequired,
    onNeedMore: PropTypes.func,
    loadMoreItems: PropTypes.number,
    initialTopInset: PropTypes.number,
  };

  componentDidMount() {
    if (this.refs.scrollable) {
      this.refs.scrollable.scrollTop = this.props.initialTopInset;
    }
  }

  render() {
    if (this.props.nbItems != this.lastNbItems) {
      this.lastNbItems = this.props.nbItems;
      this.loadingMore = false;
    }

    return (
      <div className="scrollable" ref="scrollable" onScroll={this.handleCollectionScroll.bind(this)}>
        {this.props.children}
      </div>
    );
  }

  handleCollectionScroll(e) {
    let ul = this.refs.scrollable;
    if (ul && !this.loadingMore && this.props.nbItems > 0 && this.props.loadMoreItems) {
      let scrollPosition = ul.scrollTop;
      let scrollHeight = (ul.scrollHeight - ul.clientHeight);
      let itemAvgHeight = scrollHeight / this.props.nbItems;
      let nbItemsToScroll = (scrollHeight - scrollPosition) / itemAvgHeight;

      if (nbItemsToScroll <= this.props.loadMoreItems) {
        if (this.props.onNeedMore && this.props.onNeedMore()) {
          this.loadingMore = true;
        }
      }
    }
  }
}

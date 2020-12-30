import React from 'react';
import { ModalStore } from '../../store/modal-store';
import './modal.less';

export default class Modal extends React.Component {

  render() {
    if (this.props.children) {
      return (
        <div id='modal' onClick={this.handleBackgroundClick.bind(this)}>
          <div className='content' onClick={this.handleContentClick.bind(this)}>
            {this.props.children}
          </div>
        </div>
      );
    }
  }

  handleContentClick(e) {
    e.stopPropagation();
  }

  handleBackgroundClick() {
    ModalStore.hideModal();
  }

  componentDidMount() {
    document.addEventListener('keydown', this.handleKeyDown, false);
  }

  componentWillUnmount() {
    document.removeEventListener('keydown', this.handleKeyDown, false);
  }

  handleKeyDown(e) {
    // on press escape
    if (e.keyCode === 27) {
      ModalStore.hideModal();
    }
  }

}


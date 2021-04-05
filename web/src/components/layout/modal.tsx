import React from 'react';
import { ModalStore } from '../../store/modal-store';
import './modal.less';


interface IProps {
  children: React.ReactNode;
}

export default class Modal extends React.Component<IProps> {
  render(): React.ReactNode {
    if (this.props.children) {
      return (
        <div id='modal' onClick={this.handleBackgroundClick}>
          <div className='content' onClick={this.handleContentClick}>
            {this.props.children}
          </div>
        </div>
      );
    }
  }

  componentDidMount(): void {
    document.addEventListener('keydown', this.handleKeyDown, false);
  }

  componentWillUnmount() : void{
    document.removeEventListener('keydown', this.handleKeyDown, false);
  }

  private handleContentClick = (e: React.MouseEvent) => {
    e.stopPropagation();
  }

  private handleBackgroundClick = () => {
    ModalStore.hideModal();
  }

  private handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      ModalStore.hideModal();
    }
  }
}

import React from 'react';
import { Shortcuts } from '../../shortcuts';
import { IStores, StoresContext } from '../../stores/stores';
import { CancellableEvent } from '../../utils/events';
import './modal.less';


interface IProps {
  children: React.ReactNode;
}

export default class Modal extends React.Component<IProps> {
  static contextType = StoresContext;
  declare context: IStores;

  render(): React.ReactNode {
    if (this.props.children) {
      return (
        <div id="modal" onClick={this.handleBackgroundClick}>
          <div className="content" onClick={this.preventMouseDefault} onMouseOver={this.preventMouseDefault} onWheel={this.preventMouseDefault}>
            {this.props.children}
          </div>
        </div>
      );
    }
  }

  componentDidMount(): void {
    document.addEventListener('keydown', this.handleKeyDown, false);
    Shortcuts.activateContext('modal');
  }

  componentWillUnmount(): void {
    document.removeEventListener('keydown', this.handleKeyDown, false);

    setTimeout(() => {
      // prevent keydown event that closed the modal to propagate
      Shortcuts.deactivateContext('modal');
    }, 500);
  }

  private preventMouseDefault = (e: CancellableEvent) => {
    // prevent clicking under modal
    e.stopPropagation();
  };

  private handleBackgroundClick = () => {
    this.context.session.hideModal(true);
  };

  private handleKeyDown = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') {
      this.context.session.hideModal(true);
    }
  };
}

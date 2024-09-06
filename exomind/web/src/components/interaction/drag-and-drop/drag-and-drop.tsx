
import classNames from 'classnames';
import * as React from 'react';
import './drag-and-drop.less';

interface IProps {
  children?: React.ReactNode;
  object?: unknown;
  parentObject?: unknown;
  draggable: boolean;
  droppable: boolean;
  dropPositions?: DropPosition[]; // needs to be sorted
  onDropOut?: (data: DragData) => void;
  onDropIn?: (data: DragData) => void;
}

interface IState {
  isDragged: boolean;
  isHovered: boolean;
  dropPosition?: DropPosition;
}

export interface DragData {
  effect: DragEffect;
  object: unknown;
  parentObject: unknown;
  position?: DropPosition;
}

export type DropPosition = 'before' | 'into' | 'after';

export type DragEffect = 'copy' | 'move';

let _drag_data_seq = 0;
const _drag_data: { [key: string]: DragData } = {};

export default class DragAndDrop extends React.Component<IProps, IState> {
  static defaultProps: IProps = {
    draggable: true,
    droppable: true,
    dropPositions: ['before', 'after'],
  };

  private isHovered = false;
  private isIndicatorHovered = false;
  private divElem: React.RefObject<HTMLDivElement> = React.createRef();

  constructor(props: IProps) {
    super(props);

    this.state = {
      isDragged: false,
      isHovered: false,
    };
  }

  render(): React.ReactNode {
    const classes = classNames({
      'drag-and-drop': true,
    });

    return (
      <div
        className={classes}
        ref={this.divElem}
        draggable={this.props.draggable}
        onDragStart={this.handleDragStart}
        onDragEnter={this.handleDragOver}
        onDragOver={this.handleDragOver}
        onDrop={this.handleDropIn}
        onDragLeave={this.handleDragLeave}
        onDragEnd={this.handleDragEnd}
      >

        {this.props.children}
        {this.renderDropIndicator()}
      </div>
    );
  }

  static getDraggedData(event: React.DragEvent | DragEvent, take = true): DragData | null {
    const item = event.dataTransfer.getData('item');
    if (!item) {
      return null;
    }

    const sequence = parseInt(item);
    const data = _drag_data[sequence];
    if (!data) {
      return null;
    }

    if (take) {
      delete _drag_data[sequence];
    }

    return data;
  }

  private renderDropIndicator() {
    if (this.state.isHovered) {
      const classes = classNames({
        'drop-indicator': true,
        [this.state.dropPosition]: true,
      });

      return <div
        className={classes}
        onDragOver={this.handleDragOverIndicator}
        onDragLeave={this.handleDragLeaveIndicator}
        onDrop={this.handleDropIn}
      />;
    }
  }

  private handleDragStart = (event: React.DragEvent): void => {
    const effect = (event.shiftKey || event.altKey || event.metaKey) ? 'copy' : 'move';
    event.dataTransfer.effectAllowed = effect;

    const sequence = (_drag_data_seq++).toString();
    _drag_data[sequence] = {
      effect: effect,
      object: this.props.object,
      parentObject: this.props.parentObject
    };

    event.dataTransfer.setData('item', sequence);
    this.setState({
      isDragged: true
    });
  };

  private handleDropIn = (event: React.DragEvent): void => {
    if (this.state.isDragged) {
      // we're the one being dragged, and dropped on ourself
      return;
    }

    this.setState({
      isHovered: false
    });

    const data = DragAndDrop.getDraggedData(event);
    if (this.props.onDropIn && data) {
      this.props.onDropIn({
        position: this.state.dropPosition,
        ...data
      });
    }
  };

  private handleDragOver = (event: React.DragEvent): void => {
    event.preventDefault();
    this.isHovered = true;

    // if we aren't being dragged, this means it's coming from another object
    if (!this.state.isDragged && this.props.droppable && !this.state.isHovered) {
      this.setState({
        isHovered: true
      });
    }

    // calculate the position of the mouse on us
    if (this.state.isHovered && this.props.dropPositions.length > 0) {
      const rect = this.divElem.current.getBoundingClientRect();
      const percentY = Math.abs((event.clientY - rect.y) / rect.height);

      const idx = Math.min(Math.floor(this.props.dropPositions.length * percentY), this.props.dropPositions.length - 1);
      const newPos = this.props.dropPositions[idx];
      if (newPos != this.state.dropPosition) {
        this.setState({
          dropPosition: newPos,
        });
      }
    }
  };

  private handleDragOverIndicator = () => {
    this.isIndicatorHovered = true;
  };

  private handleDragLeaveIndicator = () => {
    this.isIndicatorHovered = false;
    this.checkFullyOut();
  };

  private handleDragLeave = (): void => {
    if (this.isIndicatorHovered) {
      return;
    }

    this.isHovered = false;
    this.checkFullyOut();
  };

  // this is needed to debounce since we get leave event as soon as mouse pass from element inside the li element
  private checkFullyOut() {
    setTimeout(() => {
      if (!this.isHovered && !this.isIndicatorHovered) {
        this.setState({
          isHovered: false
        });
      }
    }, 10);
  }

  private handleDragEnd = (event: React.DragEvent): void => {
    const effect = event.dataTransfer.dropEffect;
    if (effect === 'move' || effect === 'copy') {
      if (this.props.onDropOut) {
        this.props.onDropOut({
          object: this.props.object,
          effect,
          parentObject: this.props.parentObject,
        });
      }
    }

    this.setState({
      isDragged: false
    });
  };
}

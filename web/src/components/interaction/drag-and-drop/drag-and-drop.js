
import * as React from 'react';
import PropTypes from 'prop-types';
import './drag-and-drop.less';

export default class DragAndDrop extends React.Component {
  static DraggedSequence = 0;
  static DraggedData = {};

  static propTypes = {
    object: PropTypes.object,
    parentObject: PropTypes.object,
    draggable: PropTypes.bool,
    droppable: PropTypes.bool,
    onDropOut: PropTypes.func,
    onDropIn: PropTypes.func
  };

  static defaultProps = {
    draggable: true,
    droppable: true
  };

  constructor(props) {
    super(props);

    this.state = {
      dropHovered: false,
      beingDragged: false
    };
  }

  render() {
    return (
      <div
        className="drag-and-drop"
        draggable={this.props.draggable}
        onDragStart={this.handleDragStart.bind(this)}
        onDragEnter={this.handleDragOver.bind(this)}
        onDragOver={this.handleDragOver.bind(this)}
        onDrop={this.handleDragDrop.bind(this)}
        onDragLeave={this.handleDragLeave.bind(this)}
        onDragEnd={this.handleDragEnd.bind(this)}>

        {this.props.children}
        {this.renderDropIndicator()}
      </div>
    );
  }

  renderDropIndicator() {
    if (this.state.dropHovered) {
      return <div className="drop-indicator" />;
    }
  }

  handleDragStart(event) {
    let effect = (event.shiftKey || event.altKey || event.metaKey) ? 'copy' : 'move';
    event.dataTransfer.effectAllowed = effect;

    let sequence = DragAndDrop.DraggedSequence++;
    DragAndDrop.DraggedData[sequence] = {
      effect: effect,
      object: this.props.object,
      parentObject: this.props.parentObject
    };

    event.dataTransfer.setData('item', sequence);
    this.setState({
      beingDragged: true
    });
  }

  handleDragDrop(event) {
    this.setState({
      dropHovered: false
    });
    let sequence = parseInt(event.dataTransfer.getData('item'));
    let data = DragAndDrop.DraggedData[sequence];
    delete DragAndDrop.DraggedData[sequence];
    if (this.props.onDropIn) {
      this.props.onDropIn(data.object, data.effect, data.parentObject);
    }
  }

  handleDragOver(event) {
    // if we aren't being dragged, this means it's coming from another object
    if (!this.state.beingDragged && this.props.droppable) {
      event.preventDefault();
      this.dropLeft = false;
      this.setState({
        dropHovered: true
      });
    }
  }

  handleDragLeave(event) {
    // this thing is needed to debounce since we get leave event as soon as mouse pass from element inside the li element
    this.dropLeft = true;
    setTimeout(() => {
      if (this.dropLeft) {
        this.setState({
          dropHovered: false
        });
      }
    }, 10)
  }

  handleDragEnd(event) {
    let effect = event.dataTransfer.dropEffect;
    if (effect === 'move' || effect === 'copy') {
      if (this.props.onDropOut) {
        this.props.onDropOut(this.props.object, effect, this.props.parentObject);
      }
    }
    this.setState({
      beingDragged: false
    });
  }

}



import React from 'react';
import './editable-text.less';
import PropTypes from 'prop-types';
import * as _ from 'lodash';
export default class EditableText extends React.Component {
  static propTypes = {
    initEdit: PropTypes.bool,
    text: PropTypes.string,
    multiline: PropTypes.bool,
    doubleClick: PropTypes.bool,
    placeholder: PropTypes.string,
    onChange: PropTypes.func
  };

  static defaultProps = {
    multiline: false,
    finishOnBlur: true,
    placeholder: 'Click to change'
  };

  constructor(props) {
    super(props);

    this.state = {
      editMode: !!props.initEdit,
      value: props.text || ''
    };
  }

  componentDidUpdate(prevProps, prevState) {
    if (this.state.editMode) {
      this.ensureFocus();
    }

    if (this.props.text != prevProps.text) {
      this.setState({
        value: this.props.text
      });
    }
  }

  componentDidMount() {
    if (this.state.editMode) {
      this.ensureFocus();
    }
  }

  render() {
    if (this.state.editMode) {
      if (this.props.multiline) {
        return this.renderMultiEdit();
      } else {
        return this.renderSingleEdit();
      }
    } else {
      return this.renderRead();
    }
  }

  renderRead() {
    let singleClick = !this.props.doubleClick;
    let value = _.isEmpty(this.state.value) ? <span className="empty">{this.props.placeholder}</span> : this.state.value;
    return (
      <span
        className="editable-text"
        onClick={singleClick ? this.handleReadClick.bind(this) : null}
        onDoubleClick={this.handleReadClick.bind(this)}>
        {value}
      </span>
    );
  }

  handleReadClick(e) {
    this.setState({
      editMode: true
    });
    e.stopPropagation();
  }

  renderSingleEdit() {
    return <span className="editable-text"><input type="text" ref="inputText"
      onBlur={this.handleEditBlur.bind(this)}
      onChange={this.handleEditChange.bind(this)}
      onKeyUp={this.handleEditKeyPress.bind(this)}
      value={this.state.value} /></span>;
  }

  renderMultiEdit() {
    return <span className="editable-text"><textarea ref="inputText"
      onBlur={this.handleEditBlur.bind(this)}
      onChange={this.handleEditChange.bind(this)}
      onKeyUp={this.handleEditKeyPress.bind(this)}
      value={this.state.value} /></span>;
  }

  handleEditChange(event) {
    this.setState({
      value: event.target.value
    });
  }

  handleEditKeyPress(event) {
    if (event.keyCode === 27 || (!this.props.multiline && event.keyCode === 13)) { // escape or enter in single line
      this.editFinish();
    }
  }

  handleEditBlur() {
    this.editFinish();
  }

  editFinish() {
    this.setState({
      editMode: false
    });
    if (this.props.onChange) {
      this.props.onChange(this.state.value);
    }
  }

  ensureFocus() {
    let element = this.refs.inputText;
    if (element != document.activeElement) {
      element.focus();
      element.select();
    }
  }
}


import React from 'react';
import HtmlEditor from './html-editor.js';
import './html-editor-controls.less';
import PropTypes from 'prop-types';

export default class HtmlEditorControls extends React.Component {
  static propTypes = {
    editor: PropTypes.instanceOf(HtmlEditor)
  };

  constructor(props) {
    super(props);
  }

  render() {
    return <div className="html-editor-controls">
      <ul>
        <li><i className="icon bold" onClick={this.handleToggleSelectionTag.bind(this, 'b')} /></li>
        <li><i className="icon italic" onClick={this.handleToggleSelectionTag.bind(this, 'i')}/></li>
        <li><i className="icon underline" onClick={this.handleToggleSelectionTag.bind(this, 'u')}/></li>
        <li><i className="icon strikethrough" onClick={this.handleToggleSelectionTag.bind(this, 's')}/></li>
        <li><i className="icon list-ul" onClick={this.handleUnorderedList.bind(this, 'list-ul')}/></li>
        <li><i className="icon list-ol" onClick={this.handleOrderedList.bind(this, 'list-ol')}/></li>
        <li><i className="icon outdent" onClick={this.handleOutdent.bind(this, 'outdent')}/></li>
        <li><i className="icon indent" onClick={this.handleIndent.bind(this, 'indent')}/></li>
      </ul>
    </div>;
  }

  handleToggleSelectionTag(format) {
    if (this.props.editor) {
      this.props.editor.toggleSelectionTag(format);
    }
  }

  handleUnorderedList() {
    if (this.props.editor) {
      this.props.editor.makeUnorderedList();
    }
  }

  handleOrderedList() {
    if (this.props.editor) {
      this.props.editor.makeOrderedList();
    }
  }

  handleIndent() {
    if (this.props.editor) {
      this.props.editor.indent();
    }
  }

  handleOutdent() {
    if (this.props.editor) {
      this.props.editor.outdent();
    }
  }
}

import React from 'react';
import HtmlEditor from '../../components/interaction/html-editor/html-editor.js';
import './ios-html-editor.less';
import PropTypes from 'prop-types';

export default class IosHtmlEditor extends React.Component {
  static propTypes = {
    content: PropTypes.string
  };

  lastCursorY = -1;

  constructor(props) {
    super(props);
    this.state = {
      content: props.content
    }
  }

  UNSAFE_componentWillReceiveProps(newProps) {
    if (newProps.action) {
      this.handleAction(newProps.action);
    }

    if (newProps.content) {
      this.setState({
        content: newProps.content
      });
    }
  }

  render() {
    return (
      <HtmlEditor
        content={this.state.content}
        placeholder="Type your text here"
        onBound={this.handleBound.bind(this)}
        onChange={this.handleContentChange.bind(this)}
        onCursorChange={this.handleCursorChange.bind(this)}
        onFocus={this.handleFocus.bind(this)}
      />
    );
  }

  handleBound(editor) {
    this.setState({
      editor: editor
    });
  }

  handleContentChange(newContent) {
    this.newContent = newContent;

    // Send first event directly. Otherwise we debounce
    if (!this.onceReported) {
      this.onceReported = true;
      sendIos({
        content: newContent
      });
    } else {
      // debounces event to ios
      setTimeout(() => {
        if (this.newContent === newContent) {
          sendIos({
            content: newContent
          });
        }
      }, 500);
    }

    this.sendCursor();
  }

  handleCursorChange() {
    this.sendCursor();
  }

  handleFocus(e) {
    setTimeout(() => {
      this.handleCursorChange();
    }, 100);
  }

  sendCursor() {
    let contentDoc = document.getElementsByTagName('iframe')[0].contentDocument;
    if (contentDoc) {
      let range = contentDoc.getSelection().getRangeAt(0);
      let rects = range.getClientRects();
      if (!_.isEmpty(rects)) {
        let cursorY = rects[0].top;
        if (this.lastCursorY != cursorY) {
          sendIos({
            cursorY: cursorY
          });
          this.lastCursorY = cursorY;
        }
      }
    }
  }

  handleAction(name) {
    switch (name) {
    case 'bold':
      this.state.editor.toggleSelectionTag('b');
      break;
    case 'italic':
      this.state.editor.toggleSelectionTag('i');
      break;
    case 'list-ul':
      this.state.editor.makeUnorderedList();
      break;
    case 'list-ol':
      this.state.editor.makeOrderedList();
      break;
    case 'indent':
      this.state.editor.indent();
      break;
    case 'outdent':
      this.state.editor.outdent();
      break;
    default:
      console.log('Unhandled action ' + name);
    }
  }

}


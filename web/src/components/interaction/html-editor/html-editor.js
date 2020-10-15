
import React from 'react';
import './html-editor.less';
import Squire from 'squire-rte';
import PropTypes from 'prop-types';
import * as _ from 'lodash';

export default class HtmlEditor extends React.Component {
  editor = null;

  static propTypes = {
    onBound: PropTypes.func,
    content: PropTypes.string,
    placeholder: PropTypes.string,
    onChange: PropTypes.func,
    onBlur: PropTypes.func,
    onFocus: PropTypes.func,
    onCursorChange: PropTypes.func,
    initialFocus: PropTypes.bool
  };

  static defaultProps = {
    initialFocus: true
  };

  constructor(props) {
    super(props);
    this.state = {
      content: null,
      isFocus: props.initialFocus
    };

    this.iframeRef = React.createRef();
  }

  componentDidMount() {
    this.loadEditor();
    if (this.props.onBound) {
      this.props.onBound(this);
    }
  }

  componentWillUnmount() {
    if (this.editor) {
      this.editor.destroy();
    }
  }

  UNSAFE_componentWillReceiveProps(newProps) {
    if (newProps.content !== this.props.content && newProps.content !== this.state.content && this.editor) {
      this.setState({
        content: newProps.content
      });
      this.editor.setHTML(newProps.content);
    }
  }

  render() {
    var placeholder = null;
    if (_.isEmpty(this.state.content) && _.isEmpty(this.props.content) && !this.state.isFocus && this.props.placeholder) {
      placeholder = <div className="placeholder" onClick={this.handlePlaceholderClick.bind(this)}>{this.props.placeholder}</div>;
    }

    return <div className="html-editor">
      {placeholder}
      <iframe ref={this.iframeRef} />
    </div>;
  }

  loadEditor() {
    let iframeNode = this.iframeRef.current;
    let doc = iframeNode.contentDocument;
    doc.open();

    let darkmode;
    if (window.isHybridExomind) {
      // This is for iOS dark mode. If changed, change in iOS EmailBodyWebView
      darkmode = '<style>@media (prefers-color-scheme: dark) { body { color: white; background-color: black } a { color: #4285f4; } }</style>';
    }

    // width hack: http://stackoverflow.com/questions/23083462/how-to-get-an-iframe-to-be-responsive-in-ios-safari
    doc.write(
      '<!DOCTYPE html><html style="height: 100%; font-size: 16px; font-family: Segoe UI, Arial, Helvetica, sans-serif">' +
      '<meta><title></title></meta>' +
      + darkmode +
      '<body style="height: 100%; margin: 0; -webkit-tap-highlight-color: transparent; width: 1px; min-width: 100%;"></body></html>');
    doc.close();

    this.editor = new Squire(doc, {
      // stuff here
    });

    this.editor.addEventListener('input', (s, e) => this.handleContentChange(e));
    this.editor.addEventListener('blur', (s, e) => this.handleBlur(e));
    this.editor.addEventListener('focus', (s, e) => this.handleFocus(e));
    this.editor.addEventListener('pathChange', (s, e) => this.handleCursorChange(e));
    this.editor.setKeyHandler('meta-[', (s, e, r) => this.outdent(e, r));
    this.editor.setKeyHandler('meta-]', (s, e, r) => this.indent(e, r));
    this.editor.setKeyHandler('meta-shift-o', (s, e, r) => this.makeOrderedList(e, r));
    this.editor.setKeyHandler('meta-shift-u', (s, e, r) => this.makeUnorderedList(e, r));
    this.editor.setHTML(this.state.content || this.props.content || '');

    if (this.props.initialFocus) {
      this.editor.focus();
    }
  }

  handlePlaceholderClick() {
    this.editor.focus();
  }

  handleContentChange() {
    let content = this.getContent();
    this.setState({
      content: content
    });
    if (this.props.onChange) {
      this.props.onChange(content);
    }
  }

  handleBlur() {
    this.setState({
      isFocus: false
    });
    if (this.props.onBlur) {
      this.props.onBlur();
    }
  }

  handleFocus() {
    this.setState({
      isFocus: true
    });
    if (this.props.onFocus) {
      this.props.onFocus();
    }
  }

  handleCursorChange() {
    if (this.props.onCursorChange) {
      this.props.onCursorChange();
    }
  }

  getContent() {
    return (this.editor) ? this.editor.getHTML() : null;
  }

  toggleSelectionTag(tag) {
    let range = this.editor.getSelection();
    if (this.editor.hasFormat(tag, null, range)) {
      this.editor.changeFormat(null, { tag: tag }, range);
    } else {
      this.editor.changeFormat({ tag: tag }, null, range);
    }
  }

  increaseQuoteLevel(e) {
    this.maybePreventDefault(e);
    this.editor.increaseQuoteLevel();
  }

  decreaseQuoteLevel(e) {
    this.maybePreventDefault(e);
    this.editor.decreaseQuoteLevel();
  }

  makeUnorderedList(e) {
    this.maybePreventDefault(e);
    this.editor.makeUnorderedList();
  }

  makeOrderedList(e) {
    this.maybePreventDefault(e);
    this.editor.makeOrderedList();
  }

  removeList(e) {
    this.maybePreventDefault(e);
    this.editor.removeList();
  }

  increaseListLevel(e) {
    this.maybePreventDefault(e);
    this.editor.increaseListLevel();
  }

  decreaseListLevel(e) {
    this.maybePreventDefault(e);
    this.editor.decreaseListLevel();
  }

  indent(e, range) {
    this.maybePreventDefault(e);
    if (!range) {
      range = this.editor.getSelection();
    }
    var root = range.commonAncestorContainer;
    if (this.inList(root)) {
      this.increaseListLevel();
    } else {
      this.increaseQuoteLevel();
    }
  }

  outdent(e, range) {
    this.maybePreventDefault(e);
    if (!range) {
      range = this.editor.getSelection();
    }
    var root = range.commonAncestorContainer;
    if (this.inList(root)) {
      this.decreaseListLevel();
    } else {
      this.decreaseQuoteLevel();
    }
  }

  maybePreventDefault(e) {
    if (e) {
      e.preventDefault();
    }
  }

  findParent(node, predicate) {
    do {
      if (predicate(node)) {
        return node;
      }
    } while (node == node.parentNode);
    return null;
  }

  inList(node) {
    return !!this.findParent(node, n => n.nodeName === 'LI');
  }

}



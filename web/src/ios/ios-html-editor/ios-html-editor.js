import React from "react";
import NewHtmlEditor from "../../components/interaction/html-editor/new-html-editor";
import PropTypes from "prop-types";
import "./ios-html-editor.less";

export default class IosHtmlEditor extends React.Component {
  static propTypes = {
    content: PropTypes.string,
  };

  cursorY = -1;
  content = null;

  constructor(props) {
    super(props);

    // content is set via state because it may be empty at time in props
    // props are used message passing, not as full state
    this.state = {
      content: props.content,
    };
  }

  componentDidUpdate(prevProps) {
    // if we receive an action
    if (this.props.action) {
      this.handleAction(this.props.action);
    }

    // if we receive new content and we update editor on same stack, it crashes
    setTimeout(() => {
      if (this.props.content) {
        this.setState({
          content: this.props.content,
        });
      }
    });
  }

  render() {
    return (
      <NewHtmlEditor
        content={this.state.content}
        onBound={this.handleBound}
        onChange={this.handleContentChange}
        onCursorChange={this.handleCursorChange}
        onLinkClick={this.handleLinkClick}
        allowLinkFocusClick={true}
      />
    );
  }

  handleBound = (editor) => {
    this.setState({ editor });
  }

  handleContentChange = (newContent) => {
    this.content = newContent;
    sendIos(
      JSON.stringify({
        content: this.content,
        cursorY: this.cursorY,
      })
    );
  }

  handleCursorChange = (cursor) => {
    if (cursor && cursor.rect) {
      let newCursorY = cursor.rect.top;
      if (this.cursorY != newCursorY) {
        this.cursorY = newCursorY;
        sendIos(
          JSON.stringify({
            content: this.content,
            cursorY: this.cursorY,
          })
        );
      }
    }
  }

  handleLinkClick = (url, e) => {
    e.preventDefault();
    e.stopPropagation();
    sendIos(
      JSON.stringify({
        link: url,
      })
    );
  };

  handleAction(name) {
    switch (name) {
      case "bold":
        this.state.editor.toggleInlineStyle("BOLD");
        break;
      case "italic":
        this.state.editor.toggleInlineStyle("ITALIC");
        break;
      case "strikethrough":
        this.state.editor.toggleInlineStyle("STRIKETHROUGH");
        break;
      case "code":
        this.state.editor.toggleInlineStyle("CODE");
        break;
      case "header-toggle":
        this.state.editor.toggleHeader();
        break;
      case "list-ul":
        this.state.editor.toggleBlockType("unordered-list-item");
        break;
      case "list-ol":
        this.state.editor.toggleBlockType("ordered-list-item");
        break;
      case "list-todo":
        this.state.editor.toggleBlockType("todo-list-item");
        break;
      case "code-block":
        this.state.editor.toggleBlockType("code-block");
        break;
      case "indent":
        this.state.editor.indent();
        break;
      case "outdent":
        this.state.editor.outdent();
        break;
      default:
        console.log("Unhandled action " + name);
    }
  }
}

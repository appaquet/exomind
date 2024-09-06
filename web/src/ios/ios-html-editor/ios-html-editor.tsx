import React from "react";
import HtmlEditor, { EditorCursor } from "../../components/interaction/html-editor/html-editor";
import { CancellableEvent } from "../../utils/events";
import { sendIos } from "../hybrid";
import "./ios-html-editor.less";

interface IProps {
    action?: string;
    content?: string;
}

interface IState {
    editor?: HtmlEditor
}

export default class IosHtmlEditor extends React.Component<IProps, IState> {
    // state is not used for these variables become of some feedback bug where
    // the note constantly gets changed back to previous content
    private cursorY = 0;
    private content = '';

    constructor(props: IProps) {
        super(props);
        this.content = props.content;
    }

    componentDidUpdate(prevProps: IProps) {
        // if we receive an action
        if (this.props.action) {
            this.handleAction(this.props.action);
        }

        // if we receive new content and we update editor on same stack, it crashes
        if (this.props.content && this.props.content != prevProps.content) {
            this.content = this.props.content;
            this.setState({});
        }
    }

    render() {
        return (
            <HtmlEditor
                content={this.content}
                onBound={this.handleBound}
                onChange={this.handleContentChange}
                onCursorChange={this.handleCursorChange}
                onLinkClick={this.handleLinkClick}
                initialFocus={false}
                placeholder="Type here..."
            />
        );
    }

    handleBound = (editor: HtmlEditor) => {
        this.setState({ editor });
    };

    handleContentChange = (newContent: string) => {
        this.content = newContent;

        sendIos(
            JSON.stringify({
                content: newContent,
                cursorY: this.cursorY,
            })
        );
    };

    handleCursorChange = (cursor: EditorCursor) => {
        if (cursor && cursor.rect) {
            const newCursorY = cursor.rect.top;
            if (this.cursorY != newCursorY) {
                this.cursorY = newCursorY;
                sendIos(
                    JSON.stringify({
                        content: this.content,
                        cursorY: newCursorY,
                    })
                );
            }
        }
    };

    handleLinkClick = (url: string, e: CancellableEvent) => {
        e.stopPropagation();

        sendIos(
            JSON.stringify({
                link: url,
            })
        );
    };

    handleAction(name: string) {
        switch (name) {
            case 'bold':
                this.state.editor.toggleInlineStyle('BOLD');
                break;
            case 'italic':
                this.state.editor.toggleInlineStyle('ITALIC');
                break;
            case 'strikethrough':
                this.state.editor.toggleInlineStyle('STRIKETHROUGH');
                break;
            case 'code':
                this.state.editor.toggleInlineStyle('CODE');
                break;
            case 'header-toggle':
                this.state.editor.toggleHeader();
                break;
            case 'list-ul':
                this.state.editor.toggleBlockType('unordered-list-item');
                break;
            case 'list-ol':
                this.state.editor.toggleBlockType('ordered-list-item');
                break;
            case 'list-todo':
                this.state.editor.toggleBlockType('todo-list-item');
                break;
            case 'code-block':
                this.state.editor.toggleBlockType('code-block');
                break;
            case 'indent':
                this.state.editor.indent();
                break;
            case 'outdent':
                this.state.editor.outdent();
                break;
            default:
                console.error(`Unhandled action ${name}`);
        }
    }
}

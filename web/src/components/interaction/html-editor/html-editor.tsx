
import React, { RefObject, SyntheticEvent, KeyboardEvent } from 'react';
import { EditorState, RichUtils, DraftEditorCommand, getVisibleSelectionRect, DraftBlockType, DraftInlineStyle, ContentState, Modifier } from 'draft-js';
import './html-editor.less';
import 'draft-js/dist/Draft.css';

import Editor from 'draft-js-plugins-editor';
import { convertToHTML, convertFromHTML } from 'draft-convert';

// No type definitions for this
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import createMarkdownShortcutsPlugin from 'draft-js-markdown-shortcuts-plugin';
const plugins = [createMarkdownShortcutsPlugin()];

const defaultInitialFocus = true;
const listMaxDepth = 4;

interface IProps {
  content: string;
  placeholder?: string;

  onBound?: (editor: HtmlEditor) => void;
  onChange?: (content: string) => void;
  onFocus?: () => void;
  onBlur?: () => void;
  onCursorChange?: (cursor: EditorCursor) => void;
  initialFocus?: boolean;
}

interface IState {
  editorState: EditorState;
  initialContent: ContentState;
  localChanges: boolean;
}

export default class HtmlEditor extends React.Component<IProps, IState> {
  editorRef: RefObject<Editor>;
  lastTriggeredChangeState?: ContentState;

  constructor(props: IProps) {
    super(props);

    const htmlContent = convertOldHTML(props.content);

    let state;
    if (htmlContent) {
      state = EditorState.createWithContent(fromHTML(htmlContent));
    } else {
      state = EditorState.createEmpty();
    }

    this.state = {
      editorState: state,
      initialContent: state.getCurrentContent(),
      localChanges: false,
    };

    this.editorRef = React.createRef();
  }

  componentDidMount(): void {
    if (this.props.onBound) {
      this.props.onBound(this);
    }

    if (this.props.initialFocus ?? defaultInitialFocus) {
      this.editorRef.current.focus();
    }
  }

  componentDidUpdate(prevProps: IProps): void {
    const htmlContent = toHTML(this.state.editorState.getCurrentContent());
    if (!this.state.localChanges && this.props.content !== prevProps.content && this.props.content !== htmlContent) {
      const htmlContent = convertOldHTML(this.props.content);
      const state = EditorState.push(this.state.editorState, fromHTML(htmlContent), 'insert-characters');
      this.setState({
        editorState: state,
        initialContent: state.getCurrentContent(),
      });
    }
  }

  render(): React.ReactNode {
    return <div className="html-editor">
      <Editor
        ref={this.editorRef}
        plugins={plugins}
        editorState={this.state.editorState}
        onChange={this.onChange.bind(this)}
        handleKeyCommand={this.handleKeyCommand.bind(this)}
        placeholder={this.props.placeholder}
        onFocus={this.handleFocus.bind(this)}
        onBlur={this.handleBlur.bind(this)}
      />
    </div>;
  }

  private onChange(editorState: EditorState): void {
    const contentState = editorState.getCurrentContent();

    this.setState({
      editorState,
    });

    let hasChanged = false;
    if (this.lastTriggeredChangeState != contentState && this.state.initialContent != contentState) {
      this.lastTriggeredChangeState = contentState;
      this.setState({
        localChanges: true
      });
      hasChanged = true;
    }

    // trigger separately so that we don't get props changed on same stack
    setTimeout(() => {
      if (this.props.onChange && hasChanged) {
        const htmlContent = toHTML(contentState);
        this.props.onChange(htmlContent);
      }

      if (this.props.onCursorChange) {
        const selection = editorState.getSelection();
        const blockType = editorState.getCurrentContent().getBlockForKey(selection.getStartKey()).getType();
        const inlineStyle = editorState.getCurrentInlineStyle();
        const rect = getVisibleSelectionRect(window); // cursor window position

        this.props.onCursorChange({ blockType, inlineStyle, rect })
      }
    });
  }

  private handleKeyCommand(command: DraftEditorCommand, editorState: EditorState): string {
    const newState = RichUtils.handleKeyCommand(editorState, command);

    if (newState) {
      this.onChange(newState);
      return 'handled';
    }

    return 'not-handled';
  }

  private handleBlur(): void {
    if (this.props.onBlur) {
      this.props.onBlur();
    }
  }

  private handleFocus(): void {
    if (this.props.onFocus) {
      this.props.onFocus();
    }
  }

  toggleInlineStyle(style: string): void {
    this.onChange(RichUtils.toggleInlineStyle(this.state.editorState, style));
  }

  toggleBlockType(type: string): void {
    this.onChange(RichUtils.toggleBlockType(this.state.editorState, type));
  }

  toggleHeader(): void {
    const selection = this.state.editorState.getSelection();
    const blockType = this.state.editorState.getCurrentContent().getBlockForKey(selection.getStartKey()).getType();

    let newType;
    switch (blockType) {
      case 'header-one':
        newType = 'header-two';
        break;
      case 'header-two':
        newType = 'header-three';
        break;
      case 'header-three':
        newType = 'header-four';
        break;
      case 'header-four':
        newType = 'unstyled';
        break;
      default:
        newType = 'header-one';
    }

    if (newType) {
      this.toggleBlockType(newType);
    }
  }

  indent(e: SyntheticEvent): void {
    if (!e) {
      e = {
        shift: false,
        preventDefault: () => {
          // nothing to do
        },
      } as unknown as KeyboardEvent;
    }

    const newEditorState = RichUtils.onTab(
      e as KeyboardEvent,
      this.state.editorState,
      listMaxDepth,
    );

    this.onChange(newEditorState);
  }

  outdent(e: SyntheticEvent): void {
    if (!e) {
      e = {
        shift: true,
        preventDefault: () => {
          // nothing to do
        },
      } as unknown as KeyboardEvent;
    }

    const a = e as KeyboardEvent;
    a.shiftKey = true;
    const newEditorState = RichUtils.onTab(
      a,
      this.state.editorState,
      listMaxDepth,
    );

    this.onChange(newEditorState);
  }

  // https://gist.github.com/tonis2/cfeb6d044347d6f3cbab656d6a94eee2
  clearStyle(): void {
    const { editorState } = this.state
    const selection = editorState.getSelection()
    const contentState = editorState.getCurrentContent()
    const styles = editorState.getCurrentInlineStyle()

    const removeStyles = styles.reduce((state, style) => {
      return Modifier.removeInlineStyle(state, selection, style)
    }, contentState)

    const removeBlock = Modifier.setBlockType(removeStyles, selection, 'unstyled')

    this.setState({
      editorState: EditorState.push(
        editorState,
        removeBlock,
        'change-block-type'
      )
    })
  }
}

export interface EditorCursor {
  blockType: DraftBlockType;
  inlineStyle: DraftInlineStyle;
  rect: CursorRect;
}

export interface CursorRect {
  left: number;
  width: number;
  right: number;
  top: number;
  bottom: number;
  height: number;
}

function convertOldHTML(html: string | undefined): string {
  if (!html) {
    return html;
  }

  // Squire added usless new lines after list items
  return html.replace(/<br>\s*<\/li>/mgi, "</li>");
}

function fromHTML(html: string): ContentState {
  return convertFromHTML({
    // see https://github.com/HubSpot/draft-convert
  })(html);
}

function toHTML(content: ContentState): string {
  return convertToHTML({
    // see https://github.com/HubSpot/draft-convert
  })(content);
}
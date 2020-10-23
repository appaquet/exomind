
import React, { RefObject, SyntheticEvent, KeyboardEvent } from 'react';
import { Editor, EditorState, RichUtils, DraftEditorCommand, getVisibleSelectionRect, DraftBlockType, DraftInlineStyle, ContentState, Modifier, getDefaultKeyBinding, DraftInlineStyleType, ContentBlock, DraftHandleValue, SelectionState } from 'draft-js';
import { convertToHTML, convertFromHTML, Tag } from 'draft-convert';
import Debouncer from '../../../utils/debouncer';
import 'draft-js/dist/Draft.css';
import './html-editor.less';
import { Commands } from './commands';

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
  debouncer: Debouncer;

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
    this.debouncer = new Debouncer(200);
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
        editorState={this.state.editorState}
        onChange={this.onChange.bind(this)}
        handleKeyCommand={this.handleKeyCommand.bind(this)}
        keyBindingFn={this.handleKeyBinding.bind(this)}
        placeholder={this.props.placeholder}
        handleBeforeInput={this.handleBeforeInput.bind(this)}
        handleReturn={this.handleReturn.bind(this)}
        onFocus={this.handleFocus.bind(this)}
        onBlur={this.handleBlur.bind(this)}
      />
    </div>;
  }

  private onChange(editorState: EditorState): void {
    const contentState = editorState.getCurrentContent();
    let hasChanges = false;
    if (this.lastTriggeredChangeState != contentState && this.state.initialContent != contentState) {
      this.lastTriggeredChangeState = contentState;
      hasChanges = true;
    }

    this.setState({
      editorState,
      localChanges: this.state.localChanges || hasChanges,
    });


    this.debouncer.debounce(() => {
      if (this.props.onChange && hasChanges) {
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

  private handleBeforeInput(chars: string, editorState: EditorState): DraftHandleValue {
    const curContent = editorState.getCurrentContent();
    const curSel = editorState.getSelection();

    const maxPrefixLen = 6;
    const prefixStyle: { [prefix: string]: string } = {
      '*': 'unordered-list-item',
      '-': 'unordered-list-item',
      '1.': 'ordered-list-item',
      '#': 'header-one',
      '##': 'header-two',
      '###': 'header-three',
      '####': 'header-four',
      '#####': 'header-five',
    };

    // if we just type a space, and we are not beyond the biggest prefix length
    if (chars == ' ' && curSel.getEndOffset() <= maxPrefixLen) {
      const curBlock = curContent.getBlockForKey(curSel.getFocusKey());

      // only support this if we're in an unstyled block
      if (curBlock.getType() != 'unstyled') {
        return 'not-handled';
      }

      // check if we have a style for this prefix
      const linePrefix = curBlock.getText().slice(0, maxPrefixLen);
      if (linePrefix in prefixStyle) {
        // remove pre characters
        const removeSel = SelectionState.createEmpty(curBlock.getKey()).merge({
          anchorOffset: 0,
          focusOffset: linePrefix.length,
        });
        const newContent = Modifier.replaceText(curContent, removeSel, '');
        let newState = EditorState.push(editorState, newContent, 'remove-range');

        // add list style
        newState = RichUtils.toggleBlockType(newState, prefixStyle[linePrefix]);

        // put selection after bullet
        newState = EditorState.forceSelection(newState, newContent.getSelectionAfter());

        this.onChange(newState);

        return 'handled';
      }
    }

    return 'not-handled';
  }

  private handleKeyBinding(e: KeyboardEvent): DraftEditorCommand | null {
    if (e.key === 'Tab') {
      e.preventDefault();

      const currentState = this.state.editorState;
      const currentSelection = currentState.getSelection();
      const currentContent = currentState.getCurrentContent();
      const blockType = currentContent.getBlockForKey(currentSelection.getStartKey()).getType();

      let newEditorState;
      if (blockType == 'unordered-list-item' || blockType == 'ordered-list-item') {
        newEditorState = RichUtils.onTab(e, currentState, listMaxDepth);
      } else if (!e.shiftKey) {
        newEditorState = Commands.handleIndentText(currentState);
      } else {
        newEditorState = Commands.handleOutdentText(currentState);
      }

      if (newEditorState && newEditorState != currentState) {
        this.onChange(newEditorState);
        return;
      }
    }

    return getDefaultKeyBinding(e);
  }

  private handleReturn(e: KeyboardEvent, editorState: EditorState): DraftHandleValue {
    const currentState = editorState;
    const currentSelection = currentState.getSelection();
    const currentContent = currentState.getCurrentContent();
    const currentBlock = currentContent.getBlockForKey(currentSelection.getStartKey());
    const blockType = currentBlock.getType();

    // remove block style when we return inside a header
    if (blockType.startsWith('header-')) {
      this.onChange(Commands.createUnstyledNextBlock(editorState));
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
    // https://github.com/HubSpot/draft-convert
    htmlToBlock: (nodeName, _node) => {
      if (nodeName === 'code') {
        return 'code-block';
      }
    }
  })(html);
}

function toHTML(content: ContentState): string {
  return convertToHTML({
    // https://github.com/HubSpot/draft-convert
    blockToHTML: (block) => {
      // types are incorrect
      const tBlock = block as unknown as { type: string };
      if (tBlock.type === 'code-block') {
        return <code />;
      }
    }
  })(content);
}
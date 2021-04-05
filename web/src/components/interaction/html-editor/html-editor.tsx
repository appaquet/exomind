
import React, { RefObject, SyntheticEvent, MouseEvent, KeyboardEvent } from 'react';
import { Editor, EditorState, RichUtils, DraftEditorCommand, KeyBindingUtil, getVisibleSelectionRect, DraftBlockType, DraftInlineStyle, ContentState, Modifier, getDefaultKeyBinding, DraftHandleValue, SelectionState, CompositeDecorator } from 'draft-js';
import Debouncer from '../../../utils/debouncer';
import 'draft-js/dist/Draft.css';
import './html-editor.less';
import { Commands } from './commands';
import { extractCurrentLink, findLinkEntities, Link, toggleLink } from './link';
import { convertOldHTML, fromHTML, toHTML } from './convert';
import { ModalStore } from '../../../store/modal-store';
import InputModal from '../../modals/input-modal/input-modal';

const listMaxDepth = 4;

interface IProps {
  content: string;
  placeholder?: string;

  onBound?: (editor: HtmlEditor) => void;
  onChange?: (content: string) => void;
  onFocus?: () => void;
  onBlur?: () => void;
  onCursorChange?: (cursor: EditorCursor) => void;
  onLinkClick?: (url: string, e: MouseEvent) => void;
  initialFocus?: boolean;
}

interface IState {
  editorState: EditorState;
  initialContent: ContentState;
  htmlContent?: string;
  localChanges: boolean;
}

export default class HtmlEditor extends React.Component<IProps, IState> {
  private editorRef: RefObject<Editor>;
  private lastTriggeredChangeState?: ContentState;
  private debouncer: Debouncer;

  private decorator = new CompositeDecorator([
    {
      strategy: findLinkEntities,
      component: Link,
      props: {
        editor: this,
      }
    },
  ]);

  private styleMap = {
    'CODE': {
      // use to be able to style inline code
      // see https://github.com/facebook/draft-js/issues/2302
      textDecorationColor: 'black',
    }
  };

  constructor(props: IProps) {
    super(props);

    const htmlContent = convertOldHTML(props.content);

    let editorState;
    if (htmlContent) {
      editorState = EditorState.createWithContent(fromHTML(htmlContent));
    } else {
      editorState = EditorState.createEmpty();
    }
    editorState = EditorState.set(editorState, { decorator: this.decorator });

    this.state = {
      editorState: editorState,
      initialContent: editorState.getCurrentContent(),
      localChanges: false,
    };

    this.editorRef = React.createRef();
    this.debouncer = new Debouncer(300);
  }

  componentDidMount(): void {
    if (this.props.onBound) {
      this.props.onBound(this);
    }

  const defaultInitialFocus = true;
    if (this.props.initialFocus ?? defaultInitialFocus) {
      this.editorRef.current.focus();
    }
  }

  componentDidUpdate(prevProps: IProps): void {
    if (!this.state.localChanges && this.props.content !== prevProps.content && this.props.content !== this.state.htmlContent) {
      const htmlContent = convertOldHTML(this.props.content);
      let editorState = EditorState.push(this.state.editorState, fromHTML(htmlContent), 'insert-characters');
      editorState = EditorState.set(editorState, { decorator: this.decorator });
      this.setState({
        editorState: editorState,
        initialContent: editorState.getCurrentContent(),
      });
    }
  }

  render(): React.ReactNode {
    return <div className="html-editor">
      <Editor
        ref={this.editorRef}
        editorState={this.state.editorState}
        placeholder={this.props.placeholder}
        customStyleMap={this.styleMap}
        onChange={this.handleOnChange}
        keyBindingFn={this.handleKeyBinding}
        handleKeyCommand={this.handleKeyCommand}
        handleBeforeInput={this.handleBeforeInput}
        handleReturn={this.handleReturn}
        onFocus={this.handleFocus}
        onBlur={this.handleBlur}

        spellCheck={true}
      />
    </div>;
  }

  private handleOnChange = (editorState: EditorState): void => {
    this.setState({
      editorState,
      localChanges: this.state.initialContent != editorState.getCurrentContent(),
    });

    this.debouncer.debounce(() => {
      const editorState = this.state.editorState;

      if (this.props.onChange) {
        const contentState = editorState.getCurrentContent();
        if (this.lastTriggeredChangeState != contentState && this.state.initialContent != contentState) {
          this.lastTriggeredChangeState = contentState;

          const htmlContent = toHTML(contentState);
          this.setState({ htmlContent });
          this.props.onChange(htmlContent);
        }
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

  private handleKeyBinding = (e: KeyboardEvent): DraftEditorCommand | null => {
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
        this.handleOnChange(newEditorState);
        return;
      }
    }

    if (e.key == 'k' && KeyBindingUtil.hasCommandModifier(e)) {
      e.preventDefault();

      setTimeout(() => {
        const done = (link?: string) => {
          ModalStore.hideModal();

          if (!link) {
            return;
          }
          if (!link.includes("://")) {
            link = 'entity://' + link;
          }

          this.toggleLink(link);
        };

        const currentLink = extractCurrentLink(this.state.editorState);
        ModalStore.showModal(() => {
          return <InputModal text='Enter link' initialValue={currentLink} onDone={done} />;
        })
      });
      return;
    }

    // otherwise fallback to default keyboard binding
    // see https://github.com/facebook/draft-js/blob/bc716b279299748d955bbdc398454bff45ea0191/src/component/utils/getDefaultKeyBinding.js
    return getDefaultKeyBinding(e);
  }

  /// Handles commands such as the one generated by `handleKeyBinding` (ex: bold, etc.)
  private handleKeyCommand = (command: DraftEditorCommand, editorState: EditorState): DraftHandleValue => {
    // check if draft.js can handle that command (ex: bold, etc.)
    const newState = RichUtils.handleKeyCommand(editorState, command);
    if (newState) {
      this.handleOnChange(newState);
      return 'handled';
    }

    return 'not-handled';
  }

  private handleBeforeInput = (chars: string, editorState: EditorState): DraftHandleValue => {
    // we only do insertions if we just typed a space after the prefix
    if (chars != ' ') {
      return;
    }

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
      '```': 'code-block',
    };

    // if we just type a space, and we are not beyond the biggest prefix length
    if (curSel.getEndOffset() <= maxPrefixLen) {
      const curBlock = curContent.getBlockForKey(curSel.getFocusKey());

      // only support this if we're in an unstyled block
      if (curBlock.getType() != 'unstyled') {
        return 'not-handled';
      }

      // check if we have a style for this prefix
      const linePrefix = curBlock.getText().slice(0, curSel.getEndOffset());
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

        this.handleOnChange(newState);

        return 'handled';
      }
    }

    return 'not-handled';
  }

  private handleReturn = (e: KeyboardEvent, editorState: EditorState): DraftHandleValue => {
    const curState = editorState;
    const curSel = curState.getSelection();
    const curContent = curState.getCurrentContent();
    const curBlock = curContent.getBlockForKey(curSel.getStartKey());

    // remove block style when we return inside a header
    // only do it if cursor is not at beginning of header. if it is, we're just trying to add an empty line above
    if (curBlock.getType().startsWith('header-') && curSel.getStartOffset() != 0) {
      this.handleOnChange(Commands.createUnstyledNextBlock(editorState));
      return 'handled';
    }

    return 'not-handled';
  }

  private handleBlur = (): void => {
    if (this.props.onBlur) {
      this.props.onBlur();
    }
  }

  private handleFocus = (): void => {
    if (this.props.onFocus) {
      this.props.onFocus();
    }
  }

  toggleInlineStyle(style: string): void {
    this.handleOnChange(RichUtils.toggleInlineStyle(this.state.editorState, style));
  }

  toggleBlockType(type: string): void {
    this.handleOnChange(RichUtils.toggleBlockType(this.state.editorState, type));
  }

  toggleLink(url: string | null): void {
    const newState = toggleLink(this.state.editorState, url);
    if (newState) {
      this.handleOnChange(newState);
    }
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

    this.handleOnChange(newEditorState);
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

    this.handleOnChange(newEditorState);
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
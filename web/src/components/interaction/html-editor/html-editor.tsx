import React from "react";
import { BangleEditor } from '@bangle.dev/react';
import { BangleEditorState, Plugin, BangleEditor as CoreBangleEditor } from '@bangle.dev/core';
import { bold, italic, link, bulletList, heading, listItem, orderedList, paragraph, underline, code, strike, codeBlock, blockquote, } from '@bangle.dev/base-components';
import { toHTMLString } from '@bangle.dev/utils';
import { EditorState, EditorView, NodeSelection, setBlockType } from "@bangle.dev/pm";
import { queryIsItalicActive, toggleItalic } from "@bangle.dev/base-components/dist/italic";
import { queryIsBoldActive, toggleBold } from "@bangle.dev/base-components/dist/bold";
import { keymap } from '@bangle.dev/pm';
import { queryIsHeadingActive, toggleHeading } from "@bangle.dev/base-components/dist/heading";
import { queryIsBulletListActive, queryIsTodoListActive, toggleBulletList, toggleTodoList } from "@bangle.dev/base-components/dist/bullet-list";
import { queryIsUnderlineActive, toggleUnderline } from "@bangle.dev/base-components/dist/underline";
import { queryIsCodeActive, toggleCode } from "@bangle.dev/base-components/dist/code";
import { queryIsStrikeActive, toggleStrike } from "@bangle.dev/base-components/dist/strike";
import { queryIsOrderedListActive, toggleOrderedList } from "@bangle.dev/base-components/dist/ordered-list";
import { queryIsBlockquoteActive, wrapInBlockquote } from "@bangle.dev/base-components/dist/blockquote";
import { queryIsCodeActiveBlock } from "@bangle.dev/base-components/dist/code-block";
import { indentListItem, outdentListItem } from "@bangle.dev/base-components/dist/list-item/list-item-component";
import Debouncer from "../../../utils/debouncer";
import { createLink, queryLinkAttrs, updateLink } from "@bangle.dev/base-components/dist/link";
import InputModal from "../../modals/input-modal/input-modal";
import { IStores, StoresContext } from "../../../stores/stores";

import './html-editor.less';
import '@bangle.dev/core/style.css';

// TODO: New URL / cmd-k popup
// TODO: Link tooltip on click (when focused)
// TODO: Find out how to change content

const defaultInitialFocus = false;

interface IProps {
    content: string;
    onBound?: (editor: HtmlEditor) => void;
    onChange?: (content: string) => void;
    onFocus?: () => void;
    onBlur?: () => void;
    onCursorChange?: (cursor: EditorCursor) => void;
    onLinkClick?: (url: string, e: MouseEvent) => void;
    initialFocus?: boolean;
}

interface IState {
    content?: string;
    localChanges: boolean;
    gen: number,
    editorState: BangleEditorState<unknown>;
}

export default class HtmlEditor extends React.Component<IProps, IState> {
    static contextType = StoresContext;
    declare context: IStores;

    private editor?: CoreBangleEditor;
    private debouncer: Debouncer;

    constructor(props: IProps) {
        super(props);

        let content = props.content;
        if (!content) {
            // Fix for https://github.com/bangle-io/bangle.dev/issues/231
            content = '<p></p>';
        }

        this.state = {
            content: content,
            gen: 0,
            localChanges: false,
            editorState: this.createEditorState(content),
        };

        this.debouncer = new Debouncer(300);
    }

    componentDidMount(): void {
        if (this.props.onBound) {
            this.props.onBound(this);
        }
    }

    componentDidUpdate(prevProps: IProps): void {
        if (!this.state.localChanges && this.props.content !== prevProps.content && this.props.content !== this.state.content) {
            this.setState({
                content: this.props.content,
                editorState: this.createEditorState(this.props.content),
                gen: this.state.gen + 1,
            });
        }
    }

    private createEditorState(content: string): BangleEditorState<unknown> {
        return new BangleEditorState({
            specs: [
                bold.spec(),
                italic.spec(),
                link.spec(),
                bulletList.spec(),
                orderedList.spec(),
                listItem.spec(),
                paragraph.spec(),
                heading.spec(),
                underline.spec(),
                strike.spec(),
                code.spec(),
                codeBlock.spec(),
                blockquote.spec(),
            ],
            plugins: () => [
                bold.plugins(),
                italic.plugins(),
                link.plugins(),
                bulletList.plugins(),
                orderedList.plugins(),
                listItem.plugins(),
                paragraph.plugins(),
                heading.plugins(),
                underline.plugins(),
                strike.plugins(),
                code.plugins(),
                codeBlock.plugins(),
                blockquote.plugins(),
                new Plugin({
                    view: () => ({
                        update: (view, prevState) => {
                            this.handleChange(view.state, prevState)
                        },
                    })
                }),
                keymap({
                    'Mod-k': () => {
                        this.toggleLink();
                        return true;
                    },
                })
            ],
            initialValue: content,
            editorProps: {
                handleDOMEvents: {
                    focus: () => {
                        if (this.props.onFocus) {
                            this.props.onFocus();
                        }
                        return false;
                    },
                    blur: () => {
                        if (this.props.onBlur) {
                            this.props.onBlur();
                        }
                        return false;
                    },
                    mousedown: (view, event) => {
                        // on mouse down to intercept editor's click prevention on links
                        return this.maybeHandleLinkClick(view, event);
                    },
                    dblclick: (view, event) => {
                        return this.maybeHandleLinkClick(view, event, true);
                    }
                }
            },
        })
    }

    render(): React.ReactNode {
        return <div className="html-editor">
            <BangleEditor
                key={this.state.gen}
                state={this.state.editorState}
                onReady={this.handleReady}
                focusOnInit={this.props.initialFocus ?? defaultInitialFocus}
            />
        </div>;
    }

    toggleInlineStyle(style: InlineStyle): void {
        switch (style) {
            case 'BOLD':
                toggleBold()(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'ITALIC':
                toggleItalic()(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'UNDERLINE':
                toggleUnderline()(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'CODE':
                toggleCode()(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'STRIKETHROUGH':
                toggleStrike()(this.editor.view.state, this.editor.view.dispatch);
                break;
        }
    }

    toggleBlockType(type: BlockStyle): void {
        const cursor = this.getCursor(this.editor.view.state);

        switch (type) {
            case 'header-one':
                toggleHeading(1)(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'header-two':
                toggleHeading(2)(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'header-three':
                toggleHeading(3)(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'header-four':
                toggleHeading(4)(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'unordered-list-item':
                toggleBulletList()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
                break;
            case 'ordered-list-item':
                toggleOrderedList()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
                break;
            case 'todo-list-item':
                toggleTodoList()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
                break;
            case 'blockquote':
                if (cursor.blockType != 'blockquote') {
                    wrapInBlockquote()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
                }
                break;
            case 'code-block':
                this.clearBlock();
                if (cursor.blockType != 'code-block') {
                    setBlockType(this.editor.view.state.schema.nodes['codeBlock'])(this.editor.view.state, this.editor.view.dispatch);
                }
                break;
        }
    }

    clearBlock(): void {
        const state = this.editor.view.state;
        const dispatch = this.editor.view.dispatch;
        setBlockType(state.schema.nodes.paragraph)(state, dispatch);
    }

    clearLink(): void {
        updateLink(null)(this.editor.view.state, this.editor.view.dispatch);
    }

    toggleLink(url: string | null = null): void {
        if (url) {
            createLink(url)(this.editor.view.state, this.editor.view.dispatch);
            return;
        }

        const done = (url?: string) => {
            this.context.session.hideModal();

            if (!url) {
                return;
            }

            if (!url.includes("://")) {
                url = 'entity://' + url;
            }

            if (url) {
                createLink(url)(this.editor.view.state, this.editor.view.dispatch);
            } else {
                this.clearLink();
            }
        };

        const cursor = this.getCursor();
        this.context.session.showModal(() => {
            return <InputModal text='Enter link' initialValue={cursor.link} onDone={done} />;
        })
    }

    toggleHeader(): void {
        const cursor = this.getCursor();
        if (cursor.blockType == 'header-one') {
            this.toggleBlockType('header-two');
        } else if (cursor.blockType == 'header-two') {
            this.toggleBlockType('header-three');
        } else if (cursor.blockType == 'header-three') {
            this.toggleBlockType('header-four');
        } else if (cursor.blockType == 'header-four') {
            // toggling with same block type will remove it 
            this.toggleBlockType('header-four');
        } else {
            this.toggleBlockType('header-one');
        }
    }

    indent(): void {
        indentListItem()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
    }

    outdent(): void {
        outdentListItem()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
    }

    private handleReady = (editor: CoreBangleEditor) => {
        this.editor = editor;

        if (this.props.onBound) {
            this.props.onBound(this);
        }

        if (this.props.onCursorChange && editor.view.hasFocus()) {
            this.props.onCursorChange(this.getCursor());
        }
    }

    private handleChange = (newState: EditorState, prevState: EditorState) => {
        if (!newState.doc.eq(prevState.doc)) {
            if (!this.state.localChanges) {
                this.setState({
                    localChanges: true,
                });
            }

            if (this.props.onChange) {
                this.debouncer.debounce(() => {
                    this.props.onChange(toHTMLString(newState));
                });
            }
        }

        if (this.props.onCursorChange) {
            this.props.onCursorChange(this.getCursor(newState));
        }
    }

    private maybeHandleLinkClick = (view: EditorView, event: MouseEvent, double = false): boolean => {
        if (view.hasFocus() && !event.metaKey && !double) {
            // we don't allow link click, unless it's a with double click or with meta key
            // or that it's the first click on the editor (we don't have focus)
            return false;
        }

        let el = event.target as HTMLElement;

        // if tagname is not a link, try to go up into the parenthood up 10 levels
        for (let i = 0; el.tagName !== 'A' && i < 10; i++) {
            if (el.parentNode) {
                el = el.parentNode as HTMLElement;
            }
        }

        if (el.tagName !== 'A') {
            return false;
        }

        // if it's a link, we prevent cursor from changing by stopping propagation here
        const link = el.getAttribute('href');
        if (link && this.props.onLinkClick) {
            this.props.onLinkClick(link, event);
            return true;
        }
    }

    private getCursor(state: EditorState = null): EditorCursor {
        if (!state) {
            state = this.editor.view.state;
        }

        const inlineStyle: Set<InlineStyle> = new Set();
        let blockType: BlockStyle = null;

        if (queryIsBoldActive()(state)) {
            inlineStyle.add('BOLD');
        }
        if (queryIsItalicActive()(state)) {
            inlineStyle.add('ITALIC');
        }
        if (queryIsUnderlineActive()(state)) {
            inlineStyle.add('UNDERLINE');
        }
        if (queryIsStrikeActive()(state)) {
            inlineStyle.add('STRIKETHROUGH');
        }
        if (queryIsCodeActive()(state)) {
            inlineStyle.add('CODE');
        }

        if (queryIsHeadingActive(1)(state)) {
            blockType = 'header-one';
        } else if (queryIsHeadingActive(2)(state)) {
            blockType = 'header-two';
        } else if (queryIsHeadingActive(3)(state)) {
            blockType = 'header-three';
        } else if (queryIsHeadingActive(4)(state)) {
            blockType = 'header-four';
        } else if (queryIsBulletListActive()(state)) {
            blockType = 'unordered-list-item';
        } else if (queryIsOrderedListActive()(state)) {
            blockType = 'ordered-list-item';
        } else if (queryIsTodoListActive()(state)) {
            blockType = 'todo-list-item';
        } else if (queryIsBlockquoteActive()(state)) {
            blockType = 'blockquote';
        } else if (queryIsCodeActiveBlock()(state)) {
            blockType = 'code-block';
        }


        let link;
        const attrs = queryLinkAttrs()(this.editor.view.state);
        if (attrs) {
            link = attrs.href;
        }

        return {
            blockType,
            inlineStyle,
            link,
            rect: this.getCursorRect(),
        }
    }

    // From https://github.com/bangle-io/bangle.dev/blob/b58ae1e6fd1e0b5577af04c8a74ea44e3944ad40/components/tooltip/selection-tooltip.ts#L167
    private getCursorRect(): CursorRect {
        const view = this.editor.view;
        const { selection } = view.state;
        const { head, from } = selection;

        // since head is dependent on the users choice of direction,
        // it is not always equal to `from`.
        // For textSelections we want to show the tooltip at head of the
        // selection.
        // But for NodeSelection we always want `from` since, if we go with `head`
        // coordsAtPos(head) might get the position `to` in head, resulting in
        // incorrectly getting position of the node after the selected Node.
        const pos = selection instanceof NodeSelection ? from : head;

        const start = view.coordsAtPos(pos);
        const { top, bottom, left, right } = start;
        const height = bottom - top;

        // Not sure why, but coordsAtPos does not return the correct
        // width of the element, so doing this to override it.
        let width = right - left;
        if (selection instanceof NodeSelection) {
            const domNode = view.nodeDOM(pos) as HTMLElement;
            width = domNode ? domNode.clientWidth : width;
        }

        return {
            left, width, right, top, bottom, height,
        }
    }
}

export type InlineStyle = 'BOLD' | 'ITALIC' | 'UNDERLINE' | 'STRIKETHROUGH' | 'CODE';
export type BlockStyle = 'header-one' | 'header-two' | 'header-three' | 'header-four' | 'unordered-list-item' | 'ordered-list-item' | 'todo-list-item' | 'blockquote' | 'code-block';

export interface EditorCursor {
    blockType: BlockStyle | null;
    inlineStyle: Set<InlineStyle>;
    link: string | null;
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
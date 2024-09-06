import React, { MouseEvent, UIEvent } from "react";
import { BangleEditor } from '@bangle.dev/react';
import { BangleEditorState, Plugin, BangleEditor as CoreBangleEditor } from '@bangle.dev/core';
import { bold, italic, link, bulletList, heading, listItem, orderedList, paragraph, underline, code, strike, codeBlock, blockquote, image } from '@bangle.dev/base-components';
import { safeInsert, toHTMLString } from '@bangle.dev/utils';
import { EditorState, EditorView, NodeSelection, Selection, setBlockType } from "@bangle.dev/pm";
import { keymap } from '@bangle.dev/pm';
import Debouncer from "../../../utils/debouncer";
import { createPopper, Instance } from '@popperjs/core';
import { CancellableEvent } from "../../../utils/events";
import { Shortcuts } from "../../../shortcuts";
import DragAndDrop from "../drag-and-drop/drag-and-drop";
import { EntityTraits, isEntityTraits } from "../../../utils/entities";

import './html-editor.less';
import '@bangle.dev/core/style.css';

const defaultInitialFocus = false;

interface IProps {
    content: string;
    onBound?: (editor: HtmlEditor) => void;
    onChange?: (content: string) => void;
    onFocus?: () => void;
    onBlur?: () => void;
    onCursorChange?: (cursor: EditorCursor) => void;
    onLinkClick?: (url: string, e: CancellableEvent) => void;
    linkSelector?: (cursor: EditorCursor) => Promise<SelectedLink | null>;
    initialFocus?: boolean;
    placeholder?: string;
}

export interface SelectedLink {
    url?: string;
    title?: string;
    canceled?: boolean;
}

interface IState {
    content?: string;
    localChanges: boolean;
    editorGen: number,
    editorState: BangleEditorState<unknown>;
    cursor?: EditorCursor;
    focused: boolean;
}

export default class HtmlEditor extends React.Component<IProps, IState> {
    private editor?: CoreBangleEditor;
    private debouncer: Debouncer;
    private popperRef: React.RefObject<HTMLDivElement> = React.createRef();
    private popper?: Instance;
    private popperElement?: unknown;

    private focusOverride = false;
    private inhibitIncomingUntil: Date | null = null;

    constructor(props: IProps) {
        super(props);

        let content = props.content;
        if (!content) {
            // Fix for https://github.com/bangle-io/bangle.dev/issues/231
            content = '<p></p>';
        }

        this.state = {
            content: content,
            editorGen: 0,
            localChanges: false,
            editorState: this.createEditorState(content),
            focused: false,
        };

        this.debouncer = new Debouncer(500);
    }

    componentDidMount(): void {
        if (this.props.onBound) {
            this.props.onBound(this);
        }
    }

    componentDidUpdate(prevProps: IProps): void {
        if (!this.hasFocus && this.props.content !== prevProps.content && this.props.content !== this.state.content) {
            this.setState({
                content: this.props.content,
                editorState: this.createEditorState(this.props.content),
                editorGen: this.state.editorGen + 1,
                localChanges: false,
            });
        }
    }

    componentWillUnmount(): void {
        this.editor = null;
        this.popper?.destroy();
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
                image.spec(),
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
                image.plugins(),
                new Plugin({
                    view: () => ({
                        update: (view) => {
                            this.handleChange(view.state);
                        },
                    })
                }),
                keymap({
                    'Mod-k': () => {
                        this.toggleLink();
                        return true;
                    },
                    'Ctrl-Enter': () => {
                        return this.tryOpenActiveLink(null, null);
                    },
                    'Escape': () => {
                        this.blur();
                        return true;
                    },
                })
            ],
            initialValue: content,
            editorProps: {
                handleDOMEvents: {
                    focus: () => {
                        if (this.props.onFocus) {
                            setTimeout(() => {
                                this.props.onFocus();
                            });
                        }

                        Shortcuts.activateContext('text-editor');

                        this.setState({ focused: true });

                        return false;
                    },
                    blur: () => {
                        if (this.props.onBlur) {
                            setTimeout(() => {
                                this.props.onBlur();
                            });
                        }

                        Shortcuts.deactivateContext('text-editor');

                        this.setState({ focused: true });

                        return false;
                    },
                    mousedown: (view, event) => {
                        // on mouse down to intercept editor's click prevention on links
                        return this.maybeHandleLinkClick(view, event as unknown as MouseEvent);
                    },
                    dblclick: (view, event) => {
                        return this.maybeHandleLinkClick(view, event as unknown as MouseEvent, true);
                    },
                    drop: (view, event) => {
                        const data = DragAndDrop.getDraggedData(event);
                        if (!data || !data.object || !isEntityTraits(data.object)) {
                            return false;
                        }

                        const coordinates = view.posAtCoords({
                            left: event.clientX,
                            top: event.clientY,
                        });

                        const et = data.object as EntityTraits;
                        this.toggleLink(`entity://${et.id}`, et.priorityTrait?.displayName, coordinates?.pos);

                        return true;
                    }
                }
            },
        });
    }

    render(): React.ReactNode {
        setTimeout(() => {
            this.maybeCreateLinkPopper();
        });

        return <div className="html-editor">
            {!this.hasFocus && this.isEmpty && this.props.placeholder &&
                <div className="placeholder">{this.props.placeholder}</div>
            }

            <BangleEditor
                key={this.state.editorGen}
                state={this.state.editorState}
                onReady={this.handleReady}
                focusOnInit={this.props.initialFocus ?? defaultInitialFocus}
            />

            {this.state.cursor?.link && this.hasFocus &&
                <div className="link-popper" ref={this.popperRef}>
                    <ul>
                        <li><a href="#" onMouseDown={this.handlePopperLinkEdit} onClick={this.handlePopperPreventClick}><span className="edit" /></a></li>
                        <li><a href="#" onMouseDown={this.handlePopperLinkOpen} onClick={this.handlePopperPreventClick}><span className="open" /></a></li>
                        <li><a href="#" onMouseDown={this.handlePopperLinkRemove} onClick={this.handlePopperPreventClick}><span className="remove" /></a></li>
                    </ul>
                </div>
            }
        </div>;
    }

    toggleInlineStyle(style: InlineStyle): void {
        switch (style) {
            case 'BOLD':
                bold.commands.toggleBold()(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'ITALIC':
                italic.commands.toggleItalic()(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'UNDERLINE':
                underline.commands.toggleUnderline()(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'CODE':
                code.commands.toggleCode()(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'STRIKETHROUGH':
                strike.commands.toggleStrike()(this.editor.view.state, this.editor.view.dispatch);
                break;
        }
    }

    toggleBlockType(type: BlockStyle): void {
        const cursor = this.getCursor(this.editor.view.state);

        switch (type) {
            case 'header-one':
                heading.commands.toggleHeading(1)(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'header-two':
                heading.commands.toggleHeading(2)(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'header-three':
                heading.commands.toggleHeading(3)(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'header-four':
                heading.commands.toggleHeading(4)(this.editor.view.state, this.editor.view.dispatch);
                break;
            case 'unordered-list-item':
                bulletList.commands.toggleBulletList()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
                break;
            case 'ordered-list-item':
                orderedList.commands.toggleOrderedList()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
                break;
            case 'todo-list-item':
                bulletList.toggleTodoList()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
                break;
            case 'blockquote':
                if (cursor.blockType != 'blockquote') {
                    blockquote.commands.wrapInBlockquote()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
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
        link.commands.updateLink(null)(this.editor.view.state, this.editor.view.dispatch);
    }

    async toggleLink(url: string | null = null, title: string | null = null, pos: number | null = null): Promise<void> {
        const cursor = this.getCursor();

        if (url) {
            const state = this.editor.view.state;
            const dispatch = this.editor.view.dispatch;
            if (cursor.selection.empty) {
                if (!title) {
                    title = url;
                }

                const linkMark = state.schema.marks.link.create({
                    href: url,
                });
                const linkNode = state.schema.text(title).mark([linkMark]);

                if (pos !== null && pos !== undefined) {
                    dispatch(safeInsert(linkNode, pos)(state.tr));
                } else {
                    dispatch(state.tr.replaceSelectionWith(linkNode, false));
                }
            } else {
                link.commands.createLink(url)(state, dispatch);
            }

            return;
        }

        if (this.props.linkSelector) {
            this.focusOverride = true; // override focus to prevent state change while we're selecting link
            const selectedLink = await this.props.linkSelector(cursor);
            this.focusOverride = false;

            if (selectedLink) {
                if (selectedLink.canceled) {
                    this.focus();
                    return;
                }

                this.toggleLink(selectedLink.url, selectedLink.title);
            } else {
                this.clearLink();
            }

            setTimeout(() => {
                // make sure focus comes back to editor since we may have asked for a link selection
                // in another stack since the editor seems to clear the selection otherwise
                this.editor.view.focus();
            });
        }
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
        listItem.commands.indentListItem()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
    }

    outdent(): void {
        listItem.commands.outdentListItem()(this.editor.view.state, this.editor.view.dispatch, this.editor.view);
    }

    focus(): void {
        this.editor?.view?.focus();
    }

    blur(): void {
        if (this.hasFocus) {
            const el = document.activeElement as HTMLElement;
            el.blur();
        }
    }

    get hasFocus(): boolean {
        return this.focusOverride || (this.editor?.view.hasFocus() ?? false);
    }

    get isEmpty(): boolean {
        const trimmed = this.state.content.trim();
        return trimmed == '' || trimmed == '<p></p>';
    }

    private handleReady = (editor: CoreBangleEditor) => {
        this.editor = editor;

        if (this.props.onBound) {
            this.props.onBound(this);
        }

        if (this.props.onCursorChange && editor.view.hasFocus()) {
            this.props.onCursorChange(this.getCursor());
        }
    };

    private handleChange = (newState: EditorState) => {
        this.debouncer.debounce(() => {
            const newHtmlContent = toHTMLString(newState);
            if (this.state.content != newHtmlContent) {
                this.setState({
                    content: newHtmlContent,
                    localChanges: true,
                });

                if (this.props.onChange) {
                    this.props.onChange(newHtmlContent);
                }
            }

            if (this.editor) {
                const cursor = this.getCursor(newState);
                if (this.props.onCursorChange) {
                    this.props.onCursorChange(cursor);
                }

                this.setState({ cursor });
            }
        });
    };

    private maybeHandleLinkClick = (view: EditorView, event: MouseEvent, double = false): boolean => {
        if (view.hasFocus() && !event.metaKey && !double) {
            // we don't allow link click, unless it's a with double click or with meta key
            // or that it's the first click on the editor (we don't have focus)
            return false;
        }

        return this.tryOpenActiveLink(event.target as HTMLElement, event);
    };

    private tryOpenActiveLink(target: HTMLElement | null, event: CancellableEvent | null): boolean {
        if (!target) {
            const cursor = this.getCursor();
            target = cursor.domNode;
        }

        if (!target) {
            return false;
        }

        // if tagname is not a link, try to go up into the parenthood up 10 levels
        let el = target;
        for (let i = 0; el.tagName !== 'A' && i < 10; i++) {
            if (el.parentNode) {
                el = el.parentNode as HTMLElement;
            }
        }

        if (el.tagName !== 'A') {
            return false;
        }

        if (!event) {
            event = {
                preventDefault: () => { /*nop*/ },
                stopPropagation: () => { /*nop*/ },
            };
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

        if (bold.commands.queryIsBoldActive()(state)) {
            inlineStyle.add('BOLD');
        }
        if (italic.commands.queryIsItalicActive()(state)) {
            inlineStyle.add('ITALIC');
        }
        if (underline.commands.queryIsUnderlineActive()(state)) {
            inlineStyle.add('UNDERLINE');
        }
        if (strike.commands.queryIsStrikeActive()(state)) {
            inlineStyle.add('STRIKETHROUGH');
        }
        if (code.commands.queryIsCodeActive()(state)) {
            inlineStyle.add('CODE');
        }

        if (heading.commands.queryIsHeadingActive(1)(state)) {
            blockType = 'header-one';
        } else if (heading.commands.queryIsHeadingActive(2)(state)) {
            blockType = 'header-two';
        } else if (heading.commands.queryIsHeadingActive(3)(state)) {
            blockType = 'header-three';
        } else if (heading.commands.queryIsHeadingActive(4)(state)) {
            blockType = 'header-four';
        } else if (bulletList.commands.queryIsBulletListActive()(state)) {
            blockType = 'unordered-list-item';
        } else if (orderedList.commands.queryIsOrderedListActive()(state)) {
            blockType = 'ordered-list-item';
        } else if (bulletList.queryIsTodoListActive()(state)) {
            blockType = 'todo-list-item';
        } else if (blockquote.commands.queryIsBlockquoteActive()(state)) {
            blockType = 'blockquote';
        } else if (codeBlock.commands.queryIsCodeActiveBlock()(state)) {
            blockType = 'code-block';
        }


        let linkEl;
        const attrs = link.commands.queryLinkAttrs()(this.editor.view.state);
        if (attrs) {
            linkEl = attrs.href;
        }

        const rect = this.getCursorRect();
        return {
            blockType,
            inlineStyle,
            link: linkEl,
            rect,
            domNode: rect.domNode,
            selection: rect.selection,
        };
    }

    private handlePopperLinkEdit = (e: UIEvent) => {
        this.toggleLink();
        e.stopPropagation();
        e.preventDefault();
        return false;
    };

    private handlePopperLinkOpen = (e: UIEvent) => {
        const cursor = this.getCursor();
        if (cursor.link) {
            this.props.onLinkClick(cursor.link, e);
        }
        e.stopPropagation();
        e.preventDefault();
        return false;
    };

    private handlePopperLinkRemove = (e: UIEvent) => {
        this.clearLink();
        e.stopPropagation();
        e.preventDefault();
        return false;
    };

    private handlePopperPreventClick = (e: MouseEvent) => {
        // prevent onClick since we bind on mouse down
        e.stopPropagation();
        e.preventDefault();
        return false;
    };

    // From https://github.com/bangle-io/bangle.dev/blob/b58ae1e6fd1e0b5577af04c8a74ea44e3944ad40/components/tooltip/selection-tooltip.ts#L167
    private getCursorRect(): CursorRect {
        const view = this.editor.view;
        const { selection } = view.state;

        // since head is dependent on the users choice of direction,
        // it is not always equal to `from`.
        // For textSelections we want to show the tooltip at head of the
        // selection.
        // But for NodeSelection we always want `from` since, if we go with `head`
        // coordsAtPos(head) might get the position `to` in head, resulting in
        // incorrectly getting position of the node after the selected Node.
        const pos = selection instanceof NodeSelection ? selection.from : selection.head;

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

        const domNode = view.domAtPos(pos, 1).node as HTMLElement;
        return {
            left, width, right, top, bottom, height, selection, domNode,
        };
    }

    private maybeCreateLinkPopper(): void {
        if (this.popper && !this.state.cursor?.link) {
            this.popper?.destroy();
            this.popper = null;
            this.popperElement = null;
            return;
        }

        if (!this.popperRef.current || !this.state.cursor?.link || !this.state.cursor.domNode) {
            return;
        }

        if (this.popperElement == this.state.cursor.domNode) {
            return;
        }

        this.popper = createPopper(this.state.cursor.domNode.parentElement, this.popperRef.current, {
            placement: 'top',
        });
        this.popperElement = this.state.cursor.domNode;
    }
}

export type InlineStyle = 'BOLD' | 'ITALIC' | 'UNDERLINE' | 'STRIKETHROUGH' | 'CODE';
export type BlockStyle = 'header-one' | 'header-two' | 'header-three' | 'header-four' | 'unordered-list-item' | 'ordered-list-item' | 'todo-list-item' | 'blockquote' | 'code-block';

export interface EditorCursor {
    blockType: BlockStyle | null;
    inlineStyle: Set<InlineStyle>;
    link: string | null;
    rect: CursorRect;
    selection: Selection;
    domNode: HTMLElement;
}

export interface CursorRect {
    left: number;
    width: number;
    right: number;
    top: number;
    bottom: number;
    height: number;
    selection: Selection;
    domNode: HTMLElement;
}
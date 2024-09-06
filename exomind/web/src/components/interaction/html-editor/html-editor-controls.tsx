
import classNames from 'classnames';
import React, { MouseEvent } from 'react';
import HtmlEditor, { BlockStyle, EditorCursor as EditorCursor, InlineStyle } from './html-editor';
import './html-editor-controls.less';

interface IProps {
    editor: HtmlEditor;
    cursor?: EditorCursor;
}

export default class HtmlEditorControls extends React.Component<IProps> {
    constructor(props: IProps) {
        super(props);
    }

    render(): React.ReactNode {
        // we use onMouseDown instead of onClick to prevent the editor from losing focus
        return <div className="html-editor-controls">
            <ul>
                {this.renderInlineStyleControls()}
                {this.renderLinkControl()}
                {this.renderBlockControls()}
                <li onMouseDown={(e) => this.handleOutdent('outdent', e)}><i className="icon outdent" /></li>
                <li onMouseDown={(e) => this.handleIndent('indent', e)}><i className="icon indent" /></li>
            </ul>
        </div>;
    }

    private renderInlineStyleControls(): React.ReactNode {
        const styles: { cssStyle: string, editorStyle: InlineStyle }[] = [
            {
                cssStyle: 'bold',
                editorStyle: 'BOLD',
            },
            {
                cssStyle: 'italic',
                editorStyle: 'ITALIC',
            },
            {
                cssStyle: 'underline',
                editorStyle: 'UNDERLINE',
            },
            {
                cssStyle: 'strikethrough',
                editorStyle: 'STRIKETHROUGH',
            },
            {
                cssStyle: 'code',
                editorStyle: 'CODE',
            },
        ];

        return styles.map(({ cssStyle, editorStyle }) => {
            const iconClasses = classNames({
                icon: true,
                [cssStyle]: true,
            });

            let active = false;
            if (this.props.cursor && this.props.cursor.inlineStyle.has(editorStyle)) {
                active = true;
            }

            const liClasses = classNames({ active });
            return (
                <li key={cssStyle}
                    className={liClasses}
                    onMouseDown={(e) => this.handleToggleInlineStyle(editorStyle, e)}>
                    <i className={iconClasses} />
                </li>
            );
        });
    }

    private renderLinkControl(): React.ReactNode {
        const iconClasses = classNames({
            icon: true,
            link: true,
        });

        let active = false;
        if (this.props.cursor && !!this.props.cursor.link) {
            active = true;
        }

        const liClasses = classNames({ active });

        return (
            <li className={liClasses} onMouseDown={(e) => this.handleLink(e)}><i className={iconClasses} /></li>
        );
    }

    private renderBlockControls(): React.ReactNode {
        const types = [
            {
                cssStyle: 'header-one',
                draftType: 'header-one',
            },
            {
                cssStyle: 'header-two',
                draftType: 'header-two',
            },
            {
                cssStyle: 'header-three',
                draftType: 'header-three',
            },
            {
                cssStyle: 'header-four',
                draftType: 'header-four',
            },
            {
                cssStyle: 'list-ul',
                draftType: 'unordered-list-item',
            },
            {
                cssStyle: 'list-ol',
                draftType: 'ordered-list-item',
            },
            {
                cssStyle: 'list-todo',
                draftType: 'todo-list-item',
            },
            {
                cssStyle: 'quote-right',
                draftType: 'blockquote',
            },
            {
                cssStyle: 'code',
                draftType: 'code-block',
            },
        ];

        return types.map(({ cssStyle, draftType }) => {
            const iconClasses = classNames({
                icon: true,
                [cssStyle]: true,
            });

            const liClasses = classNames({
                active: this.props.cursor && this.props.cursor.blockType == draftType,
            });
            return (
                <li key={cssStyle} className={liClasses} onMouseDown={this.handleToggleBlockType.bind(this, draftType)}><i className={iconClasses} /></li>
            );
        });
    }

    private handleToggleInlineStyle(format: InlineStyle, e: MouseEvent): void {
        e.preventDefault(); // prevent losing focus on editor
        if (this.props.editor) {
            this.props.editor.toggleInlineStyle(format);
        }
    }

    private handleToggleBlockType(format: BlockStyle, e: MouseEvent): void {
        e.preventDefault(); // prevent losing focus on editor
        if (this.props.editor) {
            this.props.editor.toggleBlockType(format);
        }
    }

    private handleLink(e: MouseEvent): void {
        e.preventDefault(); // prevent losing focus on editor
        if (this.props.editor) {
            this.props.editor.toggleLink();
        }
    }

    private handleIndent(format: string, e: MouseEvent): void {
        e.preventDefault(); // prevent losing focus on editor
        if (this.props.editor) {
            this.props.editor.indent();
        }
    }

    private handleOutdent(format: string, e: MouseEvent): void {
        e.preventDefault(); // prevent losing focus on editor
        if (this.props.editor) {
            this.props.editor.outdent();
        }
    }
}

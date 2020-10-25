
import classNames from 'classnames';
import React, { MouseEvent } from 'react';
import HtmlEditor, { EditorCursor as EditorCursor } from './html-editor';
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
    // we use onMouseDown instead of onClick to prevent DraftJS from losing focus
    // see https://github.com/facebook/draft-js/issues/696
    return <div className="html-editor-controls">
      <ul>
        {this.renderInlineStyleControls()}
        {this.renderBlockControls()}
        <li onMouseDown={this.handleOutdent.bind(this, 'outdent')}><i className="icon outdent" /></li>
        <li onMouseDown={this.handleIndent.bind(this, 'indent')}><i className="icon indent" /></li> 
      </ul>
    </div>;
  }

  private renderInlineStyleControls(): React.ReactFragment {
    const styles = [
      {
        cssStyle: 'bold',
        draftStyle: 'BOLD',
      },
      {
        cssStyle: 'italic',
        draftStyle: 'ITALIC',
      },
      {
        cssStyle: 'underline',
        draftStyle: 'UNDERLINE',
      },
      {
        cssStyle: 'strikethrough',
        draftStyle: 'STRIKETHROUGH',
      },
      {
        cssStyle: 'code',
        draftStyle: 'CODE',
      },
    ];

    return styles.map(({ cssStyle, draftStyle }) => {
      const iconClasses = classNames({
        icon: true,
        [cssStyle]: true,
      });

      let active = false;
      if (this.props.cursor && this.props.cursor.inlineStyle.has(draftStyle)) {
        active = true;
      }

      const liClasses = classNames({ active });
      return (
        <li key={cssStyle} className={liClasses} onMouseDown={this.handleToggleInlineStyle.bind(this, draftStyle)}><i className={iconClasses} /></li>
      );
    });
  }

  private renderBlockControls(): React.ReactFragment {
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

  private handleToggleInlineStyle(format: string, e: MouseEvent): void {
    e.preventDefault(); // prevent losing focus on editor
    if (this.props.editor) {
      this.props.editor.toggleInlineStyle(format);
    }
  }

  private handleToggleBlockType(format: string, e: MouseEvent): void {
    e.preventDefault(); // prevent losing focus on editor
    if (this.props.editor) {
      this.props.editor.toggleBlockType(format);
    }
  }

  private handleIndent(format: string, e: MouseEvent): void {
    e.preventDefault(); // prevent losing focus on editor
    if (this.props.editor) {
      this.props.editor.indent(e);
    }
  }

  private handleOutdent(format: string, e: MouseEvent): void {
    e.preventDefault(); // prevent losing focus on editor
    if (this.props.editor) {
      this.props.editor.outdent(e);
    }
  }
}

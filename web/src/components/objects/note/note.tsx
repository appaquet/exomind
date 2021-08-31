

import { Exocore, MutationBuilder } from 'exocore';
import { exomind } from '../../../protos';
import _ from 'lodash';
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import EditableText from '../../interaction/editable-text/editable-text';
import HtmlEditorControls from '../../interaction/html-editor/html-editor-controls';
import HtmlEditor, { EditorCursor } from '../../interaction/html-editor/html-editor';
import { SelectedItem, Selection } from '../entity-list/selection';
import './note.less';
import Navigation from '../../../navigation';

interface IProps {
    entity: EntityTraits;
    noteTrait: EntityTrait<exomind.base.v1.INote>;
    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
}

interface IState {
    savedNote: exomind.base.v1.INote;
    currentNote: exomind.base.v1.INote;
    focused: boolean;
    editor?: HtmlEditor;
    cursor?: EditorCursor;
}

export default class Note extends React.Component<IProps, IState> {
    private mounted = true;

    constructor(props: IProps) {
        super(props);

        this.state = {
            focused: false,
            savedNote: props.noteTrait.message,
            currentNote: new exomind.base.v1.Note(props.noteTrait.message),
        }
    }

    componentWillUnmount(): void {
        this.saveContent();
        this.mounted = false;
    }

    componentDidUpdate(): void {
        const note = new exomind.base.v1.Note(this.props.noteTrait.message);
        if (!this.state.focused && !_.isEqual(this.state.currentNote, note)) {
            this.setState({
                currentNote: note,
            });
        }
    }

    render(): React.ReactNode {
        return (
            <div className="entity-component note">
                <div className="entity-details">
                    <div className="title field"><span className="field-label">Title</span>
                        <span className="field-content">
                            <EditableText text={this.state.currentNote.title} onChange={this.handleTitleChange.bind(this)} />
                        </span>
                    </div>
                </div>

                <div className="object-body">
                    <HtmlEditorControls editor={this.state.editor} cursor={this.state.cursor} />
                    <HtmlEditor
                        content={this.state.currentNote.body}
                        placeholder="Type your note here"
                        onBound={this.handleContentBound.bind(this)}
                        onChange={this.handleContentChange.bind(this)}
                        onFocus={this.handleOnFocus.bind(this)}
                        onBlur={this.handleOnBlur.bind(this)}
                        onCursorChange={this.handleCursorChange.bind(this)}
                        onLinkClick={this.handleLinkClick.bind(this)}
                    />
                </div>
            </div>
        );
    }

    private handleOnFocus(): void {
        this.setState({
            focused: true,
        });
    }

    private handleOnBlur(): void {
        this.saveContent();
        this.setState({
            focused: false,
        });
    }

    private handleContentBound(editor: HtmlEditor): void {
        this.setState({
            editor: editor
        });
    }

    private handleTitleChange(newTitle: string): void {
        if (newTitle !== this.state.currentNote.title) {
            const note = this.state.currentNote;
            note.title = newTitle;

            this.setState({
                currentNote: note,
            });

            this.saveContent();
        }
    }

    private handleContentChange(newBody: string): void {
        if (newBody !== this.state.currentNote.body) {
            const note = this.state.currentNote;
            note.body = newBody;

            this.setState({
                currentNote: note,
            });

            // if after a second, it's still the same body, we save it (debouncing)
            setTimeout(() => {
                if (this.state.currentNote.body === newBody) {
                    this.saveContent();
                }
            }, 1000);
        }
    }

    private handleCursorChange(cursor: EditorCursor) {
        if (this.mounted) {
            this.setState({ cursor })
        }
    }

    private handleLinkClick(url: string, e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();

        if (url.startsWith('entity://')) {
            const entityId = url.replace('entity://', '');
            if (this.props.onSelectionChange) {
                this.props.onSelectionChange(new Selection(SelectedItem.fromEntityId(entityId)));
            }
        } else {
            Navigation.navigateExternal(url);
        }
    }

    private saveContent(): void {
        if (this.state && !_.isEqual(this.state.currentNote, this.state.savedNote)) {
            const mutation = MutationBuilder
                .updateEntity(this.props.entity.entity.id)
                .putTrait(this.state.currentNote, this.props.noteTrait.id)
                .build();

            Exocore.store.mutate(mutation);

            if (this.mounted) {
                this.setState({
                    savedNote: new exomind.base.v1.Note(this.state.currentNote),
                });
            }
        }
    }
}


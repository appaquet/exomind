

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
import InputModal from '../../modals/input-modal/input-modal';
import { IStores, StoresContext } from '../../../stores/stores';

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
    static contextType = StoresContext;
    declare context: IStores;

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
                    <div className="title field">
                        <span className="field-label">Title</span>
                        <span className="field-content">
                            <EditableText text={this.state.currentNote.title} onChange={this.handleTitleChange.bind(this)} />
                        </span>
                    </div>
                </div>

                <div className="object-body">
                    <HtmlEditorControls editor={this.state.editor} cursor={this.state.cursor} />
                    <HtmlEditor
                        content={this.state.currentNote.body}
                        onBound={this.handleContentBound.bind(this)}
                        onChange={this.handleContentChange.bind(this)}
                        onFocus={this.handleOnFocus.bind(this)}
                        onBlur={this.handleOnBlur.bind(this)}
                        onCursorChange={this.handleCursorChange.bind(this)}
                        onLinkClick={this.handleLinkClick}
                        linkSelector={this.linkSelector}
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

    private linkSelector = (url: string, cursor: EditorCursor): Promise<string | null> => {
        return new Promise((resolve) => {
            const done = (url: string | null, cancelled: boolean) => {
                this.context.session.hideModal();

                if (cancelled) {
                    return
                }

                if (!url) {
                    // clear the link
                    resolve(null);
                    return;
                }

                if (!url.includes("://")) {
                    url = 'entity://' + url;
                }
                resolve(url);
            };

            this.context.session.showModal(() => {
                return <InputModal text='Enter link' initialValue={cursor.link} onDone={done} />;
            })
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

            this.saveContent(note);
        }
    }

    private handleContentChange(newBody: string): void {
        if (newBody !== this.state.currentNote.body) {
            const note = this.state.currentNote;
            note.body = newBody;

            this.setState({
                currentNote: note,
            });

            this.saveContent(note);
        }
    }

    private handleCursorChange(cursor: EditorCursor) {
        if (this.mounted) {
            this.setState({ cursor })
        }
    }

    private handleLinkClick = (url: string, e: MouseEvent) => {
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

    private saveContent(note: exomind.base.v1.INote | null = null): void {
        if (!note) {
            note = this.state.currentNote;
        }

        if (this.state && !_.isEqual(note, this.state.savedNote)) {
            const mutation = MutationBuilder
                .updateEntity(this.props.entity.entity.id)
                .putTrait(note, this.props.noteTrait.id)
                .build();

            Exocore.store.mutate(mutation);

            if (this.mounted) {
                this.setState({
                    savedNote: new exomind.base.v1.Note(note),
                });
            }
        }
    }
}


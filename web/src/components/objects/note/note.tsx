
import { Exocore, MutationBuilder } from 'exocore';
import { exomind } from '../../../protos';
import _ from 'lodash';
import React from 'react';
import { EntityTrait, EntityTraits } from '../../../utils/entities';
import EditableText from '../../interaction/editable-text/editable-text';
import HtmlEditorControls from '../../interaction/html-editor/html-editor-controls';
import HtmlEditor, { EditorCursor, SelectedLink } from '../../interaction/html-editor/html-editor';
import { SelectedItem, Selection } from '../entity-list/selection';
import Navigation from '../../../navigation';
import { IStores, StoresContext } from '../../../stores/stores';
import LinkSelector from './link-selector';
import { CancellableEvent } from '../../../utils/events';
import { ListenerToken, Shortcuts } from '../../../shortcuts';
import { ContainerState } from '../container-state';
import './note.less';

interface IProps {
    entity: EntityTraits;
    noteTrait: EntityTrait<exomind.base.v1.INote>;
    selection?: Selection;
    onSelectionChange?: (sel: Selection) => void;
    containerState?: ContainerState,
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

    private shortcutToken?: ListenerToken;
    private mounted = false;

    constructor(props: IProps) {
        super(props);

        this.state = {
            focused: false,
            savedNote: props.noteTrait.message,
            currentNote: new exomind.base.v1.Note(props.noteTrait.message),
        }
    }

    componentDidMount(): void {
        this.mounted = true;
        this.shortcutToken = Shortcuts.register([
            {
                key: ['Enter', 'Space', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'],
                callback: this.handleShortcutFocus,
                disabledContexts: ['input', 'modal', 'text-editor'],
            },
        ]);
    }

    componentWillUnmount(): void {
        this.saveContent();
        this.mounted = false;
        Shortcuts.unregister(this.shortcutToken);
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
        if (this.props.containerState) {
            Shortcuts.setListenerEnabled(this.shortcutToken, this.props.containerState.active);

            if (!this.props.containerState.active && this.state.focused) {
                this.state.editor?.blur();
            }
        }

        return (
            <div className="entity-component note">
                <div className="entity-details">
                    <div className="title field">
                        <span className="field-label">Title</span>
                        <span className="field-content">
                            <EditableText text={this.state.currentNote.title} onChange={this.handleTitleChange} />
                        </span>
                    </div>
                </div>

                <div className="object-body">
                    <HtmlEditorControls editor={this.state.editor} cursor={this.state.cursor} />
                    <HtmlEditor
                        content={this.state.currentNote.body}
                        onBound={this.handleContentBound}
                        onChange={this.handleContentChange}
                        onFocus={this.handleOnFocus}
                        onBlur={this.handleOnBlur}
                        onCursorChange={this.handleCursorChange}
                        onLinkClick={this.handleLinkClick}
                        linkSelector={this.linkSelector}
                    />
                </div>
            </div>
        );
    }

    private handleOnFocus = (): void => {
        this.setState({
            focused: true,
        });
    }

    private handleOnBlur = (): void => {
        this.saveContent();
        this.setState({
            focused: false,
        });
    }

    private linkSelector = (cursor: EditorCursor): Promise<SelectedLink | null> => {
        return new Promise((resolve) => {
            const handleDone = (selectedLink: SelectedLink | null) => {
                this.context.session.hideModal();

                if (!selectedLink) {
                    // clear the link
                    resolve(null);
                    return;
                }

                if (!selectedLink.url.includes("://")) {
                    selectedLink.url = 'entity://' + selectedLink.url;
                }

                resolve(selectedLink);
            };

            const handleCancel = () => {
                this.context.session.hideModal(true);
                resolve({ canceled: true });
            };

            this.context.session.showModal(() => {
                return <LinkSelector initialValue={cursor.link} onDone={handleDone} onCancel={handleCancel} />;
            }, handleCancel);
        });
    }

    private handleContentBound = (editor: HtmlEditor): void => {
        this.setState({
            editor: editor
        });
    }

    private handleShortcutFocus = (): boolean => {
        this.state.editor?.focus();
        return true;
    }

    private handleTitleChange = (newTitle: string): void => {
        if (newTitle !== this.state.currentNote.title) {
            const note = this.state.currentNote;
            note.title = newTitle;

            this.setState({
                currentNote: note,
            });

            this.saveContent(note);
        }
    }

    private handleContentChange = (newBody: string): void => {
        if (newBody !== this.state.currentNote.body) {
            const note = this.state.currentNote;
            note.body = newBody;

            this.setState({
                currentNote: note,
            });

            this.saveContent(note);
        }
    }

    private handleCursorChange = (cursor: EditorCursor) => {
        if (this.mounted) {
            this.setState({ cursor })
        }
    }

    private handleLinkClick = (url: string, e: CancellableEvent) => {
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


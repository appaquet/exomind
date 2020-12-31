import React, { MouseEvent, ReactNode } from 'react';
import { ContentBlock, ContentState, EditorState, RichUtils } from "draft-js";
import HtmlEditor from "./html-editor";

interface LinkProps {
    entityKey: string;
    contentState: ContentState;
    children: ReactNode;
    editor: HtmlEditor;
}

export const Link = (props: LinkProps): ReactNode => {
    const { url } = props.contentState.getEntity(props.entityKey).getData();
    const handleClick = (e: MouseEvent) => {
        if (props.editor.props.onLinkClick) {
            props.editor.props.onLinkClick(url, e);
        }
    };
    return (
        <a href={url} onClick={handleClick} target="local">
            {props.children}
        </a>
    );
};

export function findLinkEntities(contentBlock: ContentBlock, callback: (start: number, end: number) => void, contentState: ContentState): void {
    contentBlock.findEntityRanges(
        (character) => {
            const entityKey = character.getEntity();
            return (
                entityKey !== null &&
                contentState.getEntity(entityKey).getType() === 'LINK'
            );
        },
        callback
    );
}

export function toggleLink(editorState: EditorState, url: string | null): EditorState | void {
    // TODO: If url is null, check remove link

    const contentState = editorState.getCurrentContent();

    const contentStateWithEntity = contentState.createEntity(
        'LINK',
        'MUTABLE',
        { url: url }
    );
    const entityKey = contentStateWithEntity.getLastCreatedEntityKey();

    let newEditorState = EditorState.set(editorState, { currentContent: contentStateWithEntity });
    newEditorState = RichUtils.toggleLink(
        newEditorState,
        newEditorState.getSelection(),
        entityKey
    );

    return newEditorState;
}

export function extractCurrentLink(editorState: EditorState): string | null {
    const curContent = editorState.getCurrentContent();
    const curSel = editorState.getSelection();

    const selStartKey = curSel.getStartKey();
    const selectedBlock = curContent.getBlockForKey(selStartKey);

    const linkKey = selectedBlock.getEntityAt(0);
    if (!linkKey) {
        return;
    }

    const linkInstance = curContent.getEntity(linkKey);
    const { url } = linkInstance.getData();

    return url;
}
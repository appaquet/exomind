import { ContentBlock, ContentState, EditorState, Modifier, SelectionState } from "draft-js"

export class Commands {
    static handleIndentText(editorState: EditorState): EditorState | void {
        const curContent = editorState.getCurrentContent();
        const curSel = editorState.getSelection();

        // if single focus selection (not spanning multiple blocks & same start/end focus), we simply insert 2 spaces
        if (curSel.getStartKey() == curSel.getEndKey() && curSel.getStartOffset() == curSel.getEndOffset()) {
            const newContentState = Modifier.replaceText(curContent, curSel, '  ');
            return EditorState.push(editorState, newContentState, 'insert-characters');
        }

        // otherwise, indent at beginning of each block
        let newContent = curContent;
        iterBlocks(curContent, curSel, (block) => {
            const newSel = SelectionState.createEmpty(block.getKey()).merge({
                anchorOffset: 0,
                focusOffset: 0,
            });

            newContent = Modifier.replaceText(
                newContent,
                newSel,
                '  '
            );
        });

        if (newContent != curContent) {
            const newState = EditorState.push(editorState, newContent, 'insert-characters');
            return EditorState.forceSelection(newState, curSel);
        }
    }

    static handleOutdentText(editorState: EditorState): EditorState | void {
        const curContent = editorState.getCurrentContent();
        const curSel = editorState.getSelection();

        // outdent blocks
        let newContent = curContent;
        iterBlocks(curContent, curSel, (block) => {
            const beginText = block.getText().slice(0, 2);
            const toDelete = Math.min(countInitSpaces(beginText), 2);

            if (toDelete == 0) {
                return;
            }

            const removeSel = SelectionState.createEmpty(block.getKey()).merge({
                anchorOffset: 0,
                focusOffset: toDelete,
            });

            newContent = Modifier.removeRange(newContent, removeSel, 'backward');
        });

        if (newContent != curContent) {
            const newState = EditorState.push(editorState, newContent, 'remove-range');
            return EditorState.forceSelection(newState, curSel);
        }
    }

    // Creates a new block after the current one without any style.
    // Used to create new line after header.
    static createUnstyledNextBlock(editorState: EditorState): EditorState {
        const currentState = editorState;
        const currentSelection = currentState.getSelection();
        const currentContent = currentState.getCurrentContent();

        let newContent = Modifier.splitBlock(currentContent, currentSelection);
        newContent = Modifier.setBlockType(newContent, newContent.getSelectionAfter(), 'unstyled');
        return EditorState.push(editorState, newContent, "split-block");
    }
}

function iterBlocks(content: ContentState, selection: SelectionState, f: (block: ContentBlock) => void) {
    let block = content.getBlockForKey(selection.getStartKey());
    for (; ;) {
        f(block);

        if (block.getKey() == selection.getEndKey()) {
            return;
        }

        const nextBlock = content.getBlockAfter(block.getKey());
        if (!nextBlock) {
            return;
        }

        block = nextBlock;
    }
}

function countInitSpaces(text: string): number {
    for (let i = 0; i < text.length; i++) {
        if (text[i] != ' ') {
            return i;
        }
    }

    return text.length;
}
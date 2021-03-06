import { MouseEvent } from "react";

export type ActionResult = 'remove' | void;

export class EntityActions {
    constructor(public buttons: ButtonAction[] = [], public inlineEdit: InlineAction | null = null) {
        this.buttons = buttons;
        this.inlineEdit = inlineEdit;
    }

    get isEmpty(): boolean {
        return !this.buttons || this.buttons.length == 0;
    }
}

export class ButtonAction {
    constructor(public icon: string, public callback: (action: ButtonAction, e: unknown) => ActionResult) {
        this.icon = icon;
        this.callback = callback;
    }

    trigger(e: MouseEvent): ActionResult {
        return this.callback(this, e);
    }
}

export class InlineAction {
    constructor(public callback: (action: InlineAction) => void) {
        this.callback = callback;
    }

    trigger(): void {
        return this.callback(this);
    }
}
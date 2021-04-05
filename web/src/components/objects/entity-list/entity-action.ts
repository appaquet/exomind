import { MouseEvent } from "react";

export type ActionResult = 'remove' | void;

export class EntityActions {
    constructor(public buttons: ButtonAction[] = [], public inlineEdit: InlineAction | null = null) {
    }

    get isEmpty(): boolean {
        return this.buttons.length == 0;
    }
}

export class ButtonAction {
    constructor(public icon: string, public callback: (action: ButtonAction, e: unknown) => ActionResult) {
    }

    trigger(e: MouseEvent): ActionResult {
        return this.callback(this, e);
    }
}

export class InlineAction {
    constructor(public callback: (action: InlineAction) => void) {
    }

    trigger(): void {
        return this.callback(this);
    }
}
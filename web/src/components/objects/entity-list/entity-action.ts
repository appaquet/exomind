import { MouseEvent } from "react";
import { IMenuItem } from "../../layout/menu";

export type ActionResult = 'remove' | void;

export class EntityActions {
    constructor(public buttons: ButtonAction[] = [], public inlineEdit: InlineAction | null = null) {
        this.buttons = buttons;
        this.inlineEdit = inlineEdit;
    }

    get isEmpty(): boolean {
        return !this.buttons || this.buttons.length == 0;
    }

    toMenuItems(): IMenuItem[] {
        return this.buttons.map(b => {
            return {
                label: b.label,
                icon: b.icon,
                onClick: (e: MouseEvent) => {
                    return b.trigger(e);
                }
            }
        });
    }
}

export class ButtonAction {
    constructor(public label: string, public icon: string, public callback: (action: ButtonAction, e: MouseEvent) => ActionResult) {
        this.label = label;
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
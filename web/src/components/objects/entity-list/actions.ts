import { MouseEvent } from "react";
import { IAction } from "../../../utils/actions";
import { IMenuItem } from "../../layout/menu";

export type ActionResult = 'remove' | void;

export class ListEntityActions {
    constructor(public buttons: ButtonAction[] = [], public inlineAction: InlineAction | null = null) {
    }

    static fromActions(actions: IAction[]): ListEntityActions {
        const buttons = actions.map(a => {
            return new ButtonAction(a.label, a.icon, (_ba, e) => {
                // TODO: Handle async
                a.execute(e, a);
                return;
            });
        });
        return new ListEntityActions(buttons);
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
import { MouseEvent } from "react";
import { IAction } from "../../../utils/actions";
import { IMenuItem } from "../../layout/menu";

export type ActionResult = 'remove' | void;

export class ListEntityActions {
    constructor(public buttons: ListEntityAction[] = [], public inlineAction: InlineAction | null = null) {
    }

    static fromActions(actions: IAction[]): ListEntityActions {
        const buttons = actions.map(a => {
            return new ListEntityAction(a.label, a.icon, async (_ba, e) => {
                const res = await a.execute(e);
                if (res?.remove === true) {
                    return 'remove';
                }
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
                    b.trigger(e);
                }
            };
        });
    }
}

export class ListEntityAction {
    constructor(public label: string, public icon: string, public callback: (action: ListEntityAction, e: MouseEvent) => Promise<ActionResult>) {
    }

    async trigger(e: MouseEvent): Promise<ActionResult> {
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
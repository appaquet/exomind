
export default class EntityAction {
    public shouldRemove = false;

    constructor(public icon: string, public callback: (action: EntityAction, e: unknown) => void) {
    }

    trigger(e: MouseEvent): void {
        this.callback(this, e);
    }
}
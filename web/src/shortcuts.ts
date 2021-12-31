
export type Context =
    'text-editor' |
    'input' | // automatically inferred if focused on an input element
    'modal';
export type ListenerToken = number;

interface Mapping {
    key: string | string[];
    callback: (event: KeyboardEvent) => boolean;
    disabledContexts?: Context[];
    token?: ListenerToken;
    disabled?: boolean;
}

interface Listener {
    mappings: Mapping[];
    disabled: boolean;
}

export class Shortcuts {
    private static mappings: { [key: string]: Mapping[] } = {};
    private static listeners: { [token: ListenerToken]: Listener } = {};
    private static nextListener = 0;

    private static activeContexts: Set<Context> = new Set();

    private static lastKeyEvent?: KeyboardEvent;
    private static lastKeyTime?: Date;

    static get lastShortcutTime(): Date | null {
        return this.lastKeyTime;
    }

    static get usedRecently(): boolean {
        if (!this.lastShortcutTime) {
            return false;
        }

        return new Date().getTime() - this.lastShortcutTime.getTime() < 1000;
    }

    static register(mapping: Mapping | Mapping[]): ListenerToken {
        if (!Array.isArray(mapping)) {
            mapping = [mapping];
        }

        const token = this.nextListener++;
        for (const m of mapping) {
            m.token = token;
            if (!Array.isArray(m.key)) {
                m.key = [m.key];
            }

            for (const key of m.key) {
                if (!this.mappings[key]) {
                    this.mappings[key] = [];
                }
                this.mappings[key].push(m);
            }
        }
        this.listeners[token] = {
            mappings: mapping,
            disabled: false,
        };

        return token;
    }

    static unregister(token: ListenerToken): void {
        const listener = this.listeners[token];
        delete this.listeners[token];

        for (const mapping of listener.mappings) {
            for (const key of mapping.key) {
                const keyMappings = this.mappings[key];
                const index = keyMappings.indexOf(mapping);
                if (index >= 0) {
                    keyMappings.splice(index, 1);
                }
            }
        }
    }

    static activateContext(ctx: Context): void {
        this.activeContexts.add(ctx);
    }

    static deactivateContext(ctx: Context): void {
        this.activeContexts.delete(ctx);
    }

    static setListenerEnabled(token: ListenerToken, enabled: boolean): void {
        const listener = this.listeners[token];
        if (!listener || listener.disabled == !enabled) {
            return;
        }

        for (const mapping of listener.mappings) {
            mapping.disabled = !enabled;
        }
        listener.disabled = !enabled;
    }

    static _handleKeyDown(event: KeyboardEvent): void {
        if (event.key == 'Meta' || event.key == 'Alt' || event.key == 'Control') {
            return;
        }

        if (this.lastKeyEvent && this.lastKeyTime && new Date().getTime() - this.lastKeyTime.getTime() < 500) {
            if (this.checkKey(this.lastKeyEvent, event)) {
                this.lastKeyEvent = event;
                this.lastKeyTime = new Date();
                return;
            }
        }

        if (this.checkKey(event, null)) {
            this.lastKeyEvent = event;
            this.lastKeyTime = new Date();
            return;
        }

        this.lastKeyEvent = event;
        this.lastKeyTime = new Date();
    }

    private static checkKey(firstEvent: KeyboardEvent, secondEvent: KeyboardEvent | null): boolean {
        let postfix = '';
        if (secondEvent) {
            postfix += ` ${this.remapKey(secondEvent.key)}`;
        }

        if (firstEvent.metaKey) {
            if (this._triggerKey(`Mod-${this.remapKey(firstEvent.key)}${postfix}`, secondEvent || firstEvent)) {
                return true;
            }
        }

        if (firstEvent.ctrlKey) {
            if (this._triggerKey(`Ctrl-${this.remapKey(firstEvent.key)}${postfix}`, secondEvent || firstEvent)) {
                return true;
            }

            if (this._triggerKey(`Mod-${this.remapKey(firstEvent.key)}${postfix}`, secondEvent || firstEvent)) {
                return true;
            }
        }

        if (this._triggerKey(`${this.remapKey(firstEvent.key)}${postfix}`, secondEvent || firstEvent)) {
            return true;
        }

        return false;
    }

    private static _triggerKey(key: string, event: KeyboardEvent): boolean {
        const keyMappings = this.mappings[key];
        if (!keyMappings) {
            return false;
        }

        for (const keyMapping of keyMappings) {
            if ((keyMapping.disabled || false) || this.anyActiveContexts(keyMapping.disabledContexts)) {
                continue
            }

            const handled = keyMapping.callback(event);
            if (!handled) {
                continue;
            }

            event.stopPropagation();
            event.preventDefault();
            return true;
        }

        return false;
    }

    private static anyActiveContexts(other: Context[] | null): boolean {
        const activeContexts = new Set(this.activeContexts);
        if (document.activeElement instanceof HTMLInputElement || document.activeElement instanceof HTMLTextAreaElement) {
            activeContexts.add('input');
        }

        if (!other || activeContexts.size == 0) {
            return false;
        }

        for (const ctx of other) {
            if (activeContexts.has(ctx)) {
                return true;
            }
        }

        return false;
    }

    private static remapKey(key: string): string {
        if (key == ' ') {
            return 'Space';
        }

        return key;
    }
}

document.addEventListener('keydown', (e: KeyboardEvent) => {
    Shortcuts._handleKeyDown(e);
});

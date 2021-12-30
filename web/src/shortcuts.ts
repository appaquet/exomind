
import Navigation from "./navigation";
import { Stores } from "./stores/stores";

export type Context = 'text-editor' | 'input';
export type ListenerToken = number;

interface Mapping {
    key: string | string[];
    callback: (event: KeyboardEvent) => boolean;
    noContext?: Context[];
    token?: ListenerToken;
}

export class Shortcuts {
    private static mappings: { [key: string]: Mapping[] } = {};
    private static listeners: { [token: ListenerToken]: Mapping[] } = {};
    private static nextListener = 0;

    private static activeContexts: Set<Context> = new Set();

    private static lastKey?: KeyboardEvent;
    private static lastKeyTime?: Date;

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
        this.listeners[token] = mapping;

        return token;
    }

    static unregister(token: ListenerToken): void {
        const listenerMapping = this.listeners[token];
        delete this.listeners[token];

        console.log('unregister', token);
        for (const mapping of listenerMapping) {
            for (const key of mapping.key) {
                const keyMappings = this.mappings[key];
                const index = keyMappings.indexOf(mapping);
                if (index >= 0) {
                    console.log('unregister mapping', token, index);
                    keyMappings.splice(index, 1);
                }
            }
        }
    }

    static activateContext(ctx: Context): void {
        console.log('activateContext', ctx);
        this.activeContexts.add(ctx);
    }

    static deactivateContext(ctx: Context): void {
        console.log('deactivateContext', ctx);
        this.activeContexts.delete(ctx);
    }

    static _handleKeyDown(event: KeyboardEvent): void {
        if (event.key == 'Meta' || event.key == 'Alt' || event.key == 'Control') {
            return;
        }

        console.log('event', event);

        if (this.lastKey && this.lastKeyTime && new Date().getTime() - this.lastKeyTime.getTime() < 1000) {
            if (this.checkKey(this.lastKey, event)) {
                return;
            }
        }

        if (this.checkKey(event, null)) {
            return;
        }

        this.lastKey = event;
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
        console.log('checking', key);
        const keyMappings = this.mappings[key];
        if (!keyMappings) {
            return false;
        }

        for (const keyMapping of keyMappings) {
            if (this.anyActiveContexts(keyMapping.noContext)) {
                continue
            }

            const handled = keyMapping.callback(event);
            if (!handled) {
                continue;
            }

            console.log(event.key, 'handled!')
            event.stopPropagation();
            event.preventDefault();
            return true;
        }

        return false;
    }

    private static anyActiveContexts(other: Context[] | null): boolean {
        if (!other || this.activeContexts.size == 0) {
            return false;
        }

        for (const ctx of other) {
            if (this.activeContexts.has(ctx)) {
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

Shortcuts.register([
    {
        key: 'Mod-e t',
        callback: () => {
            Stores.settings.toggleDarkMode();
            return true;
        },
        noContext: ['text-editor'],
    },
    {
        key: 'Mod-e i',
        callback: () => {
            Navigation.navigate(Navigation.pathForInbox())
            return true;
        },
        noContext: ['text-editor'],
    },
    {
        key: 'Mod-e z',
        callback: () => {
            Navigation.navigate(Navigation.pathForSnoozed())
            return true;
        },
        noContext: ['text-editor'],
    },
    {
        key: 'Mod-e r',
        callback: () => {
            Navigation.navigate(Navigation.pathForRecent())
            return true;
        },
        noContext: ['text-editor'],
    },
]);

import Navigation from "./navigation";
import { Stores } from "./stores/stores";

type Context = 'text-editor' | 'input';

interface Listener {
    key: string;
    callback: (event: KeyboardEvent) => void;
    noContext?: Context[];
}

export class Shortcuts {
    private static listeners: { [key: string]: Listener } = {};
    private static contexts: Set<Context> = new Set();

    private static lastKey?: KeyboardEvent;
    private static lastKeyTime?: Date;

    static addListener(listener: Listener | Listener[]): void {
        if (Array.isArray(listener)) {
            for (const l of listener) {
                this.addListener(l);
            }
        } else {
            this.listeners[listener.key] = listener;
        }
    }

    static removeListener(key: string): void {
        delete this.listeners[key];
    }

    static activateContext(ctx: Context): void {
        console.log('activateContext', ctx);
        this.contexts.add(ctx);
    }

    static deactivateContext(ctx: Context): void {
        console.log('deactivateContext', ctx);
        this.contexts.delete(ctx);
    }

    static _handleKeyDown(event: KeyboardEvent): void {
        if (event.key == 'Meta' || event.key == 'Alt' || event.key == 'Control') {
            return;
        }

        let firstEvent = event;
        let postfix = '';
        if (this.lastKey && this.lastKeyTime && new Date().getTime() - this.lastKeyTime.getTime() < 1000) {
            postfix = ` ${firstEvent.key}`;
            firstEvent = this.lastKey;
        }

        console.log('event', firstEvent);
        if (firstEvent.metaKey) {
            if (this._triggerKey(`Mod-${firstEvent.key}${postfix}`, firstEvent)) {
                return;
            }
        }

        if (firstEvent.ctrlKey) {
            if (this._triggerKey(`Ctl-${firstEvent.key}${postfix}`, firstEvent)) {
                return;
            }

            if (this._triggerKey(`Mod-${firstEvent.key}${postfix}`, firstEvent)) {
                return;
            }
        }

        if (this._triggerKey(`${firstEvent.key}${postfix}`, firstEvent)) {
            return;
        }

        this.lastKey = event;
        this.lastKeyTime = new Date();
    }

    static _triggerKey(key: string, event: KeyboardEvent): boolean {
        console.log('checking', key);
        const listener = this.listeners[key];
        if (!listener) {
            return false;
        }

        if (this.anyActiveContexts(listener.noContext)) {
            return false;
        }

        console.log('triggering', event.key);
        listener.callback(event);

        event.stopPropagation();
        event.preventDefault();

        return true;
    }

    private static anyActiveContexts(other: Context[] | null): boolean {
        if (!other || this.contexts.size == 0) {
            return false;
        }

        for (const ctx of other) {
            if (this.contexts.has(ctx)) {
                return true;
            }
        }

        return false;
    }
}

document.addEventListener('keydown', (e: KeyboardEvent) => {
    Shortcuts._handleKeyDown(e);
});

Shortcuts.addListener([
    {
        key: 'Mod-e t',
        callback: () => {
            Stores.settings.toggleDarkMode();
        },
        noContext: ['text-editor'],
    },
    {
        key: 'Mod-e i',
        callback: () => {
            Navigation.navigate(Navigation.pathForInbox())
        },
        noContext: ['text-editor'],
    },
    {
        key: 'Mod-e z',
        callback: () => {
            Navigation.navigate(Navigation.pathForSnoozed())
        },
        noContext: ['text-editor'],
    },
    {
        key: 'Mod-e r',
        callback: () => {
            Navigation.navigate(Navigation.pathForRecent())
        },
        noContext: ['text-editor'],
    },
]);
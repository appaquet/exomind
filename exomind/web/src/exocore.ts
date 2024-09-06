import { Exocore, ExocoreInstance, LocalNode, WasmModule } from 'exocore';
import { runInAction } from 'mobx';
import { exomind } from './protos';
import { Stores } from './stores/stores';

export async function initNode(): Promise<WasmModule> {
    const module = await Exocore.ensureLoaded();

    let node: LocalNode;
    try {
        node = Exocore.node.from_storage(localStorage);
    } catch (e) {
        console.error('Couldn\'t load node from storage', e);
    }

    if (!node) {
        node = Exocore.node.generate();
        node.save_to_storage(localStorage);
    }

    Stores.session.node = node;
    if (node.has_configured_cell) {
        bootNode();
    } else {
        runInAction(() => {
            Stores.session.showDiscovery = true;
        });
    }

    return module;
}

export async function resetNode(): Promise<void> {
    const node = Exocore.node.generate();
    node.save_to_storage(localStorage);

    restartNode();
}

export function restartNode(): void {
    const sessionStore = Stores.session;

    runInAction(() => {
        sessionStore.cellInitialized = false;
        sessionStore.cellError = null;
        sessionStore.showDiscovery = true;
    });

    Exocore.reset();
}

export async function bootNode(): Promise<Exocore | null> {
    const sessionStore = Stores.session;

    try {
        const instance = await Exocore.initialize(sessionStore.node);
        registerTypes(instance);

        runInAction(() => {
            sessionStore.cellInitialized = true;
            sessionStore.cellError = null;
            sessionStore.showDiscovery = false;
        });

        Stores.collections.fetchCollections();

        return instance;

    } catch (e) {
        console.error('Couldn\'t initialize exocore', e);
        runInAction(() => {
            sessionStore.cellInitialized = false;
            sessionStore.cellError = e;
        });
    }
}

export function registerTypes(instance?: ExocoreInstance): void {
    if (!instance) {
        instance = Exocore.default;
    }

    instance.registry.registerMessage(exomind.base.v1.EmailThread, 'exomind.base.v1.EmailThread');
    instance.registry.registerMessage(exomind.base.v1.Email, 'exomind.base.v1.Email');
    instance.registry.registerMessage(exomind.base.v1.EmailPart, 'exomind.base.v1.EmailPart');
    instance.registry.registerMessage(exomind.base.v1.DraftEmail, 'exomind.base.v1.DraftEmail');
    instance.registry.registerMessage(exomind.base.v1.Account, 'exomind.base.v1.Account');
    instance.registry.registerMessage(exomind.base.v1.Collection, 'exomind.base.v1.Collection');
    instance.registry.registerMessage(exomind.base.v1.CollectionChild, 'exomind.base.v1.CollectionChild');
    instance.registry.registerMessage(exomind.base.v1.Task, 'exomind.base.v1.Task');
    instance.registry.registerMessage(exomind.base.v1.Note, 'exomind.base.v1.Note');
    instance.registry.registerMessage(exomind.base.v1.Link, 'exomind.base.v1.Link');
    instance.registry.registerMessage(exomind.base.v1.Snoozed, 'exomind.base.v1.Snoozed');
    instance.registry.registerMessage(exomind.base.v1.Unread, 'exomind.base.v1.Unread');
}
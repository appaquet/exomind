import { Exocore, ExocoreInstance, LocalNode, WasmModule } from 'exocore';
import { runInAction } from 'mobx';
import { exomind } from './protos';
import { Stores } from './stores/stores';

export async function initNode(): Promise<WasmModule> {
    const module = await Exocore.ensureLoaded();

    let node: LocalNode;
    try {
        node = Exocore.node.from_storage(localStorage)
    } catch (e) {
        console.log('Couldn\'t load node from storage', e);
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
        })
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
        console.log('Couldn\'t initialize exocore', e);
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

    instance.registry.registerMessage(exomind.base.EmailThread, 'exomind.base.EmailThread');
    instance.registry.registerMessage(exomind.base.Email, 'exomind.base.Email');
    instance.registry.registerMessage(exomind.base.EmailPart, 'exomind.base.EmailPart');
    instance.registry.registerMessage(exomind.base.DraftEmail, 'exomind.base.DraftEmail');
    instance.registry.registerMessage(exomind.base.Account, 'exomind.base.Account');
    instance.registry.registerMessage(exomind.base.Collection, 'exomind.base.Collection');
    instance.registry.registerMessage(exomind.base.CollectionChild, 'exomind.base.CollectionChild');
    instance.registry.registerMessage(exomind.base.Task, 'exomind.base.Task');
    instance.registry.registerMessage(exomind.base.Note, 'exomind.base.Note');
    instance.registry.registerMessage(exomind.base.Link, 'exomind.base.Link');
    instance.registry.registerMessage(exomind.base.Snoozed, 'exomind.base.Snoozed');
}
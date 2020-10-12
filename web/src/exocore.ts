import { Exocore } from 'exocore';
import { exomind } from './protos';
import { autorun } from 'mobx';
import { StoresInstance } from './store/stores';

let currentConfig: Record<string, unknown> | null = null;
autorun(() => {
    if (StoresInstance.settings.exocoreConfig && currentConfig != StoresInstance.settings.exocoreConfig) {
        currentConfig = StoresInstance.settings.exocoreConfig;

        StoresInstance.session.exocoreInitialized = false;
        initialize(currentConfig)
            .then(() => {
                StoresInstance.session.exocoreInitialized = true;
            })
            .catch((err) => {
                console.log('Error loading exocore', err);
            });
    }
})

export async function ensureLoaded(): Promise<void> {
    return await Exocore.ensureLoaded();
}

async function initialize(config: Record<string, unknown>): Promise<void> {
    const instance = await Exocore.initialize(config);
    Exocore.defaultInstance = instance;

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

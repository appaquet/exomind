
import { observable, action, computed, autorun } from 'mobx';
import React from 'react';

export interface ISettingsStore {
    exocoreConfig?: Record<string, unknown>;
    test: boolean;
}

export class SettingsStore implements ISettingsStore {
    @observable exocoreConfig?: Record<string, unknown>;
    @observable test: boolean;

    constructor(syncLocalStorage?: boolean) {
        if (window.localStorage && (syncLocalStorage ?? true)) {
            this.setupLocalStorageSync();
        }
    }

    @computed get asJson(): ISettingsStore {
        return {
            exocoreConfig: this.exocoreConfig,
            test: this.test,
        }
    }

    @action updateFromJson(json: ISettingsStore): void {
        this.exocoreConfig = json.exocoreConfig;
        this.test = json.test;
    }

    private setupLocalStorageSync() {
        if (window.localStorage.settings) {
            try {
                this.updateFromJson(JSON.parse(window.localStorage.settings) as ISettingsStore);
            }
            catch (e) {
                console.log('Error parsing local storage app settings', e);
            }
        }

        autorun(() => {
            window.localStorage.settings = JSON.stringify(this.asJson);
        });
    }
}

export class SessionStore {
    @observable exocoreInitialized = false;
}

export class Stores {
    constructor(public settings: SettingsStore, public session: SessionStore) {
    }
}

export const StoresInstance: Stores = {
    settings: new SettingsStore(),
    session: new SessionStore(),
}

export const StoresContext = React.createContext<Stores | null>(null);

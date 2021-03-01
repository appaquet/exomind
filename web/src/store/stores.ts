import { LocalNode } from 'exocore';
import { observable, action, computed, autorun, makeAutoObservable } from 'mobx';
import React from 'react';
export interface ISettingsStore {
    darkMode: boolean;
}

export class PersistedStore implements ISettingsStore {
    @observable darkMode = false;

    constructor(syncLocalStorage?: boolean) {
        makeAutoObservable(this);

        if (window.localStorage && (syncLocalStorage ?? true)) {
            this.setupLocalStorageSync();
        }
    }

    @computed get asJson(): ISettingsStore {
        return {
            darkMode: this.darkMode,
        }
    }

    @action updateFromJson(json: ISettingsStore): void {
        this.darkMode = json.darkMode;
    }

    @action setDarkMode(dark: boolean): void {
        this.darkMode = dark;
    }

    @action toggleDarkMode(): void {
        this.darkMode = !this.darkMode;
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

        this.checkTheme();

        autorun(() => {
            window.localStorage.settings = JSON.stringify(this.asJson);
            this.checkTheme();
        });
    }

    private checkTheme() {
        document.querySelector('html').dataset.theme = (this.darkMode) ? 'theme-dark' : '';
    }
}

export class SessionStore {
    @observable private _node: LocalNode = null;

    constructor() {
        makeAutoObservable(this);
    }

    get node(): LocalNode {
        return this._node;
    }

    @action set node(n: LocalNode) {
        if (this._node) {
            this._node.free();
        }
        this._node = n;
    }

    @observable showDiscovery = false;

    @observable cellInitialized = false;

    @observable cellError?: string;
}

export class Stores {
    constructor(public settings: PersistedStore, public session: SessionStore) {
    }
}

export const StoresInstance: Stores = {
    settings: new PersistedStore(),
    session: new SessionStore(),
}

export const StoresContext = React.createContext<Stores | null>(null);

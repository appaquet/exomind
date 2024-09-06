import { action, autorun, computed, makeAutoObservable, observable } from "mobx";

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
        };
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
                console.error('Error parsing local storage app settings', e);
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

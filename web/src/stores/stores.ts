import React from 'react';
import { CollectionStore } from './collections';
import { PersistedStore } from './persisted';
import { SessionStore } from './session';

export interface IStores {
    readonly settings: PersistedStore;
    readonly session: SessionStore;
    readonly collections: CollectionStore;
}

export const Stores: IStores = {
    settings: new PersistedStore(),
    session: new SessionStore(),
    collections: new CollectionStore(),
};

export const StoresContext = React.createContext<IStores | null>(null);

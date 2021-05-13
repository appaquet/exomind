import { LocalNode } from "exocore";
import { action, makeAutoObservable, observable } from "mobx";
import Path from "../utils/path";

export type ModalRenderer = () => React.ReactNode;

export class SessionStore {
    @observable private _node: LocalNode = null;

    constructor() {
        makeAutoObservable(this);
    }

    @observable currentPath = new Path('/');

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

    @observable currentModal?: ModalRenderer;

    @action showModal(render: ModalRenderer): void {
        this.currentModal = render;
    }

    @action hideModal(): void {
        this.currentModal = null;
    }
}
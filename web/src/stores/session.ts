import { LocalNode } from "exocore";
import { action, makeAutoObservable, observable } from "mobx";
import Path from "../utils/path";

export type ModalRenderer = () => React.ReactNode;
export type ModalCancel = () => void;

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
    @observable modalCancel?: ModalCancel;

    @action showModal(render: ModalRenderer, modalCancel: ModalCancel | null = null): void {
        this.currentModal = render;
        this.modalCancel = modalCancel;
    }

    @action hideModal(canceled = false): void {
        this.currentModal = null;
        if (canceled) {
            this.modalCancel?.();
            this.modalCancel = null;
        }
    }
}

import * as protos from '../protos';
import { exocore } from '../protos';
export { protos, exocore }

import { CellWrapper } from './cell';
import { DiscoveryAccessor } from './discovery';
export { CellWrapper as Cell };

import { NodeAccessor } from './node';
export { NodeAccessor };

import { Registry, matchTrait } from './registry';
export { Registry, matchTrait }

import { Store, WatchedQueryWrapper, MutationBuilder, QueryBuilder, TraitQueryBuilder } from './store';
export { WatchedQueryWrapper, MutationBuilder, QueryBuilder, TraitQueryBuilder };

import * as wasm from './wasm';
import { WasmModule, ExocoreClient, LocalNode, Discovery } from './wasm';
export { WasmModule, ExocoreClient, LocalNode, Discovery };

export class Exocore {
    static default: ExocoreInstance = null;

    static get initialized(): boolean {
        return Exocore.default != null;
    }

    static async ensureLoaded(): Promise<WasmModule> {
        return await wasm.getOrLoadModule();
    }

    static async initialize(node: LocalNode): Promise<ExocoreInstance> {
        const module = await Exocore.ensureLoaded();

        let instance: ExocoreInstance;
        const onStatusChange = (status: string) => {
            instance._triggerStatusChange(status)
        }

        const innerClient = new module.ExocoreClient(node, onStatusChange);
        instance = new ExocoreInstance(innerClient, node);

        if (!Exocore.default) {
            Exocore.default = instance;
        }

        return instance;
    }

    static get cell(): CellWrapper {
        return Exocore.default.cell;
    }

    static get store(): Store {
        return Exocore.default.store;
    }

    static get registry(): Registry {
        return Exocore.default.registry;
    }

    static buildInfo(): exocore.core.BuildInfo {
        const module = wasm.getModule();
        const infoBytes = module.build_info()
        return exocore.core.BuildInfo.decode(infoBytes);
    }

    static node = new NodeAccessor();

    static discovery = new DiscoveryAccessor();

    static reset(): void {
        if (Exocore.default) {
            Exocore.default.free();
            Exocore.default = null;
        }
    }
}

export class ExocoreInstance {
    wasmClient: ExocoreClient;
    cell: CellWrapper;
    store: Store;
    status: string;
    registry: Registry;
    node: LocalNode;
    onChange?: () => void;

    constructor(client: ExocoreClient, node: LocalNode) {
        this.wasmClient = client;
        this.cell = new CellWrapper(client);
        this.store = new Store(client);
        this.registry = new Registry();
        this.node = node;
    }

    free() {
        this.wasmClient.free();
    }

    _triggerStatusChange(status: string): void {
        this.status = status;
        if (this.onChange) {
            this.onChange();
        }
    }
}

export function toProtoTimestamp(date: Date): protos.google.protobuf.ITimestamp {
    const epoch = date.getTime();
    const seconds = Math.floor(epoch / 1000);

    return new protos.google.protobuf.Timestamp({
        seconds: seconds,
        nanos: (epoch - (seconds * 1000)) * 1000000,
    });
}

export function fromProtoTimestamp(ts: protos.google.protobuf.ITimestamp): Date {
    return new Date((ts.seconds as number) * 1000 + ts.nanos / 1000000);
}
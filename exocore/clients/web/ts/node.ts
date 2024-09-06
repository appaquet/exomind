import * as wasm from './wasm';
import { LocalNode } from './wasm';

export class NodeAccessor {
    generate(): LocalNode {
        const module = wasm.getModule();
        return module.LocalNode.generate();
    }

    from_storage(storage: Storage): LocalNode {
        const module = wasm.getModule();
        return module.LocalNode.from_storage(storage);
    }

    from_yaml(yaml: string): LocalNode {
        const module = wasm.getModule();
        return module.LocalNode.from_yaml(yaml);
    }
}
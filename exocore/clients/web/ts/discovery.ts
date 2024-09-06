import * as wasm from './wasm';
import { Discovery } from './wasm';

export class DiscoveryAccessor {
    create(discoveryServiceUrl?: string): Discovery {
        const module = wasm.getModule();
        return new module.Discovery(discoveryServiceUrl);
    }
}
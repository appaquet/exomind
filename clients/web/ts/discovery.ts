import * as wasm from './wasm';
import { Discovery } from './wasm';

export class DiscoveryAccessor {
    create(discoveryServiceUrl?: string): Discovery {
        const module = wasm.getModule();
        return module.Discovery.new(discoveryServiceUrl);
    }
}
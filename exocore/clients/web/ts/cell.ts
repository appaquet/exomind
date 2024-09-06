import { ExocoreClient } from "./wasm";

export class CellWrapper {
    wasmClient: ExocoreClient;
    statusChangeCallback: () => void;

    constructor(inner: ExocoreClient) {
        this.wasmClient = inner;
    }

    generateAuthToken(expirationDays?: number): string {
        return this.wasmClient.cell_generate_auth_token(expirationDays ?? 0);
    }
}

export type CancellableEvent = {
    stopPropagation: () => void;
    preventDefault: () => void;
}
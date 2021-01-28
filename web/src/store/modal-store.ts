
import { EventEmitter } from 'fbemitter';

export type ModalRenderer = () => React.ReactNode;

export class ModalStore {
  static emitter = new EventEmitter();
  static currentRenderer?: ModalRenderer = null;

  static onChange(cb: () => void, ctx: unknown): void {
    ModalStore.emitter.addListener('change', cb, ctx);
  }

  static showModal(handler: ModalRenderer): void {
    ModalStore.currentRenderer = handler;
    ModalStore.emitter.emit('change');
  }

  static hideModal(): void {
    ModalStore.currentRenderer = null;
    ModalStore.emitter.emit('change');
  }
}


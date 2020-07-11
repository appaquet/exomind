
import { EventEmitter } from 'fbemitter';

export class ModalStore {
  static emitter = new EventEmitter();
  static currentRenderer = null;

  static onChange(cb, ctx) {
    ModalStore.emitter.addListener('change', cb, ctx);
  }

  static showModal(handler) {
    ModalStore.currentRenderer = handler;
    ModalStore.emitter.emit('change');
  }

  static hideModal() {
    ModalStore.currentRenderer = null;
    ModalStore.emitter.emit('change');
  }
}


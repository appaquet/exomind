
import { EventEmitter } from 'fbemitter';

export default class ObservableDictionary {
  emitter = new EventEmitter();
  data = {};

  onChange(cb, ctx) {
    this.emitter.addListener('change', cb, ctx);
  }

  set(key, value) {
    if (this.data[key] !== value) {
      this.data[key] = value;
      setTimeout(() => {
        this.emitter.emit('change', key);
      }, 0);
    } else {
      this.data[key] = value;
    }
  }

  get(key) {
    return this.data[key];
  }
}


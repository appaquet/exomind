
export default class Debouncer {
  private delay: number;
  private lastCallback: () => void;
  private lastTimeout?: unknown;

  constructor(delay: number) {
    this.delay = delay;
  }

  debounce(cb: ()=>void): void {
    this.lastCallback = cb;

    if (this.lastTimeout) {
      clearTimeout(this.lastTimeout as number);
      this.lastTimeout = null;
    }

    this.lastTimeout = setTimeout(() => {
      if (this.lastCallback === cb) {
        cb();
        this.lastTimeout = null;
      }
    }, this.delay);
  }
}
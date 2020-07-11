
export default class Debouncer {
  private delay: number;
  private lastCallback: () => void;

  constructor(delay: number) {
    this.delay = delay;
  }

  debounce(cb: ()=>void): void {
    this.lastCallback = cb;

    setTimeout(() => {
      if (this.lastCallback === cb) {
        cb();
      }
    }, this.delay);
  }
}
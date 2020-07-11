
export default class Path {
  components: string[];
  str: string;

  constructor(path: Path | string | string[]) {
    if (Array.isArray(path)) {
      path = path.join('/');
    }

    if (path instanceof Path) {
      this.components = path.components;
      this.str = path.str;
    } else {
      this.components = path.split('/').map(s => s.trim()).filter(s => s !== '');
      this.str = this.components.join('/');
    }
  }

  isRoot(): boolean {
    return this.components.length === 0;
  }

  take(n: number): Path {
    return new Path(this.components.slice(0, n));
  }

  drop(n: number): Path {
    return new Path(this.components.slice(n));
  }

  pop(n: number): Path {
    return new Path(this.components.slice(0, -n));
  }

  equals(other: Path | string | string[]): boolean {
    if (!(other instanceof Path)) {
      other = new Path(other);
    }
    return other.str === this.str;
  }

  toString(): string {
    return this.str;
  }
}


import { runInAction } from 'mobx';
import { ColumnConfigs } from './components/pages/columns/columns-config';
import { Stores } from './stores/stores';
import { EntityTraits } from './utils/entities';
import Path from './utils/path';

export interface INavigationHost {
  history: IHistory;
  openPopup(path: Path): void;
  closeWindow(): void;
  openExternal(url: string): void;
}

export default class Navigation {
  static host: INavigationHost;
  static history: IHistory;

  static initialize(host: INavigationHost): void {
    Navigation.host = host;
    Navigation.history = host.history;
    Navigation.currentPath = host.history.initialPath;
  }

  static get currentPath(): Path {
    return Stores.session.currentPath;
  }

  static set currentPath(path: Path) {
    runInAction(() => {
      if (!path.equals(Stores.session.currentPath)) {
        Stores.session.currentPath = path;
      }
    });
  }

  static navigate(path: string | Path, replace = false): void {
    try {
      const obj = new Path(path);
      Navigation.history.push(obj, replace);
      Navigation.currentPath = obj;
    } catch (e) {
      console.error('failed to load link', e);
    }
  }

  static navigateBack(): void {
    Navigation.history.back();
  }

  static navigateForward(): void {
    Navigation.history.forward();
  }

  static navigatePopup(path: string | Path): void {
    const obj = new Path(path);
    Navigation.host.openPopup(obj);
  }

  static closeWindow(): void {
    Navigation.host.closeWindow();
  }

  static navigateExternal(url: string): void {
    Navigation.host.openExternal(url);
  }

  static pathForInbox(): string {
    return Navigation.pathForColumnsConfig(ColumnConfigs.forInbox());
  }

  static pathForSnoozed(): string {
    return Navigation.pathForColumnsConfig(ColumnConfigs.forSnoozed());
  }

  static pathForRecent(): string {
    return Navigation.pathForColumnsConfig(ColumnConfigs.forRecent());
  }

  static pathForSearch(keywords: string): string {
    return Navigation.pathForColumnsConfig(ColumnConfigs.forSearch(keywords));
  }

  static pathForSettings(): string {
    return '/t';
  }

  static pathForEntity(entity: EntityTraits | string): string {
    let entityId;
    if (typeof entity == 'string') {
      entityId = entity;
    } else {
      entityId = entity.id;
    }

    if (entityId == "inbox") {
      return Navigation.pathForInbox();
    } else {
      return Navigation.pathForColumnsConfig(ColumnConfigs.forEntity(entityId));
    }
  }

  static pathForColumnsConfig(config: ColumnConfigs): string {
    return '/c/' + config;
  }

  static pathForFullscreen(entityId: string): string {
    return '/s/' + entityId;
  }

  static pathForNodeConfig(): string {
    return '/nc';
  }

  static isColumnsPath(path: Path): boolean {
    return new Path(path).take(1).toString() === 'c';
  }

  static isFullscreenPath(path: Path): boolean {
    return new Path(path).take(1).toString() === 's';
  }

  static isNodeConfigPath(path: Path): boolean {
    return new Path(path).take(1).toString() === 'nc';
  }

  static isSettingsPath(path: Path): boolean {
    return new Path(path).take(1).toString() === 't';
  }
}

export function setupLinkClickNavigation(fallback: (e: MouseEvent, el: HTMLElement) => void): void {
  document.addEventListener('click', (e) => {
    // if tagname is not a link, try to go up into the parenthood up 20 levels
    let el = e.target as HTMLElement;
    for (let i = 0; el.tagName !== 'A' && i < 20; i++) {
      if (el.parentNode) {
        el = el.parentNode as HTMLElement;
      }
    }

    if (el.tagName === 'A') {
      if (el.getAttribute('target') == 'local' || (el.getAttribute('rel')?.indexOf('nofollow') ?? -1) >= 0) {
        // if target is marked as local or nofollow, it's handled elsewhere
        return false;
      }

      const url = el.getAttribute('href');

      // if it's a local URL, we catch it and send it to navigation
      if (url.startsWith('/') || url.startsWith(window.location.origin) && !el.getAttribute('target')) {
        Navigation.navigate(url);
        e.preventDefault();
        e.stopPropagation();
        return false;
      }

      if (fallback) {
        fallback(e, el);
      }
    }
  });
}

export interface IHistory {
  initialPath: Path;
  push(path: Path, replace: boolean): void;
  back(): void;
  forward(): void;
}

export class InMemoryHistory implements IHistory {
  private backHistory: Path[] = [];
  private forwardHistory: Path[] = [];
  initialPath: Path = new Path('');

  constructor() {
    this.backHistory.push(this.initialPath);
  }

  push(path: Path, replace: boolean): void {
    if (!this.curPath()) {
      return;
    }

    if (replace) {
      this.backHistory.pop();
    }
    this.backHistory.push(path);
    this.forwardHistory = [];

    Navigation.currentPath = path;
  }

  back(): void {
    const curPath = this.curPath();
    if (curPath) {
      this.forwardHistory.push(curPath);
    }

    this.backHistory.pop(); // remove current path

    const backPath = this.curPath();
    if (backPath) {
      Navigation.currentPath = backPath;
    }
  }

  forward(): void {
    const forwardPath = this.forwardHistory.pop();
    if (forwardPath) {
      this.backHistory.push(forwardPath);
      Navigation.currentPath = forwardPath;
    }
  }

  private curPath(): Path | null {
    if (this.backHistory.length > 0) {
      return this.backHistory[this.backHistory.length - 1];
    } else {
      return null;
    }
  }
}

export class BrowserHistory implements IHistory {
  initialPath: Path = BrowserHistory.getBrowserPath();

  constructor() {
    window.onpopstate = () => {
      const path = BrowserHistory.getBrowserPath();
      Navigation.currentPath = path;
    };
  }

  push(path: Path, replace: boolean): void {
    if (replace) {
      window.history.replaceState({}, null, '/' + path.toString());
    } else {
      window.history.pushState({}, null, '/' + path.toString());
    }
  }

  back(): void {
    window.history.back();
  }

  forward(): void {
    window.history.forward();
  }

  private static getBrowserPath(): Path {
    return new Path(window.location.pathname.replace('/', ''));
  }
}
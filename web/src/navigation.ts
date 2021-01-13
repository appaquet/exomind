import { EventEmitter } from 'fbemitter';
import { ColumnsConfig } from './components/pages/columns/columns-config';
import { EntityTraits } from './store/entities';
import Path from './utils/path';

export interface INavigationHost {
  initialPath: Path,

  pushHistory(path: Path, replace: boolean): void;

  openPopup(path: Path): void;

  openExternal(url: string): void;
}

export default class Navigation {
  static currentPath: Path = null;
  static emitter = new EventEmitter();
  static host: INavigationHost;

  static initialize(host: INavigationHost): void {
    Navigation.currentPath = host.initialPath;
    Navigation.host = host;
  }

  static onNavigate(cb: () => void, ctx: unknown): void {
    Navigation.emitter.addListener('change', cb, ctx);
  }

  static notifyChange(): void {
    // set timeout needed since we may do it from within a renderer
    setTimeout(() => {
      Navigation.emitter.emit('change');
    }, 1);
  }

  static navigate(path: string | Path, replace = false): void {
    const obj = new Path(path);

    Navigation.host.pushHistory(obj, replace);
    Navigation.currentPath = obj;
    Navigation.notifyChange();
  }

  static navigateBack(): void {
    if (window.history) {
      window.history.back();
    }
  }

  static navigateForward(): void {
    if (window.history) {
      window.history.forward();
    }
  }

  static navigatePopup(path: string | Path): void {
    const obj = new Path(path);
    Navigation.host.openPopup(obj);
  }

  static navigateExternal(url: string): void {
    Navigation.host.openExternal(url);
  }

  static pathForInbox(): string {
    return Navigation.pathForColumnsConfig(ColumnsConfig.forInbox());
  }

  static pathForSnoozed(): string {
    return Navigation.pathForColumnsConfig(ColumnsConfig.forSnoozed());
  }

  static pathForRecent(): string {
    return Navigation.pathForColumnsConfig(ColumnsConfig.forRecent());
  }

  static pathForSearch(keywords: string): string {
    return Navigation.pathForColumnsConfig(ColumnsConfig.forSearch(keywords));
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
      return Navigation.pathForColumnsConfig(ColumnsConfig.forEntity(entityId));
    }
  }

  static pathForColumnsConfig(config: ColumnsConfig): string {
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
    let el = e.target as HTMLElement;

    // if tagname is not a link, try to go up into the parenthood up 10 levels
    for (let i = 0; el.tagName !== 'A' && i < 10; i++) {
      if (el.parentNode) {
        el = el.parentNode as HTMLElement;
      }
    }

    if (el.tagName === 'A') {
      if (el.getAttribute('target') == 'local') {
        // if target is marked as local, it means it's handled by another component
        // Ex: `note.tsx`
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
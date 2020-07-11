import { EventEmitter } from 'fbemitter';
import Constants from './constants';
import { ColumnsConfig } from './components/pages/columns/columns-config';
import { EntityTraits } from './store/entities';
import Path from './utils/path';
import { exocore } from 'exocore';

export default class Navigation {
  static currentPath: Path = null;
  static emitter = new EventEmitter();

  static onNavigate(cb: () => void, ctx: unknown): void {
    Navigation.emitter.addListener('change', cb, ctx);
  }

  static notifyChange(): void {
    // set timeout needed since we may do it from within a renderer
    setTimeout(() => {
      Navigation.emitter.emit('change');
    }, 1);
  }

  static navigate(path: string, replace = false): void {
    const obj = new Path(path);

    if (window.history) {
      if (replace) {
        window.history.replaceState({}, null, Constants.basePath + obj.toString());
      } else {
        window.history.pushState({}, null, Constants.basePath + obj.toString());
      }
    }
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

  static getBrowserCurrentPath(): Path {
    return new Path(window.location.pathname.replace(Constants.basePath, ''));
  }

  static pathForInbox(): string {
    return Navigation.pathForColumnsConfig(ColumnsConfig.forInbox());
  }

  static pathForSnoozed(): string {
    return Navigation.pathForColumnsConfig(ColumnsConfig.forSnoozed());
  }

  static pathForHistory(): string {
    return Navigation.pathForColumnsConfig(ColumnsConfig.forHistory());
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

  static pathForFullscreen(entity: exocore.index.IEntity): string {
    return '/s/' + entity.id;
  }

  static pathForBootstrap(): string {
    return '/bootstrap';
  }

  static isColumnsPath(path: Path): boolean {
    return new Path(path).take(1).toString() === 'c';
  }

  static isFullscreenPath(path: Path): boolean {
    return new Path(path).take(1).toString() === 's';
  }

  static isBootstrapPath(path: Path): boolean {
    return new Path(path).take(1).toString() === 'bootstrap';
  }

  static isSettingsPath(path: Path): boolean {
    return new Path(path).take(1).toString() === 't';
  }
}

window.onpopstate = () => {
  Navigation.currentPath = Navigation.getBrowserCurrentPath();
  Navigation.notifyChange();
};

Navigation.currentPath = Navigation.getBrowserCurrentPath();